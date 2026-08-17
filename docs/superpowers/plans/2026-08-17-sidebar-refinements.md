# Sidebar Refinements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve toolbar ergonomics by moving sidebar toggle to toolbar and maximize browser viewport with explicit height allocation.

**Architecture:** Three localized changes to `examples/tabbrowser.rs` - add sidebar toggle to toolbar (leftmost position), remove old toggle buttons from sidebar area, and set browser content height to fill available vertical space.

**Tech Stack:** Rust (edition 2021), egui 0.36, eframe, egui_webview

## Global Constraints

- Rust edition 2021
- egui 0.36 API (no SidePanel/TopBottomPanel - use manual layout)
- Manual layout using ui.horizontal/vertical with Frame
- All changes localized to examples/tabbrowser.rs
- Preserve existing state management (sidebar_open: bool in BrowserApp)
- No new dependencies

---

## File Structure

### Files to Modify

- `examples/tabbrowser.rs` - Apply all 3 changes

**Modification points:**
1. Lines ~189-204: Toolbar section (add sidebar toggle)
2. Lines ~156-158: Remove "Hide Sidebar" button
3. Lines ~180-182: Remove "Show Sidebar" button  
4. Line ~288-292: Browser content rendering (set height)

---

### Task 1: Add Sidebar Toggle to Toolbar

**Files:**
- Modify: `examples/tabbrowser.rs:189-204`

**Interfaces:**
- Consumes: `self.sidebar_open: bool` from BrowserApp struct
- Produces: Toolbar with sidebar toggle button at leftmost position, updates `self.sidebar_open` on click

- [ ] **Step 1: Read current toolbar code**

```bash
# View current toolbar section
sed -n '189,229p' examples/tabbrowser.rs
```

Expected: See toolbar starting with back button (◀) at line 191

- [ ] **Step 2: Add sidebar toggle button before back button**

Modify `examples/tabbrowser.rs` at line 189. Change from:

```rust
ui.horizontal(|ui| {
    // Back button
    if ui.button("◀").clicked() {
```

To:

```rust
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
```

- [ ] **Step 3: Build to verify syntax**

```bash
cargo build --example tabbrowser
```

Expected: Clean build with no errors

- [ ] **Step 4: Run and test sidebar toggle in toolbar**

```bash
cargo run --example tabbrowser
```

Manual test:
- [ ] Sidebar toggle button appears leftmost in toolbar
- [ ] Button shows "◀◀" when sidebar is open
- [ ] Click "◀◀" → sidebar collapses, button changes to "▶▶"
- [ ] Click "▶▶" → sidebar expands, button changes to "◀◀"

Expected: Sidebar toggle works from toolbar

- [ ] **Step 5: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: add sidebar toggle to toolbar

- Add ◀◀/▶▶ button at leftmost position in toolbar
- Toggle sidebar_open state on click
- Visual separator between sidebar toggle and back button

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Remove Old Sidebar Toggles

**Files:**
- Modify: `examples/tabbrowser.rs:155-184`

**Interfaces:**
- Consumes: Existing sidebar layout code (lines 155-184)
- Produces: Sidebar area with no toggle buttons, only content display

- [ ] **Step 1: Read current sidebar code**

```bash
# View current sidebar section
sed -n '155,184p' examples/tabbrowser.rs
```

Expected: See "Hide Sidebar" button at line 156-158 and "Show Sidebar" button at lines 180-182

- [ ] **Step 2: Remove "Hide Sidebar" button**

Modify `examples/tabbrowser.rs` at line 155. Change from:

```rust
if self.sidebar_open {
    if ui.button("Hide Sidebar").clicked() {
        self.sidebar_open = false;
    }
    ui.vertical(|ui| {
```

To:

```rust
if self.sidebar_open {
    ui.vertical(|ui| {
```

- [ ] **Step 3: Remove "Show Sidebar" button**

Modify `examples/tabbrowser.rs` at line ~178 (after previous change, line numbers shift). Change from:

```rust
} else {
    if ui.button("Show Sidebar").clicked() {
        self.sidebar_open = true;
    }
    ui.separator();
}
```

To:

```rust
}
ui.separator();
```

Note: The separator should be outside the if block so it renders whether sidebar is open or closed.

- [ ] **Step 4: Build to verify syntax**

```bash
cargo build --example tabbrowser
```

Expected: Clean build with no errors

- [ ] **Step 5: Run and verify sidebar has no toggle button**

```bash
cargo run --example tabbrowser
```

Manual test:
- [ ] Sidebar content displays (heading, text) without toggle button
- [ ] When sidebar collapsed, no "Show Sidebar" button appears (only separator)
- [ ] Toolbar sidebar toggle (from Task 1) still works

Expected: Sidebar is clean content display only

- [ ] **Step 6: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "refactor: remove sidebar toggle buttons from sidebar area

- Delete 'Hide Sidebar' button (was inside sidebar)
- Delete 'Show Sidebar' button (was when sidebar closed)
- Sidebar now displays content only
- Toggle control moved to toolbar in previous commit

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Set Browser Height to Fill Available Space

**Files:**
- Modify: `examples/tabbrowser.rs:287-303`

**Interfaces:**
- Consumes: Browser content rendering code (lines 287-303)
- Produces: Browser content that fills 100% of available vertical space

- [ ] **Step 1: Read current browser content code**

```bash
# View current browser content section
sed -n '287,303p' examples/tabbrowser.rs
```

Expected: See browser content rendering with `tab.view.ui(ui, ui.available_size())` at line ~292

- [ ] **Step 2: Add explicit height claim before rendering**

Modify `examples/tabbrowser.rs` at line ~288. Change from:

```rust
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];

    // Render browser content
    let response = tab.view.ui(ui, ui.available_size());
```

To:

```rust
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];

    // Claim full available height
    ui.set_height(ui.available_height());

    // Render browser content
    let response = tab.view.ui(ui, ui.available_size());
```

- [ ] **Step 3: Build to verify syntax**

```bash
cargo build --example tabbrowser
```

Expected: Clean build with no errors

- [ ] **Step 4: Run and test browser height**

```bash
cargo run --example tabbrowser
```

Manual test:
- [ ] Create a tab (click "+ New Tab")
- [ ] Navigate to https://dure.app
- [ ] Browser content fills all vertical space below tab bar
- [ ] No white space gap between tab bar and browser
- [ ] Resize window → browser content adjusts height dynamically

Expected: Browser viewport maximized, fills 100% of available vertical space

- [ ] **Step 5: Regression testing**

Continue running and test:
- [ ] Tab switching still works
- [ ] URL navigation works (enter URL, click Go)
- [ ] Back/forward buttons work
- [ ] Multiple tabs scroll horizontally
- [ ] Tab closing works (× button)
- [ ] WebView events fire (URL bar updates when page loads)
- [ ] Empty state displays correctly when last tab closed

Expected: All existing functionality preserved

- [ ] **Step 6: Commit**

```bash
git add examples/tabbrowser.rs
git commit -m "feat: maximize browser viewport with explicit height

- Set ui.set_height(ui.available_height()) before rendering
- Browser content now fills 100% of vertical space
- No gaps between tab bar and browser
- Dynamic height adjustment on window resize

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

After completing all tasks, verify:

**1. Spec Coverage:**
- ✅ Sidebar toggle relocated to toolbar (leftmost position) - Task 1
- ✅ Old sidebar toggles removed - Task 2
- ✅ Browser content fills 100% available height - Task 3
- ✅ Sidebar toggle uses ◀◀ / ▶▶ icons - Task 1
- ✅ No regression in existing functionality - Task 3, Step 5

**2. Placeholder Scan:**
- ✅ No TBD, TODO, or "fill in details"
- ✅ All code blocks complete
- ✅ All test steps specific
- ✅ All file paths exact

**3. Type Consistency:**
- ✅ `sidebar_open: bool` used consistently across all tasks
- ✅ Button labels ("◀◀" / "▶▶") consistent
- ✅ egui API calls match egui 0.36 (ui.set_height, ui.available_height)

**4. Build Verification:**
```bash
cargo build --example tabbrowser
cargo run --example tabbrowser
```

Expected: Clean build, all 3 changes visible in running application.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-17-sidebar-refinements.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
