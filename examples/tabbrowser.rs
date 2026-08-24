#![allow(clippy::needless_pass_by_value)] // It's ok here as it is an example
use eframe::{emath::Align, App, NativeOptions};
use egui::{CentralPanel, Context, Id, Layout, SidePanel, TextEdit, TopBottomPanel, Widget, Window};
use wry::raw_window_handle::HasWindowHandle;

use egui_webview::{init_webview, webview_end_frame, EguiWebView, WebViewEvent};

pub struct WebBrowser {
    id: Id,
    url_bar: String,
    view: EguiWebView,
}

impl Drop for WebBrowser {
    fn drop(&mut self) {
        println!("🗑️  Dropping WebBrowser {:?}", self.id);
    }
}

impl WebBrowser {
    pub fn new(ctx: &Context, id: Id, url: &str, window: &impl HasWindowHandle) -> Self {
        let view = EguiWebView::new(ctx, id, window, |b| b.with_url(url));

        Self {
            id,
            url_bar: url.to_string(),
            view,
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> bool {
        let mut open = true;
        Window::new("Browser")
            .id(self.id)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(800.0);
                    ui.set_height(1024.0);

                    // Button icon arrow left
                    if ui.button("◀").clicked() {
                        self.view.back();
                    }

                    if ui.button("▶").clicked() {
                        self.view.forward();
                    }
                    ui.label("URL:");

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.menu_button("☰", |ui| {
                            ui.set_min_width(150.0);
                            let _ = ui.button("I have no function");
                            let _ = ui.button("My existence is meaningless");
                            if ui.button("Why did you click me?").clicked() {
                                self.view
                                    .view
                                    .load_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                                    .unwrap();
                            }
                        });

                        let btn_resp = ui.button("Open");
                        let text_resp = TextEdit::singleline(&mut self.url_bar)
                            .desired_width(ui.available_width())
                            .ui(ui);

                        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || btn_resp.clicked()
                        {
                            self.view.view.load_url(&self.url_bar).unwrap();
                        }
                    });
                });

                self.view
                    .ui(ui, ui.available_size())
                    .events
                    .into_iter()
                    .for_each(|e| {
                        if let WebViewEvent::Loaded(url) = e {
                            self.url_bar = url;
                        }
                    });
            });

        if !open {
            println!("❌ Window {:?} close requested", self.id);
        }

        open
    }
}

pub struct BrowserApp {
    tabs: Vec<WebBrowser>,
    active_tab: Option<usize>,
    sidebar_open: bool,
    url_input: String,
    next_tab_id: usize,
    initialized: bool,
}

impl BrowserApp {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            sidebar_open: true,
            url_input: String::new(),
            next_tab_id: 0,
            initialized: false,
        }
    }

    fn add_tab(&mut self, ctx: &Context, frame: &impl HasWindowHandle, url: &str) {
        let tab = WebBrowser::new(
            ctx,
            Id::new(format!("tab_{}", self.next_tab_id)),
            url,
            frame,
        );

        self.tabs.push(tab);
        self.active_tab = Some(self.tabs.len() - 1);
        self.url_input = url.to_string();
        self.next_tab_id += 1;
    }

    fn close_tab(&mut self, idx: usize) {
        self.tabs.remove(idx);

        if self.tabs.is_empty() {
            self.active_tab = None;
            self.url_input.clear();
        } else if let Some(active) = self.active_tab {
            if active >= self.tabs.len() {
                // Active tab was beyond the removed tab, adjust index
                self.active_tab = Some(self.tabs.len() - 1);
            } else if idx <= active && active > 0 {
                // Removed tab was before or at active, shift active left
                self.active_tab = Some(active - 1);
            }

            // Sync URL bar with new active tab
            if let Some(new_active) = self.active_tab {
                self.url_input = self.tabs[new_active].url_bar.clone();
            }
        }
    }

    pub fn ui(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // 1. Left Sidebar Panel (collapsible, resizable)
        SidePanel::left("browser_sidebar")
            .resizable(true)
            .default_width(100.0)
            .width_range(80.0..=200.0)
            .show_animated(ctx, self.sidebar_open, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Sidebar");
                });
                ui.separator();
                ui.label("This is some demo text in the sidebar.");
                ui.label("You can add more content here later.");
                ui.add_space(10.0);
                ui.label("The sidebar is resizable by dragging the edge.");
                ui.label("Click the arrow to collapse/expand.");
            });

        // 2. Top Toolbar Panel (navigation controls)
        TopBottomPanel::top("browser_toolbar")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Sidebar toggle (leftmost position)
                    if self.sidebar_open {
                        if ui.button("◀◀").clicked() {
                            self.sidebar_open = false;
                        }
                    } else {
                        if ui.button("▶▶").clicked() {
                            self.sidebar_open = true;
                        }
                    }

                    ui.separator();

                    // Back button
                    if ui.button("◀").clicked() {
                        if let Some(idx) = self.active_tab {
                            self.tabs[idx].view.back();
                        }
                    }

                    // Forward button
                    if ui.button("▶").clicked() {
                        if let Some(idx) = self.active_tab {
                            self.tabs[idx].view.forward();
                        }
                    }

                    ui.separator();

                    // URL input
                    ui.label("URL:");
                    let response = ui.add(
                        TextEdit::singleline(&mut self.url_input)
                            .desired_width(ui.available_width() - 60.0)
                    );

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(idx) = self.active_tab {
                            if let Err(e) = self.tabs[idx].view.view.load_url(&self.url_input) {
                                eprintln!("Failed to load URL: {}", e);
                            }
                        }
                    }

                    // Go button
                    if ui.button("Go").clicked() {
                        if let Some(idx) = self.active_tab {
                            if let Err(e) = self.tabs[idx].view.view.load_url(&self.url_input) {
                                eprintln!("Failed to load URL: {}", e);
                            }
                        }
                    }
                });
            });

        // 3. Central Panel (tabs + browser content)
        CentralPanel::default().show(ctx, |ui| {
            if self.tabs.is_empty() {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No tabs open");
                        ui.label("Click '+' to create a new tab.");
                        ui.add_space(20.0);
                        if ui.button("+ New Tab").clicked() {
                            self.add_tab(ui.ctx(), frame, "https://dure.app");
                        }
                    });
                });
            } else {
                // Tab bar with horizontal scrolling
                egui::ScrollArea::horizontal()
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut tab_to_close: Option<usize> = None;

                            for (idx, tab) in self.tabs.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        // Tab button
                                        if ui.selectable_label(
                                            self.active_tab == Some(idx),
                                            format!("Tab {}", idx + 1)
                                        ).clicked() {
                                            self.active_tab = Some(idx);
                                            self.url_input = tab.url_bar.clone();
                                        }

                                        // Close button
                                        if ui.small_button("×").clicked() {
                                            tab_to_close = Some(idx);
                                        }
                                    });
                                });
                            }

                            // + button to add new tab
                            if ui.button("+").clicked() {
                                self.add_tab(ui.ctx(), frame, "https://dure.app");
                            }

                            // Close tab after iteration (avoid borrow conflict)
                            if let Some(idx) = tab_to_close {
                                self.close_tab(idx);
                            }
                        });
                    });

                ui.separator();

                // Browser content area - render all webviews, show only active
                for (idx, tab) in self.tabs.iter_mut().enumerate() {
                    let is_active = Some(idx) == self.active_tab;
                    let size = if is_active {
                        ui.available_size()
                    } else {
                        egui::vec2(0.0, 0.0)
                    };

                    ui.push_id(tab.id, |ui| {
                        let response = tab.view.ui(ui, size);

                        // Handle WebView events only for active tab
                        if is_active {
                            for event in response.events {
                                if let WebViewEvent::Loaded(url) = event {
                                    tab.url_bar = url.clone();
                                    self.url_input = url;
                                }
                            }
                        }
                    });
                }
            }
        });
    }
}

impl App for BrowserApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Process GTK events for webview
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }

        // Initialize webview system on first frame
        if !self.initialized {
            init_webview(ctx);
            egui_extras::install_image_loaders(ctx);
            self.initialized = true;
        }

        // CRITICAL: Request continuous repainting for responsive webview input
        ctx.request_repaint();

        // Render BrowserApp UI
        self.ui(ctx, frame);

        webview_end_frame(ctx);
    }
}

pub fn main() -> eframe::Result<()> {
    // wry's webkitgtk backend can only embed a child WebView into an X11 window (via XEmbed);
    // under a native Wayland session `build_as_child` fails with `UnsupportedWindowHandle`.
    // Force winit and GTK onto X11 (via XWayland) before any window/display is created.
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    // Safety: called at the very start of `main`, before any other threads exist.
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::set_var("GDK_BACKEND", "x11");
    }

    // Initialize GTK for webview support on Linux/OpenBSD
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    gtk::init().expect("Failed to initialize GTK");

    let native_options = NativeOptions::default();

    eframe::run_native(
        "Tabbed Browser",
        native_options,
        Box::new(|_cc| Ok(Box::new(BrowserApp::new()))),
    )
}
