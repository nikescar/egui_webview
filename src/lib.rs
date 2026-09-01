use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};

use egui::{Context, Id, Sense, Ui, Vec2};
use serde::{Deserialize, Serialize};

// Platform-specific WebView implementations
#[cfg(target_os = "android")]
mod android_webview;

#[cfg(not(target_os = "android"))]
use wry::dpi::{Position, Size};
#[cfg(not(target_os = "android"))]
use wry::raw_window_handle::HasWindowHandle;
#[cfg(not(target_os = "android"))]
use wry::{PageLoadEvent, WebView};

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
)))]
use wry::WebViewBuilderExtUnix;

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
)))]
pub fn create_gtk_container() -> gtk::Fixed {
    use gtk::prelude::*;
    let container = gtk::Fixed::new();
    container.set_visible(true);
    container
}

// Desktop version uses wry
#[cfg(not(target_os = "android"))]
pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    id: Id,
    events: Arc<Mutex<VecDeque<WebViewEvent>>>,
    #[allow(dead_code)]
    context: Context,
    last_bounds: Option<wry::Rect>,
}

// Android version uses JNI WebView
#[cfg(target_os = "android")]
pub struct EguiWebView {
    pub view: Arc<android_webview::AndroidWebView>,
    id: Id,
    events: Arc<Mutex<VecDeque<WebViewEvent>>>,
    #[allow(dead_code)]
    context: Context,
}

impl Debug for EguiWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiWebView").field("id", &self.id).finish()
    }
}

impl Drop for EguiWebView {
    fn drop(&mut self) {
        println!("🗑️  Dropping EguiWebView {:?}", self.id);
        println!("   Arc strong count: {}", Arc::strong_count(&self.view));

        // Explicitly hide the webview before dropping
        self.view.set_visible(false).ok();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JsEvent {
    event: JsEventType,
    __egui_webview: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum JsEventType {
    Focus,
    Blur,
}

pub enum WebViewEvent {
    Focus,
    Blur,
    Loading(String),
    Loaded(String),
    Ipc(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PageCommand {
    // Screenshot,
    Click { x: f32, y: f32 },
    Back,
    Forward,
}

pub struct WebViewResponse {
    pub events: Vec<WebViewEvent>,
    pub egui_response: egui::Response,
    pub webview_visible: bool,
}

impl EguiWebView {
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    pub fn new_with_container(
        ctx: &Context,
        id: impl Into<Id>,
        container: &gtk::Fixed,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let events_clone = events.clone();
        let events_clone2 = events.clone();

        let id = id.into();
        ctx.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                )
                .clone()
        });

        let mut builder = wry::WebViewBuilder::new();

        builder = build(builder);

        #[allow(clippy::arc_with_non_send_sync)]
        let view_ref = Arc::new(Mutex::new(None::<Arc<WebView>>));
        let view_ref_weak = view_ref.clone();
        let ctx_clone = ctx.clone();

        builder = builder
            .with_devtools(true)
            .with_on_page_load_handler(move |event, url| {
                match event {
                    PageLoadEvent::Started => {
                        if let Ok(guard) = view_ref_weak.lock() {
                            if let Some(view) = guard.as_ref() {
                                if let Err(err) = view.evaluate_script(include_str!("webview.js")) {
                                    println!("Error loading webview script: {err}");
                                }
                            }
                        }
                    }
                    PageLoadEvent::Finished => {}
                }

                if let Ok(mut queue) = events_clone.lock() {
                    queue.push_back(WebViewEvent::Loaded(url));
                }
            })
            .with_ipc_handler(move |msg: http::Request<String>| {
                let result = Self::handle_js_event(msg.body().clone(), &ctx_clone);
                if let Ok(mut queue) = events_clone2.lock() {
                    queue.push_back(result);
                }
            });

        // Use GTK-native rendering for much better performance
        #[allow(clippy::arc_with_non_send_sync)]
        let web_view = Arc::new(builder.build_gtk(container).unwrap());

        *view_ref.lock().unwrap() = Some(web_view.clone());

        // Set visible once - never toggle visibility for performance
        web_view.set_visible(true).ok();

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.views.insert(id, Arc::downgrade(&web_view));
        });

        Self {
            events,
            view: web_view,
            id,
            context: ctx.clone(),
            last_bounds: None,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        window: &impl HasWindowHandle,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let events_clone = events.clone();
        let events_clone2 = events.clone();

        let id = id.into();
        ctx.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                )
                .clone()
        });

        let mut builder = wry::WebViewBuilder::new();

        builder = build(builder);

        #[allow(clippy::arc_with_non_send_sync)]
        let view_ref = Arc::new(Mutex::new(None::<Arc<WebView>>));
        let view_ref_weak = view_ref.clone();
        let ctx_clone = ctx.clone();

        builder = builder
            .with_devtools(true)
            .with_on_page_load_handler(move |event, url| {
                match event {
                    PageLoadEvent::Started => {
                        if let Ok(guard) = view_ref_weak.lock() {
                            if let Some(view) = guard.as_ref() {
                                if let Err(err) = view.evaluate_script(include_str!("webview.js")) {
                                    println!("Error loading webview script: {err}");
                                }
                            }
                        }
                    }
                    PageLoadEvent::Finished => {}
                }

                if let Ok(mut queue) = events_clone.lock() {
                    queue.push_back(WebViewEvent::Loaded(url));
                }
            })
            .with_ipc_handler(move |msg: http::Request<String>| {
                let result = Self::handle_js_event(msg.body().clone(), &ctx_clone);
                if let Ok(mut queue) = events_clone2.lock() {
                    queue.push_back(result);
                }
            });

        #[allow(clippy::arc_with_non_send_sync)]
        let web_view = Arc::new(builder.build_as_child(window).unwrap());

        *view_ref.lock().unwrap() = Some(web_view.clone());

        // Set visible once - never toggle visibility for performance
        web_view.set_visible(true).ok();

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.views.insert(id, Arc::downgrade(&web_view));
        });

        Self {
            events,
            view: web_view,
            id,
            context: ctx.clone(),
            last_bounds: None,
        }
    }

    #[cfg(target_os = "android")]
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        url: &str,
    ) -> Self {
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let id = id.into();

        ctx.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                )
                .clone()
        });

        // Create Android WebView
        let web_view = Arc::new(
            android_webview::AndroidWebView::new(url)
                .expect("Failed to create Android WebView")
        );

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.views.insert(id, Arc::downgrade(&web_view));
        });

        Self {
            events,
            view: web_view,
            id,
            context: ctx.clone(),
        }
    }

    fn handle_js_event(msg: String, _ctx: &Context) -> WebViewEvent {
        let event = serde_json::from_str::<JsEvent>(&msg).map(|e| e.event);

        match event {
            Ok(JsEventType::Focus) => WebViewEvent::Focus,
            Ok(JsEventType::Blur) => WebViewEvent::Blur,
            Err(_) => WebViewEvent::Ipc(msg),
        }
    }


    #[allow(clippy::needless_pass_by_value)]
    fn send_command(&self, command: PageCommand) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&command)?;
        self.view
            .evaluate_script(&format!("__egui_webview_handle_command({json})"))?;
        Ok(())
    }

    pub fn back(&self) {
        self.send_command(PageCommand::Back).ok();
    }

    pub fn forward(&self) {
        self.send_command(PageCommand::Forward).ok();
    }

    /// Pause media playback and reduce resource usage
    pub fn pause(&self) {
        self.view
            .evaluate_script("document.querySelectorAll('video, audio').forEach(el => el.pause())")
            .ok();
    }

    /// Resume media playback
    pub fn resume(&self) {
        self.view
            .evaluate_script("document.querySelectorAll('video, audio').forEach(el => el.play())")
            .ok();
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) -> WebViewResponse {
        let response = ui.allocate_response(size, Sense::click());

        // Drain events from the queue
        let events: Vec<WebViewEvent> = if let Ok(mut queue) = self.events.lock() {
            queue.drain(..).inspect(|e| match e {
                WebViewEvent::Focus => {
                    ui.memory_mut(|mem| mem.request_focus(response.id));
                }
                _ => {}
            }).collect()
        } else {
            Vec::new()
        };

        if response.clicked() {
            response.request_focus();
            let pos = response.hover_pos();
            if let Some(pos) = pos {
                let relative = (pos - response.rect.min) / ui.ctx().pixels_per_point();

                self.send_command(PageCommand::Click {
                    x: relative.x,
                    y: relative.y,
                })
                .ok();
            }
        }

        if response.gained_focus() {
            self.view.focus().ok();
        }

        // Track visible webviews for end-of-frame visibility management
        ui.ctx().memory_mut(|mem| {
            let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.rendered_this_frame.insert(self.id);
        });

        // Convert egui points to physical pixels using pixels_per_point
        // This handles different DPI/zoom ratios across monitors
        #[cfg(not(target_os = "android"))]
        {
            let pixels_per_point = ui.ctx().pixels_per_point();
            let physical_rect = response.rect * pixels_per_point;

            // Only update bounds if size/position changed (major performance optimization)
            let new_bounds = wry::Rect {
                position: Position::Physical(wry::dpi::PhysicalPosition::new(
                    physical_rect.min.x as i32,
                    physical_rect.min.y as i32,
                )),
                size: Size::Physical(wry::dpi::PhysicalSize::new(
                    physical_rect.width() as u32,
                    physical_rect.height() as u32,
                )),
            };

            // Compare bounds manually since wry::Rect doesn't implement PartialEq
            let bounds_changed = if let Some(last) = &self.last_bounds {
                // Extract physical positions and sizes for comparison
                let (last_x, last_y) = match last.position {
                    Position::Physical(p) => (p.x, p.y),
                    Position::Logical(p) => (p.x as i32, p.y as i32),
                };
                let (last_w, last_h) = match last.size {
                    Size::Physical(s) => (s.width, s.height),
                    Size::Logical(s) => (s.width as u32, s.height as u32),
                };

                last_x != physical_rect.min.x as i32
                    || last_y != physical_rect.min.y as i32
                    || last_w != physical_rect.width() as u32
                    || last_h != physical_rect.height() as u32
            } else {
                true // First time, always update
            };

            if bounds_changed {
                self.view.set_bounds(new_bounds).ok();
                self.last_bounds = Some(new_bounds);
            }
        }

        // Android: bounds management handled by Android layout system
        #[cfg(target_os = "android")]
        {
            // Android WebView bounds are managed by the view hierarchy
            // No need to manually set bounds here
        }

        WebViewResponse {
            events,
            egui_response: response,
            webview_visible: true,
        }
    }

}

#[cfg(not(target_os = "android"))]
#[derive(Clone, Debug)]
struct GlobalWebViewState {
    views: HashMap<Id, Weak<WebView>>,
    rendered_this_frame: HashSet<Id>,
}

#[cfg(target_os = "android")]
#[derive(Clone, Debug)]
struct GlobalWebViewState {
    views: HashMap<Id, Weak<android_webview::AndroidWebView>>,
    rendered_this_frame: HashSet<Id>,
}

#[allow(unsafe_code)]
unsafe impl Send for GlobalWebViewState {}
#[allow(unsafe_code)]
unsafe impl Sync for GlobalWebViewState {}

pub const WEBVIEW_ID: &str = "egui_webview";

pub fn init_webview(ctx: &Context) {
    ctx.memory_mut(|mem| {
        if mem
            .data
            .get_temp::<GlobalWebViewState>(Id::new(WEBVIEW_ID))
            .is_some()
        {
            return;
        }
        mem.data.insert_temp(
            Id::new(WEBVIEW_ID),
            GlobalWebViewState {
                rendered_this_frame: HashSet::new(),
                views: HashMap::new(),
            },
        );
    });
}

pub fn webview_end_frame(ctx: &Context) {
    ctx.memory_mut(|mem| {
        let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
            Id::new(WEBVIEW_ID),
            || unreachable!(),
        );

        let before_count = state.views.len();

        // Keep webviews visible - manage positioning with bounds only
        // Toggling visibility every frame is a major performance bottleneck
        state.views.retain(|id, view| {
            let can_upgrade = view.upgrade().is_some();
            if !can_upgrade {
                println!("🧹 Cleaning up dead weak reference for {:?}", id);
            }
            can_upgrade
        });

        let after_count = state.views.len();
        if before_count != after_count {
            println!("📊 Global views: {} → {}", before_count, after_count);
        }

        state.rendered_this_frame.clear();
    });
}

// Android-specific implementation
#[cfg(target_os = "android")]
impl EguiWebView {
    /// Create a new Android WebView
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        _window: &impl std::any::Any, // Ignored on Android
        build: impl FnOnce(&str) -> String,
    ) -> Self {
        let id = id.into();
        let events = Arc::new(Mutex::new(VecDeque::new()));

        // Get initial URL from builder closure
        let url = build("about:blank");

        // Create Android WebView via JNI
        let view = Arc::new(
            android_webview::AndroidWebView::new(&url)
                .expect("Failed to create Android WebView")
        );

        Self {
            view,
            id,
            events,
            context: ctx.clone(),
        }
    }

    /// Render the webview in the UI (Android version)
    pub fn ui(&mut self, ui: &mut Ui) -> WebViewResponse {
        // Allocate space for webview in the UI
        let response = ui.allocate_response(
            ui.available_size(),
            Sense::click_and_drag(),
        );

        // Collect events
        let events = if let Ok(mut queue) = self.events.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        };

        // Android WebView bounds are managed by Android layout system
        // No need to set bounds like on desktop

        WebViewResponse {
            events,
            egui_response: response,
            webview_visible: true,
        }
    }

    /// Navigate to URL
    pub fn load_url(&self, url: &str) {
        self.view.load_url(url).ok();
    }

    /// Go back
    pub fn go_back(&self) {
        self.view.go_back().ok();
    }

    /// Go forward
    pub fn go_forward(&self) {
        self.view.go_forward().ok();
    }
}
