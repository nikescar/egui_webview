# Tab Rendering Height Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix web content height capped at ~150px by replacing manual layout with egui Panel system for automatic height allocation.

**Architecture:** Refactor `BrowserApp::ui()` method to use three-panel hierarchy: SidePanel::left() for sidebar, TopBottomPanel::top() for toolbar, CentralPanel::default() for tabs and browser content. CentralPanel automatically claims remaining vertical space, fixing the height issue.

**Tech Stack:** Rust, egui, wry, egui_webview

## Global Constraints

- Rust edition 2021
- Single file modification: `examples/tabbrowser.rs`
- No changes to BrowserApp struct fields or methods (add_tab, close_tab)
- Sidebar width range: 80.0..=200.0 pixels
- All existing functionality must be preserved
- Code must compile with no warnings
- Manual visual testing required (no automated tests for UI layout)

---

## File Structure

### Files to Modify
- `examples/tabbrowser.rs:155-318` - Refactor `BrowserApp::ui()` method from manual layout to Panel system

**Decomposition:**
- Lines 155-180: Replace manual sidebar with `SidePanel::left()`
- Lines 182-237: Extract toolbar to `TopBottomPanel::top()`
- Lines 242-314: Wrap browser content in `CentralPanel::default()`
- Line 300: Remove `ui.set_height(ui.available_height())` (no longer needed)

---

### Task 1: Add egui Panel imports and replace manual layout with SidePanel

**Files:**
- Modify: `examples/tabbrowser.rs:2-6` (add imports)
- Modify: `examples/tabbrowser.rs:155-180` (replace sidebar)

**Interfaces:**
- Consumes: `self.sidebar_open: bool`, existing sidebar content
- Produces: `SidePanel::left()` with `.show_animated()` for smooth collapse

- [ ] **Step 1: Add Panel imports**

Add to imports at top of file (after line 3):

```rust
use egui::{Context, Id, Layout, Popup, TextEdit, Widget, Window, SidePanel, TopBottomPanel, CentralPanel};
```

Expected: Code compiles (just adding imports)

- [ ] **Step 2: Build to verify imports**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Replace BrowserApp::ui() method signature and start with SidePanel**

Replace the entire `pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)` method starting at line 155.

**Remove this:**
```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    ui.horizontal(|ui| {
        // 1. Left Sidebar (manual layout)
        if self.sidebar_open {
            ui.vertical(|ui| {
                ui.set_width(100.0);
                egui::Frame::new()
                    .fill(ui.style().visuals.faint_bg_color)
                    .show(ui, |ui| {
                        ui.set_min_width(80.0);
                        ui.set_max_width(200.0);

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
```

**Replace with:**
```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    // 1. Left Sidebar Panel (collapsible, resizable)
    SidePanel::left("browser_sidebar")
        .resizable(true)
        .default_width(100.0)
        .width_range(80.0..=200.0)
        .show_animated(ui.ctx(), self.sidebar_open, |ui| {
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
```

Expected: Code compiles (temporary placeholder)

- [ ] **Step 4: Build to verify SidePanel compiles**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 5: Commit SidePanel**

```bash
git add examples/tabbrowser.rs
git commit -m "$(cat <<'EOF'
refactor: replace manual sidebar with SidePanel::left()

- Add egui Panel imports (SidePanel, TopBottomPanel, CentralPanel)
- Replace manual ui.horizontal > ui.vertical sidebar with SidePanel::left()
- Use .show_animated() for smooth collapse animation
- Set .width_range(80.0..=200.0) for resizable sidebar
- Temporary placeholder for toolbar and content (next task)

Part 1 of panel-based layout refactor to fix height allocation.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created successfully

---

### Task 2: Add TopBottomPanel for toolbar

**Files:**
- Modify: `examples/tabbrowser.rs:155-318` (add toolbar panel)

**Interfaces:**
- Consumes: `SidePanel::left()` from Task 1, existing toolbar content from lines 184-237
- Produces: `TopBottomPanel::top()` with sidebar toggle, back/forward, URL bar, Go button

- [ ] **Step 1: Replace placeholder with TopBottomPanel for toolbar**

Replace the placeholder line:
```rust
    // Placeholder for toolbar and content (to be replaced in next task)
    ui.label("TODO: Add toolbar and content panels");
```

With the complete TopBottomPanel:
```rust
    // 2. Top Toolbar Panel (navigation controls)
    TopBottomPanel::top("browser_toolbar")
        .show(ui.ctx(), |ui| {
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

    // Placeholder for content panel (to be replaced in next task)
    ui.label("TODO: Add CentralPanel");
```

Expected: Code compiles

- [ ] **Step 2: Build to verify TopBottomPanel compiles**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds

- [ ] **Step 3: Commit TopBottomPanel**

```bash
git add examples/tabbrowser.rs
git commit -m "$(cat <<'EOF'
refactor: add TopBottomPanel for toolbar

- Extract toolbar from nested ui.vertical to TopBottomPanel::top()
- Toolbar content unchanged: sidebar toggle, back/forward, URL bar, Go
- All button click handlers preserved
- Temporary placeholder for content panel (next task)

Part 2 of panel-based layout refactor to fix height allocation.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created successfully

---

### Task 3: Add CentralPanel for browser content (CRITICAL FIX)

**Files:**
- Modify: `examples/tabbrowser.rs:155-318` (add content panel)

**Interfaces:**
- Consumes: `SidePanel::left()` from Task 1, `TopBottomPanel::top()` from Task 2, existing tab/browser content
- Produces: `CentralPanel::default()` with tabs and webview, **automatic height allocation** (fixes the 150px bug)

- [ ] **Step 1: Replace placeholder with CentralPanel**

Replace the placeholder line:
```rust
    // Placeholder for content panel (to be replaced in next task)
    ui.label("TODO: Add CentralPanel");
```

With the complete CentralPanel:
```rust
    // 3. Central Panel (tabs + browser content)
    CentralPanel::default().show(ui.ctx(), |ui| {
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

                // ✅ CRITICAL FIX: ui.set_height() removed!
                // CentralPanel automatically provides full remaining height
                // ui.available_size() now returns correct height (not 150px)
                let response = tab.view.ui(ui, ui.available_size());

                // Handle WebView events
                for event in response.events {
                    if let WebViewEvent::Loaded(url) = event {
                        tab.url_bar = url.clone();
                        self.url_input = url;
                    }
                }
            }
        }
    });
```

**CRITICAL:** Notice the removed line - no more `ui.set_height(ui.available_height())`! CentralPanel handles this automatically.

Expected: Code compiles

- [ ] **Step 2: Build to verify CentralPanel compiles**

```bash
cargo build --example tabbrowser
```

Expected: Build succeeds with no warnings

- [ ] **Step 3: Commit CentralPanel (the fix!)**

```bash
git add examples/tabbrowser.rs
git commit -m "$(cat <<'EOF'
fix: add CentralPanel to auto-fill browser content height

- Wrap tab bar and browser content in CentralPanel::default()
- CentralPanel automatically claims remaining vertical space
- CRITICAL FIX: Remove ui.set_height(ui.available_height()) line
- ui.available_size() now returns correct height (not 150px!)
- Tab bar and WebView event handling unchanged

Part 3 of panel-based layout refactor - this fixes the height bug.

Fixes: Web content capped at 150px with black area below
Solution: CentralPanel auto-fills height after SidePanel and TopBottomPanel

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created successfully

---

### Task 4: Manual testing and verification

**Files:**
- Test: `examples/tabbrowser.rs` (run and verify all features)

**Interfaces:**
- Consumes: Complete panel-based layout from Tasks 1-3
- Produces: Verified working browser with correct height allocation

- [ ] **Step 1: Run the example**

```bash
cargo run --example tabbrowser
```

Expected: Window opens with sidebar visible

- [ ] **Step 2: Test height allocation**

Manual test:
1. Click "+ New Tab" button
2. Observe web content height
3. Verify: Web content fills window height (NOT 150px)
4. Verify: No black area below web content
5. Verify: Tab controls fully visible at all times
6. Resize window vertically
7. Verify: Web content scales with window

Expected: Web content fills full window height dynamically

- [ ] **Step 3: Test sidebar**

Manual test:
1. Verify sidebar is visible by default
2. Click collapse button (◀◀)
3. Verify: Sidebar hides with smooth animation
4. Verify: Web content expands to use freed space
5. Click expand button (▶▶)
6. Verify: Sidebar shows with smooth animation
7. Drag sidebar right edge to resize
8. Verify: Sidebar resizes smoothly
9. Try to resize below 80px or above 200px
10. Verify: Sidebar respects width limits

Expected: Sidebar collapse/expand/resize works smoothly

- [ ] **Step 4: Test toolbar**

Manual test:
1. Verify toolbar visible at top
2. Click back button (◀) with no history
3. Verify: Nothing happens (expected - no history)
4. Type "https://www.rust-lang.org" in URL bar
5. Press Enter
6. Wait for page to load
7. Verify: Page loads in browser content
8. Verify: URL bar updates to "https://www.rust-lang.org"
9. Click back button
10. Verify: Browser navigates back to https://dure.app
11. Click forward button (▶)
12. Verify: Browser goes forward to rust-lang.org
13. Type new URL and click Go button
14. Verify: Navigation works

Expected: All toolbar controls functional

- [ ] **Step 5: Test tab management**

Manual test:
1. Click "+" button in tab bar
2. Verify: New tab created (Tab 2)
3. Verify: New tab becomes active
4. Click "+" again
5. Verify: Tab 3 created
6. Click "Tab 1" button
7. Verify: Tab 1 becomes active
8. Verify: URL bar shows Tab 1's URL
9. Click "Tab 2" button
10. Verify: Tab 2 becomes active
11. Create 10 more tabs (13 total)
12. Verify: Tab bar scrolls horizontally
13. Click "×" on Tab 7
14. Verify: Tab 7 closes, tab count reduced
15. Close all tabs one by one
16. Verify: Empty state appears when last tab closed

Expected: Tab creation, switching, closing all work correctly

- [ ] **Step 6: Test edge cases**

Manual test:
1. Create new tab
2. Navigate to a very tall webpage (e.g., https://news.ycombinator.com)
3. Verify: Vertical scrolling works within webview
4. Navigate to a very wide webpage
5. Verify: Horizontal scrolling works within webview
6. Rapidly switch between tabs (click tabs quickly)
7. Verify: No flickering or layout glitches
8. Rapidly toggle sidebar (click ◀◀ and ▶▶ repeatedly)
9. Verify: Smooth animation, no visual artifacts

Expected: All edge cases handled gracefully

- [ ] **Step 7: Visual regression verification**

Compare before and after:

**Before fix:**
- Web content: ~150px height
- Black area: Rest of window below web content
- Tab controls: Partially hidden

**After fix:**
- Web content: Full window height (minus toolbar)
- Black area: None
- Tab controls: Fully visible

Expected: All "After fix" criteria met

- [ ] **Step 8: Final commit (if manual testing passes)**

```bash
git add examples/tabbrowser.rs
git commit -m "$(cat <<'EOF'
docs: verify panel-based layout fix with manual testing

Manual testing checklist completed:
✅ Web content fills full window height (not 150px)
✅ No black area below web content
✅ Tab controls fully visible
✅ Sidebar collapse/expand with smooth animation
✅ Sidebar resizing works (80-200px range)
✅ Toolbar controls functional (back/forward/URL/Go)
✅ Tab management works (create/switch/close)
✅ Tab bar horizontal scrolling works
✅ Window resizing updates web content height
✅ Edge cases handled (tall/wide pages, rapid actions)

Visual regression test: PASS
- Before: 150px web content, black area below
- After: Full height web content, no black area

All success criteria met.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Final commit created

---

## Self-Review Checklist

After completing all tasks, verify:

**1. Spec Coverage:**
- ✅ Replace manual layout with Panels (Tasks 1-3)
- ✅ SidePanel::left() for sidebar (Task 1)
- ✅ TopBottomPanel::top() for toolbar (Task 2)
- ✅ CentralPanel::default() for browser content (Task 3)
- ✅ Remove ui.set_height() line (Task 3, Step 1)
- ✅ Smooth sidebar animation with .show_animated() (Task 1)
- ✅ Sidebar width range 80-200px (Task 1)
- ✅ All existing functionality preserved (Task 4)
- ✅ Manual testing checklist (Task 4)

**2. Placeholder Scan:**
- ✅ No TBD or TODO in final code (placeholders only in intermediate commits)
- ✅ All code blocks complete
- ✅ All expected outputs specified
- ✅ All test steps have verification criteria

**3. Type Consistency:**
- ✅ `SidePanel::left("browser_sidebar")` ID consistent
- ✅ `TopBottomPanel::top("browser_toolbar")` ID consistent
- ✅ `CentralPanel::default()` no ID needed
- ✅ `self.sidebar_open: bool` used consistently
- ✅ `self.active_tab: Option<usize>` used consistently
- ✅ `self.tabs: Vec<WebBrowser>` used consistently

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-17-tab-rendering-fix.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?