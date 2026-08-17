#![allow(clippy::needless_pass_by_value)] // It's ok here as it is an example
use eframe::{emath::Align, NativeOptions};
use egui::{Context, Id, Layout, Panel, Popup, TextEdit, Widget, Window, CentralPanel};
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
                        let menu_button = ui.button("☰");

                        Popup::menu(&menu_button).show(|ui| {
                            ui.set_width(ui.min_size().x);
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
}

impl BrowserApp {
    pub fn new(ctx: &Context) -> Self {
        init_webview(ctx);

        Self {
            tabs: Vec::new(),
            active_tab: None,
            sidebar_open: true,
            url_input: String::new(),
            next_tab_id: 0,
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

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // 1. Left Sidebar Panel (collapsible, resizable)
        Panel::left("browser_sidebar")
            .resizable(true)
            .default_size(100.0)
            .size_range(80.0..=200.0)
            .show_collapsible(ui, &mut self.sidebar_open, |ui| {
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

        // Placeholder for toolbar and content (to be replaced in next task)
        ui.label("TODO: Add toolbar and content panels");
    }
}

pub fn main() -> eframe::Result<()> {
    // Initialize GTK for webview support on Linux/OpenBSD
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    gtk::init().expect("Failed to initialize GTK");

    let mut browser_app: Option<BrowserApp> = None;

    eframe::run_ui_native(
        "Tabbed Browser",
        NativeOptions::default(),
        move |ui, frame| {
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

            egui_extras::install_image_loaders(ui.ctx());

            // CRITICAL: Request continuous repainting for responsive webview input
            ui.ctx().request_repaint();

            // Initialize BrowserApp on first frame
            if browser_app.is_none() {
                browser_app = Some(BrowserApp::new(ui.ctx()));
            }

            // Render BrowserApp UI
            if let Some(app) = &mut browser_app {
                app.ui(ui, frame);
            }

            webview_end_frame(ui.ctx());
        },
    )
}
