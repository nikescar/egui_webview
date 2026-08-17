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
                        });
                });
            }
            ui.separator();

            // 2. Main content area (toolbar + content)
            ui.vertical(|ui| {
                // Toolbar
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

                ui.separator();

                // Content area (empty state or tabs)
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

                    // Browser content area
                    if let Some(active_idx) = self.active_tab {
                        let tab = &mut self.tabs[active_idx];

                        // Claim full available height
                        ui.set_height(ui.available_height());

                        // Render browser content
                        let response = tab.view.ui(ui, ui.available_size());

                        // Handle WebView events
                        for event in response.events {
                            if let WebViewEvent::Loaded(url) = event {
                                // Update tab's URL bar
                                tab.url_bar = url.clone();
                                // Sync to shared URL input
                                self.url_input = url;
                            }
                        }
                    }
                }
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
