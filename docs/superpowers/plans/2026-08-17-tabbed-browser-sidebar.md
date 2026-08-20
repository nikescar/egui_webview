# Tabbed Browser with Sidebar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor tabbrowser.rs from multiple browser windows to single main window with tabs and sidebar panel.

**Architecture:** Add BrowserApp struct that wraps multiple WebBrowser instances as tabs, with collapsible sidebar, shared toolbar, and tab bar. Minimal changes to WebBrowser (preserve existing functionality).

**Tech Stack:** Rust, egui, wry, egui_webview

## Global Constraints

- Rust edition 2021
- Default new tab URL: `https://dure.app`
- Sidebar size range: 280.0..=400.0 pixels
- Start with empty state (no tabs)
- Maintain continuous repaint for responsive webview input
- Tab bar must support horizontal scrolling
- Active tab invariant: `active_tab == Some(i)` ⟹ `i < tabs.len()`
- Empty state invariant: `tabs.is_empty()` ⟹ `active_tab == None`

---

## File Structure

### Files to Modify
- `examples/tabbrowser.rs` - Complete refactor with BrowserApp struct, preserve WebBrowser

**Decomposition:**
- `WebBrowser` struct: Minimal changes (keep existing fields/methods, add content-only rendering)
- `BrowserApp` struct: New wrapper for tabs, sidebar, toolbar
- `main()` function: Replace eframe::run_ui_native loop to use BrowserApp

---

### Task 1: Create BrowserApp struct with empty state

**Files:**
- Modify: `examples/tabbrowser.rs` (add BrowserApp struct before main())

**Interfaces:**
- Consumes: None (initial task)
- Produces: `BrowserApp::new(ctx: &Context) -> Self`, `BrowserApp::ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`

- [ ] **Step 1: Add BrowserApp struct definition**

Add after `WebBrowser` impl blocks, before `main()`:

```rust
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
    
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("No tabs open");
                ui.label("Click '+' to create a new tab.");
            });
        });
    }
}
```

Expected: Code compiles (BrowserApp not used yet)

- [ ] **Step 2: Replace main() to use BrowserApp**

Replace the existing `main()` function body (starting from `let mut windows = vec![];`):

```rust
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
```

Expected: Code compiles

- [ ] **Step 3: Build and run to verify empty state**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 4: Run and verify empty state displays**

```bash
cargo run --example tabbrowser
```

Expected: 
- Window opens showing "No tabs open" heading
- "Click '+' to create a new tab." label displayed
- No errors in terminal

- [ ] **Step 5: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: add BrowserApp struct with empty state

- Create BrowserApp wrapper for tab management
- Replace main() to use BrowserApp instead of Vec<WebBrowser>
- Show empty state UI when no tabs open
- Initialize webview system in BrowserApp::new()

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Add sidebar panel

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp::ui()` method

**Interfaces:**
- Consumes: `BrowserApp::ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`
- Produces: Sidebar panel rendered in UI

- [ ] **Step 1: Update BrowserApp::ui() to add sidebar panel**

Replace the `BrowserApp::ui()` method:

```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::{SidePanel, TopBottomPanel, CentralPanel};
    
    // 1. Left Sidebar Panel (collapsible, resizable)
    SidePanel::left("browser_sidebar")
        .resizable(true)
        .default_width(280.0)
        .width_range(280.0..=400.0)
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
    
    // 2. Central Panel (empty state for now)
    CentralPanel::default().show(ui, |ui| {
        ui.centered_and_justified(|ui| {
            ui.heading("No tabs open");
            ui.label("Click '+' to create a new tab.");
        });
    });
}
```

Expected: Code compiles

- [ ] **Step 2: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Run and verify sidebar displays**

```bash
cargo run --example tabbrowser
```

Expected:
- Left sidebar visible with "💻 Sidebar" heading
- Demo text displayed in sidebar
- Sidebar can be collapsed by clicking arrow icon
- Sidebar can be resized by dragging right edge
- Sidebar respects 280-400px width range

- [ ] **Step 4: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: add collapsible resizable sidebar panel

- Add SidePanel::left() with 280-400px width range
- Sidebar collapsible with toggle button
- Display demo text in sidebar
- Sidebar open by default

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Add toolbar with URL bar and navigation buttons

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp::ui()` method

**Interfaces:**
- Consumes: `BrowserApp::ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`, `self.url_input: String`
- Produces: Toolbar panel with back/forward/URL/Go buttons (non-functional yet)

- [ ] **Step 1: Add toolbar panel between sidebar and central panel**

Update `BrowserApp::ui()` method to add toolbar after sidebar:

```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::{SidePanel, TopBottomPanel, CentralPanel};
    
    // 1. Left Sidebar Panel (collapsible, resizable)
    SidePanel::left("browser_sidebar")
        .resizable(true)
        .default_width(280.0)
        .width_range(280.0..=400.0)
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
    
    // 2. Top Toolbar Panel (URL bar + nav buttons)
    TopBottomPanel::top("browser_toolbar")
        .show(ui, |ui| {
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
        });
    
    // 3. Central Panel (empty state for now)
    CentralPanel::default().show(ui, |ui| {
        ui.centered_and_justified(|ui| {
            ui.heading("No tabs open");
            ui.label("Click '+' to create a new tab.");
        });
    });
}
```

Expected: Code compiles

- [ ] **Step 2: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Run and verify toolbar displays**

```bash
cargo run --example tabbrowser
```

Expected:
- Toolbar visible at top with back/forward buttons
- URL text input field displayed
- "Go" button visible
- Buttons don't do anything yet (expected)
- URL field is editable

- [ ] **Step 4: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: add toolbar with URL bar and navigation buttons

- Add TopBottomPanel::top() for toolbar
- Add back/forward buttons (◀ ▶)
- Add URL text input field
- Add Go button
- Navigation logic placeholder (to be implemented)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Implement tab creation (+ button)

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp` (add add_tab method, update ui() to add + button)

**Interfaces:**
- Consumes: `BrowserApp::ui()`, `WebBrowser::new(ctx: &Context, id: Id, url: &str, window: &impl HasWindowHandle) -> Self`
- Produces: `BrowserApp::add_tab(&mut self, ctx: &Context, frame: &impl HasWindowHandle, url: &str)`

- [ ] **Step 1: Add add_tab() method to BrowserApp**

Add this method to `impl BrowserApp`:

```rust
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
```

Expected: Code compiles

- [ ] **Step 2: Add + button in empty state**

Update the CentralPanel section in `BrowserApp::ui()`:

```rust
// 3. Central Panel (empty state or tabs)
CentralPanel::default().show(ui, |ui| {
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
        // Tabs will be rendered here later
        ui.label("Tabs rendering coming soon...");
    }
});
```

Expected: Code compiles

- [ ] **Step 3: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 4: Run and test tab creation**

```bash
cargo run --example tabbrowser
```

Manual test:
1. Click "+ New Tab" button
2. Verify: "Tabs rendering coming soon..." appears (empty state gone)
3. Verify: URL bar shows "https://dure.app"

Expected: Tab created successfully, UI switches from empty state

- [ ] **Step 5: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: implement tab creation with + button

- Add add_tab() method to create new WebBrowser tab
- Add '+ New Tab' button in empty state
- Set active_tab when creating tab
- Initialize next_tab_id counter
- Sync url_input with new tab URL
- Default new tab URL: https://dure.app

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Implement tab bar UI with tab switching

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp::ui()` CentralPanel section

**Interfaces:**
- Consumes: `BrowserApp::ui()`, `self.tabs: Vec<WebBrowser>`, `self.active_tab: Option<usize>`
- Produces: Tab bar UI with selectable tabs and + button

- [ ] **Step 1: Replace "Tabs rendering coming soon" with tab bar**

Update the CentralPanel section in `BrowserApp::ui()`:

```rust
// 3. Central Panel (Tab bar + content)
CentralPanel::default().show(ui, |ui| {
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
            .id_source("tab_scroll_area")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
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
                            });
                        });
                    }
                    
                    // + button to add new tab
                    if ui.button("+").clicked() {
                        self.add_tab(ui.ctx(), frame, "https://dure.app");
                    }
                });
            });
        
        ui.separator();
        
        // Browser content area (placeholder for now)
        if let Some(active_idx) = self.active_tab {
            ui.label(format!("Active tab: {} (browser content coming soon)", active_idx + 1));
        }
    }
});
```

Expected: Code compiles

- [ ] **Step 2: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Run and test tab bar**

```bash
cargo run --example tabbrowser
```

Manual test:
1. Click "+ New Tab" - tab "Tab 1" appears
2. Click "+" button in tab bar - "Tab 2" appears
3. Click "+" again - "Tab 3" appears
4. Click "Tab 1" - verify it becomes selected (highlighted)
5. Click "Tab 2" - verify it becomes selected
6. Verify: URL bar updates when switching tabs
7. Create 10+ tabs - verify horizontal scrolling works

Expected: Tab bar displays, tab switching works, URL syncs

- [ ] **Step 4: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: implement tab bar UI with tab switching

- Add ScrollArea::horizontal() for scrollable tab bar
- Render tabs with selectable_label
- Implement tab switching (update active_tab)
- Sync url_input when switching tabs
- Add + button in tab bar
- Show placeholder for browser content

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Implement tab closing (X button)

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp` (add close_tab method, update ui() to add X button)

**Interfaces:**
- Consumes: `BrowserApp::ui()`, `self.tabs: Vec<WebBrowser>`, `self.active_tab: Option<usize>`
- Produces: `BrowserApp::close_tab(&mut self, idx: usize)`

- [ ] **Step 1: Add close_tab() method to BrowserApp**

Add this method to `impl BrowserApp`:

```rust
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
```

Expected: Code compiles

- [ ] **Step 2: Add X button to each tab**

Update the tab bar section in `BrowserApp::ui()` CentralPanel:

```rust
// Tab bar with horizontal scrolling
egui::ScrollArea::horizontal()
    .id_source("tab_scroll_area")
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
```

Expected: Code compiles

- [ ] **Step 3: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 4: Run and test tab closing**

```bash
cargo run --example tabbrowser
```

Manual test:
1. Create 3 tabs (Tab 1, Tab 2, Tab 3)
2. Click × on Tab 2
3. Verify: Tab 2 removed, Tab 1 and Tab 3 remain
4. Click × on Tab 1
5. Verify: Only Tab 3 remains (now Tab 1)
6. Click × on last tab
7. Verify: Empty state appears ("No tabs open")

Expected: Tab closing works, empty state appears when last tab closed

- [ ] **Step 5: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: implement tab closing with X button

- Add close_tab() method with index adjustment logic
- Add small × button to each tab
- Handle closing last tab → empty state
- Adjust active_tab when closing tabs
- Sync url_input after closing tab
- Avoid borrow conflict with tab_to_close deferred removal

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Implement URL navigation

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp::ui()` toolbar section

**Interfaces:**
- Consumes: `BrowserApp::ui()`, `self.active_tab: Option<usize>`, `self.tabs[idx].view: EguiWebView`
- Produces: Functional back/forward/URL navigation

- [ ] **Step 1: Implement back button navigation**

Update the back button in toolbar section of `BrowserApp::ui()`:

```rust
// Back button
if ui.button("◀").clicked() {
    if let Some(idx) = self.active_tab {
        self.tabs[idx].view.back();
    }
}
```

Expected: Code compiles

- [ ] **Step 2: Implement forward button navigation**

Update the forward button in toolbar section:

```rust
// Forward button
if ui.button("▶").clicked() {
    if let Some(idx) = self.active_tab {
        self.tabs[idx].view.forward();
    }
}
```

Expected: Code compiles

- [ ] **Step 3: Implement URL bar Enter key navigation**

Update the URL input response handling:

```rust
let response = ui.add(
    TextEdit::singleline(&mut self.url_input)
        .desired_width(ui.available_width() - 60.0)
);

if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
    if let Some(idx) = self.active_tab {
        if let Err(e) = self.tabs[idx].view.load_url(&self.url_input) {
            eprintln!("Failed to load URL: {}", e);
        }
    }
}
```

Expected: Code compiles

- [ ] **Step 4: Implement Go button navigation**

Update the Go button:

```rust
// Go button
if ui.button("Go").clicked() {
    if let Some(idx) = self.active_tab {
        if let Err(e) = self.tabs[idx].view.load_url(&self.url_input) {
            eprintln!("Failed to load URL: {}", e);
        }
    }
}
```

Expected: Code compiles

- [ ] **Step 5: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 6: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: implement URL navigation controls

- Connect back button to active tab's webview.back()
- Connect forward button to active tab's webview.forward()
- Implement URL bar Enter key navigation
- Implement Go button navigation
- Add error logging for load_url failures

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Render active browser content and handle WebView events

**Files:**
- Modify: `examples/tabbrowser.rs:BrowserApp::ui()` CentralPanel browser content area

**Interfaces:**
- Consumes: `BrowserApp::ui()`, `self.tabs[active_idx]: WebBrowser`, `WebBrowser.view.ui() -> EguiWebViewResponse`
- Produces: Browser content rendering and WebViewEvent::Loaded handling

- [ ] **Step 1: Replace browser content placeholder with actual rendering**

Update the browser content area in CentralPanel section of `BrowserApp::ui()`:

```rust
ui.separator();

// Browser content area
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];
    
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
```

Expected: Code compiles

- [ ] **Step 2: Build**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Run and test browser rendering**

```bash
cargo run --example tabbrowser
```

Manual test:
1. Create new tab (should load https://dure.app)
2. Verify: Browser content displays
3. Verify: URL bar shows "https://dure.app"
4. Type new URL in URL bar (e.g., "https://www.rust-lang.org")
5. Press Enter
6. Verify: Browser navigates to new URL
7. Verify: URL bar updates when page loads
8. Click back button
9. Verify: Browser goes back to https://dure.app
10. Click forward button
11. Verify: Browser goes forward
12. Create second tab
13. Switch between tabs
14. Verify: Each tab maintains its own content

Expected: Browser renders, navigation works, URL syncs

- [ ] **Step 4: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: render active browser content and handle WebView events

- Render active tab's webview content in CentralPanel
- Handle WebViewEvent::Loaded to sync URL bar
- Update tab.url_bar when page loads
- Update shared url_input when page loads
- Each tab maintains independent browser state

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Final integration, cleanup, and manual testing

**Files:**
- Modify: `examples/tabbrowser.rs` (remove unused code, verify all features)

**Interfaces:**
- Consumes: Complete BrowserApp implementation
- Produces: Fully functional tabbed browser with sidebar

- [ ] **Step 1: Remove or comment out old WebBrowser window rendering**

Verify that the old `WebBrowser::ui()` method signature is preserved (we're not using it in window mode, but keeping it for compatibility):

No code changes needed - WebBrowser struct is used directly via its `view` field.

Expected: No changes needed

- [ ] **Step 2: Verify window title**

Confirm main() has correct window title:

```rust
eframe::run_ui_native(
    "Tabbed Browser",  // Window title
    NativeOptions::default(),
    move |ui, frame| {
```

Expected: Title is "Tabbed Browser"

- [ ] **Step 3: Build release version**

```bash
cargo build --release --example tabbrowser
```

Expected: Build succeeds with no warnings

- [ ] **Step 4: Run full manual test checklist**

```bash
cargo run --release --example tabbrowser
```

**Tab Management:**
- [ ] Click "+ New Tab" creates new tab with https://dure.app
- [ ] New tab becomes active automatically
- [ ] Click tab switches to that tab
- [ ] Click "×" closes tab
- [ ] Closing last tab shows empty state
- [ ] Empty state displays message and "+ New Tab" button

**Navigation:**
- [ ] Back button works on active tab
- [ ] Forward button works on active tab
- [ ] URL bar shows active tab's URL
- [ ] Typing URL + Enter navigates active tab
- [ ] Click "Go" button navigates active tab
- [ ] URL bar updates when page loads

**Sidebar:**
- [ ] Sidebar visible by default
- [ ] Collapse arrow hides sidebar
- [ ] Expand arrow shows sidebar
- [ ] Sidebar can be resized by dragging right edge
- [ ] Sidebar respects 280-400px width range
- [ ] Sidebar displays demo text

**Edge Cases:**
- [ ] Create 10+ tabs - horizontal scrolling works
- [ ] Close middle tab - active tab preserved correctly
- [ ] Close active tab - switches to adjacent tab
- [ ] Type in webview textboxes - responsive input (no lag)

**Performance:**
- [ ] Create 10+ tabs - UI remains responsive
- [ ] Switch between tabs quickly - no lag
- [ ] Resize sidebar - smooth animation

Expected: All tests pass

- [ ] **Step 5: Clean up debug prints if any**

Check for any println! statements that should be removed:

```bash
grep -n "println!" examples/tabbrowser.rs
```

The only println! should be in `WebBrowser::Drop` trait (keep for debugging):
```rust
impl Drop for WebBrowser {
    fn drop(&mut self) {
        println!("🗑️  Dropping WebBrowser {:?}", self.id);
    }
}
```

Expected: Only Drop trait println! remains

- [ ] **Step 6: Final commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: complete tabbed browser with sidebar implementation

Final integration of all features:
- Tabbed interface with horizontal scrolling
- Collapsible/resizable sidebar (280-400px)
- Shared toolbar with URL bar and navigation
- Empty state when no tabs open
- Tab creation (+) and closing (×)
- URL navigation (back/forward/Go/Enter)
- WebView event handling for URL sync
- Default new tab URL: https://dure.app

Manual testing complete:
✓ Tab management (create, switch, close)
✓ Navigation (back, forward, URL bar)
✓ Sidebar (collapse, resize, demo text)
✓ Edge cases (multiple tabs, scrolling)
✓ Performance (10+ tabs, responsive input)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

After completing all tasks, verify:

**1. Spec Coverage:**
- ✅ Tabs with horizontal scrolling (Task 5)
- ✅ "+" button to add tabs (Task 4, 5)
- ✅ "×" button to close tabs (Task 6)
- ✅ Empty state when no tabs (Task 1, 6)
- ✅ Start with no tabs (Task 1)
- ✅ Default URL https://dure.app (Task 4)
- ✅ Shared URL bar in toolbar (Task 3, 7)
- ✅ Back/forward buttons (Task 3, 7)
- ✅ Sidebar: left, collapsible, resizable 280-400px (Task 2)
- ✅ Demo text in sidebar (Task 2)
- ✅ Continuous repaint for responsive webview (Task 1 - main())
- ✅ Minimal WebBrowser refactoring (used directly via view field)
- ✅ Clean state management (Task 6 - close_tab logic)

**2. Placeholder Scan:**
- ✅ No TBD, TODO, or "implement later"
- ✅ All code blocks complete
- ✅ All navigation logic implemented
- ✅ All event handling implemented

**3. Type Consistency:**
- ✅ `BrowserApp::new(ctx: &Context)` used consistently
- ✅ `BrowserApp::add_tab(&mut self, ctx: &Context, frame: &impl HasWindowHandle, url: &str)` signature consistent
- ✅ `BrowserApp::close_tab(&mut self, idx: usize)` signature consistent
- ✅ `self.tabs: Vec<WebBrowser>` type consistent
- ✅ `self.active_tab: Option<usize>` type consistent
- ✅ `self.url_input: String` type consistent

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-17-tabbed-browser-sidebar.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
