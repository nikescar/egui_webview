#![allow(clippy::needless_pass_by_value)] // It's ok here as it is an example
use eframe::{emath::Align, NativeOptions};
use egui::{Context, Id, Layout, Popup, TextEdit, Widget, Window};
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
                            ui.set_width(ui.min_size().x + 200.0);
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

    pub fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            // 1. Left Sidebar (manual layout)
            if self.sidebar_open {
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    egui::Frame::new()
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            ui.set_min_width(280.0);
                            ui.set_max_width(400.0);

                            ui.vertical_centered(|ui| {
                                ui.heading("💻 Sidebar");
                            });
                            ui.separator();
                            ui.label("This is some demo text in the sidebar.");
                            ui.label("You can add more content here later.");
                            ui.add_space(10.0);
                            ui.label("The sidebar is resizable by dragging the edge.");
                            ui.label("Click the arrow to collapse/expand.");

                            if ui.button("Hide Sidebar").clicked() {
                                self.sidebar_open = false;
                            }
                        });
                });
                ui.separator();
            } else {
                if ui.button("Show Sidebar").clicked() {
                    self.sidebar_open = true;
                }
                ui.separator();
            }

            // 2. Main content area (toolbar + content)
            ui.vertical(|ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    // Back button
                    if ui.button("◀").clicked() {
                        // Navigation logic will be added later
                    }

                    // Forward button
                    if ui.button("▶").clicked() {
                        // Navigation logic will be added later
                    }

                    ui.separator();

                    // URL input
                    ui.label("URL:");
                    let response = ui.add(
                        TextEdit::singleline(&mut self.url_input)
                            .desired_width(ui.available_width() - 60.0)
                    );

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Navigation logic will be added later
                    }

                    // Go button
                    if ui.button("Go").clicked() {
                        // Navigation logic will be added later
                    }
                });

                ui.separator();

                // Content area (empty state for now)
                ui.centered_and_justified(|ui| {
                    ui.heading("No tabs open");
                    ui.label("Click '+' to create a new tab.");
                });
            });
        });
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
