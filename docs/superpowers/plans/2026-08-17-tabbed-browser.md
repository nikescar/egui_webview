# Tabbed Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform egui_webview into a Chrome/Firefox-style tabbed browser with resizable sidebar containing favorites, categorized directory, and browsing history.

**Architecture:** Component-based separation with TabManager, Sidebar, and UiState composed into TabbedBrowser. Session persistence via JSON serialization. Clean single ownership model.

**Tech Stack:** Rust 1.81+, egui 0.36, wry 0.56, serde + serde_json, eframe

## Global Constraints

- Rust edition 2021, rustfmt mandatory
- egui 0.36, wry 0.56.1 (in Cargo.toml)
- Session file: `~/.local/share/egui_webview/session.json`
- History limit: 500 entries, prune to 400 when exceeded
- Max tabs on restore: 100
- No `.unwrap()` in production code (use `?` or explicit handling)
- DRY, YAGNI, TDD - write tests first

---

## File Structure

```
src/
├── lib.rs                    # Module exports
├── tab_manager.rs            # TabManager + BrowserTab (~200 lines)
├── sidebar.rs                # Sidebar + data structures (~250 lines)
├── ui_state.rs               # UiState (~30 lines)
├── tabbed_browser.rs         # TabbedBrowser + UI (~500 lines)
└── persistence.rs            # Session save/load (~100 lines)

examples/
└── tabbed_browser.rs         # Example (~150 lines)
```

---

### Task 1: Create TabManager Foundation

**Files:**
- Create: `src/tab_manager.rs`
- Modify: `src/lib.rs` (add exports)

**Interfaces:**
- Consumes: Nothing
- Produces: `TabManager`, `BrowserTab` with methods `create_tab`, `close_tab`, `set_active`, `reorder`, `duplicate_tab`, `toggle_pin`

- [ ] Create `src/tab_manager.rs` with BrowserTab and TabManager structs (serde derive, basic fields)
- [ ] Add `create_tab(url: String) -> u64` with test
- [ ] Add `close_tab(index: usize) -> bool` with tests for middle tab and last tab
- [ ] Add `set_active(index: usize)` with test
- [ ] Add `reorder(from: usize, to: usize)` with test tracking active index
- [ ] Add `duplicate_tab(index: usize) -> u64` with test
- [ ] Add `toggle_pin(index: usize)` with test
- [ ] Run `cargo test --lib tab_manager` - all tests pass
- [ ] Update `src/lib.rs` with `mod tab_manager; pub use tab_manager::{TabManager, BrowserTab};`
- [ ] Commit: "feat: add TabManager with tab lifecycle methods"

---

### Task 2: Create Sidebar Foundation

**Files:**
- Create: `src/sidebar.rs`
- Modify: `src/lib.rs` (add exports)

**Interfaces:**
- Consumes: Nothing
- Produces: `Sidebar`, `SidebarTab` enum, `Bookmark`, `Category`, `HistoryEntry` with methods

- [ ] Create `src/sidebar.rs` with all data structures (Bookmark, Category, HistoryEntry, SidebarTab enum, Sidebar)
- [ ] Add `add_favorite(title: String, url: String)` with test
- [ ] Add `remove_favorite(index: usize)` with test
- [ ] Add `add_history(url: String, title: String)` with test (prepends to list)
- [ ] Add `prune_history()` with test (500 -> 400)
- [ ] Add `toggle_category(name: &str)` with test
- [ ] Add `add_category(category: Category)` helper
- [ ] Run `cargo test --lib sidebar` - all tests pass
- [ ] Update `src/lib.rs` exports
- [ ] Commit: "feat: add Sidebar with bookmarks, directory, and history"

---

### Task 3: Create TabbedBrowser Composition

**Files:**
- Create: `src/ui_state.rs`
- Create: `src/tabbed_browser.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `TabManager`, `Sidebar` from Tasks 1-2
- Produces: `TabbedBrowser` with `with_defaults()` constructor

- [ ] Create `src/ui_state.rs` with UiState struct (sidebar_default_width, show_new_tab_button)
- [ ] Create `src/tabbed_browser.rs` with TabbedBrowser composing all three components
- [ ] Add accessors: `tab_manager()`, `sidebar()`, `ui_state()` and `_mut` variants
- [ ] Add `with_defaults()` that creates initial tab and populates sidebar categories (Rust, egui, Examples)
- [ ] Write tests verifying default state
- [ ] Run `cargo test --lib tabbed_browser ui_state`
- [ ] Update `src/lib.rs` exports
- [ ] Commit: "feat: add TabbedBrowser composition with default data"

---

### Task 4: Add Persistence Layer

**Files:**
- Create: `src/persistence.rs`
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: `TabbedBrowser` from Task 3
- Produces: `save_session`, `load_session` functions

- [ ] Create `src/persistence.rs`
- [ ] Add `fn get_session_path() -> PathBuf` (returns `~/.local/share/egui_webview/session.json`)
- [ ] Add `fn save_session(browser: &TabbedBrowser) -> Result<(), String>` with directory creation
- [ ] Add `fn load_session() -> Result<TabbedBrowser, String>` with error handling
- [ ] Write test for round-trip serialization
- [ ] Add `TabbedBrowser::save` and `TabbedBrowser::load` convenience methods
- [ ] Run `cargo test --lib persistence`
- [ ] Update `src/lib.rs` exports
- [ ] Commit: "feat: add session persistence to JSON"

---

### Task 5: Create Example Application Structure

**Files:**
- Create: `examples/tabbed_browser.rs`

**Interfaces:**
- Consumes: `TabbedBrowser`, `init_webview`, `webview_end_frame` from lib
- Produces: Runnable example with main loop

- [ ] Create `examples/tabbed_browser.rs`
- [ ] Add GTK initialization for BSD/Linux
- [ ] Add eframe::run_ui_native with TabbedBrowser state
- [ ] Add GTK event loop processing
- [ ] Add `ctx.request_repaint()` for webview responsiveness
- [ ] Load session on startup, save on exit
- [ ] Add basic placeholder UI (CentralPanel with "Tabbed Browser - Coming Soon")
- [ ] Run `cargo run --example tabbed_browser` - window opens
- [ ] Commit: "feat: add tabbed browser example skeleton"

---

### Task 6: Implement Tab Strip UI

**Files:**
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: `TabManager` methods
- Produces: `render_tab_strip` method rendering top panel

- [ ] Add `impl TabbedBrowser` method `render_tab_strip(&mut self, ui: &mut egui::Ui)`
- [ ] Use `egui::TopBottomPanel::top` with 48px height
- [ ] Render horizontal tab list with pinned tabs first
- [ ] Add tab click to activate, close button (`×`), pinned icon (📌)
- [ ] Add "+" button for new tab
- [ ] Implement drag-and-drop for reordering (basic - use `ui.dnd_drop_zone`)
- [ ] Test by updating example to call `render_tab_strip`
- [ ] Run example, verify tabs render and can be clicked
- [ ] Commit: "feat: implement tab strip UI rendering"

---

### Task 7: Implement Sidebar UI

**Files:**
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: `Sidebar` methods
- Produces: `render_sidebar` method rendering left panel

- [ ] Add `render_sidebar(&mut self, ui: &mut egui::Ui) -> Option<String>` (returns clicked URL)
- [ ] Use `egui::SidePanel::left` with resizable (280-400px), collapsible
- [ ] Add secondary tabs for Favorites/Directory/History
- [ ] Render Favorites as vertical button list
- [ ] Render Directory as collapsible tree (categories with items)
- [ ] Render History as scrollable list with timestamps
- [ ] Return clicked URL as `Some(url)`
- [ ] Update example to call `render_sidebar` and create tab on click
- [ ] Run example, verify sidebar works
- [ ] Commit: "feat: implement sidebar UI with all three tabs"

---

### Task 8: Implement Central Panel

**Files:**
- Modify: `src/tabbed_browser.rs`
- Modify: `examples/tabbed_browser.rs`

**Interfaces:**
- Consumes: Active tab's `EguiWebView`
- Produces: `render_central_panel` showing webview or empty state

- [ ] Add `render_central_panel(&mut self, ui: &mut egui::Ui, frame: &impl HasWindowHandle)`
- [ ] Use `egui::CentralPanel::default()`
- [ ] If active tab exists: initialize webview if needed, render `tab.view.ui()`
- [ ] If no tabs: show centered "New Tab" button
- [ ] Handle webview events (update title, add history)
- [ ] Update example to call `render_central_panel`
- [ ] Run example, verify webview displays
- [ ] Commit: "feat: implement central panel with webview rendering"

---

### Task 9: Wire Up Tab-Sidebar Interactions

**Files:**
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: All previous UI components
- Produces: Complete interaction flow

- [ ] In sidebar rendering, when URL clicked: call `self.tab_manager.create_tab(url)` and `self.sidebar.add_history(url, title)`
- [ ] In webview events, when page loads: update tab title and add to history
- [ ] After adding history, call `self.sidebar.prune_history()`
- [ ] Add right-click menu on tabs for duplicate/pin operations
- [ ] Test full flow: click directory -> new tab opens -> page loads -> history updated
- [ ] Run example, verify end-to-end flow
- [ ] Commit: "feat: wire up tab and sidebar interactions"

---

### Task 10: Add Session Restoration

**Files:**
- Modify: `examples/tabbed_browser.rs`
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: `persistence` module
- Produces: Load on startup, save on exit

- [ ] In example `main()`: check if session exists, load or create default
- [ ] After loading session: recreate webviews for all tabs (call `init_webview`, create `EguiWebView` for each tab)
- [ ] Limit restored tabs to 100 (safety check)
- [ ] Add `on_exit` hook to save session
- [ ] Test: run example, open tabs, close, reopen - tabs restored
- [ ] Commit: "feat: add session restoration on startup/exit"

---

### Task 11: Add Default Keyboard Shortcuts (Basic)

**Files:**
- Modify: `src/tabbed_browser.rs`

**Interfaces:**
- Consumes: egui input events
- Produces: Keyboard shortcut handling

- [ ] Add input handling for Ctrl+T (new tab)
- [ ] Add input handling for Ctrl+W (close active tab)
- [ ] Add input handling for Ctrl+Tab / Ctrl+Shift+Tab (next/prev tab)
- [ ] Test shortcuts work
- [ ] Commit: "feat: add basic keyboard shortcuts"

---

### Task 12: Polish and Error Handling

**Files:**
- Modify: Multiple files

**Interfaces:**
- Consumes: All components
- Produces: Robust error handling

- [ ] Add error handling for webview creation failures (show error message in tab)
- [ ] Add validation for invalid URLs (show error, keep tab open)
- [ ] Handle corrupted session file (delete, use defaults)
- [ ] Add logging for important events (tab created, session saved, etc.)
- [ ] Test edge cases (close all tabs, 100+ tabs, corrupted session)
- [ ] Commit: "fix: add comprehensive error handling"

---

### Task 13: Final Integration and Documentation

**Files:**
- Modify: `README.md`
- Modify: `examples/webview.rs` (update comment)

**Interfaces:**
- Consumes: Complete implementation
- Produces: Documented, tested application

- [ ] Update README with tabbed browser features
- [ ] Add screenshot or ASCII diagram
- [ ] Document keyboard shortcuts
- [ ] Run `cargo test` - all tests pass
- [ ] Run `cargo clippy` - no warnings
- [ ] Run `cargo run --example tabbed_browser` - full manual test (50+ tabs, sidebar operations, session restore)
- [ ] Commit: "docs: update README with tabbed browser features"

---

## Self-Review Checklist

After completing all tasks:

**Spec Coverage:**
- ✅ Single main window with tab strip - Task 6
- ✅ Open, close, reorder, pin, duplicate tabs - Tasks 1, 6, 9
- ✅ Resizable/collapsible sidebar - Task 7
- ✅ Favorites, directory, history - Tasks 2, 7
- ✅ Click sidebar URLs opens new tabs - Task 9
- ✅ Session persistence - Tasks 4, 10
- ✅ Empty state when no tabs - Task 8
- ✅ Pre-populated defaults - Task 3
- ✅ Works on OpenBSD/GTK - All tasks

**Placeholder Check:**
- No TBD or TODO in code
- All error handling implemented
- All tests have expected outputs

**Type Consistency:**
- Method signatures match across tasks
- Field names consistent (e.g., `active_index` not `current_index`)

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-17-tabbed-browser.md`.

**Two execution options:**

**1. Subagent-Driven (recommended)** - Dispatch fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**

