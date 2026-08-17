# Tab Switching Fix Design

**Date:** 2026-08-17  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Fix the tab switching bug where selecting a tab updates the UI state but the webview content remains stuck on the previous tab's site.

## Problem Statement

**Bug:** When loading Tab 1 with site A and Tab 2 with site B, clicking Tab 1 selects it (UI shows it's active) but the webview still displays site B instead of site A.

**User Impact:**
- Tab selection works (UI highlights correct tab)
- URL bar updates correctly
- Webview content does NOT switch - shows wrong site
- Browser history and navigation belong to wrong tab

**Reproducibility:** 100% - occurs every time when switching between tabs with different sites loaded.

## Root Cause Analysis

**Current implementation** (lines 287-303 in `examples/tabbrowser.rs`):

```rust
// Browser content area
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];
    let response = tab.view.ui(ui, ui.available_size());
    // Handle events...
}
```

**Root cause:** Multiple `EguiWebView` instances (one per tab) are rendered in the same `CentralPanel` UI space without proper isolation. When `active_tab` changes and we call `tab.view.ui()` for a different tab, egui doesn't properly distinguish between the different webview instances because they're all rendered in the same location.

**Why it happens:**
- Each `WebBrowser` has its own `view: EguiWebView` with a unique `id: Id`
- CentralPanel provides a shared UI space for rendering
- When switching tabs, we render a different webview instance in the same space
- egui doesn't track which webview belongs to which tab without explicit isolation

## Solution Design

**Approach:** Wrap each active webview in an `egui::Area` with the tab's unique ID. This creates an isolated UI layer for each webview, allowing egui to properly track and render the correct one when tabs switch.

### Architecture

**egui::Area isolation pattern:**

```rust
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];
    
    egui::Area::new(tab.id)
        .fixed_pos([0.0, 0.0])
        .show(ui.ctx(), |ui| {
            let response = tab.view.ui(ui, ui.available_size());
            // Handle events...
        });
}
```

**Key components:**
1. **Area ID:** Uses `tab.id` (unique per tab) for egui tracking
2. **Positioning:** `.fixed_pos([0.0, 0.0])` anchors to CentralPanel top-left
3. **Size:** `ui.available_size()` maintains full height from CentralPanel
4. **Context:** `.show(ui.ctx(), ...)` - Areas require Context, not Ui

### How It Works

**egui::Area behavior:**
- Creates an isolated UI layer with a unique ID
- Only one Area with a given ID is visible at a time
- Switching tabs changes which Area is active
- Inactive Areas are hidden (not destroyed), preserving state

**State preservation:**
- Each tab's webview keeps its own browsing session
- History, cookies, and DOM state maintained per-tab
- Switching back to a tab resumes from where it left off

**Performance:**
- Only the active Area is rendered and visible
- Inactive webviews are hidden (minimal resource use)
- No duplication or unnecessary rendering

## Implementation Details

### File Changes

**File:** `examples/tabbrowser.rs`  
**Lines:** 287-303 (browser content area in CentralPanel)

**Before:**
```rust
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
```

**After:**
```rust
// Browser content area
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];

    // Wrap in Area for proper webview isolation per tab
    egui::Area::new(tab.id)
        .fixed_pos([0.0, 0.0])
        .show(ui.ctx(), |ui| {
            let response = tab.view.ui(ui, ui.available_size());

            // Handle WebView events
            for event in response.events {
                if let WebViewEvent::Loaded(url) = event {
                    tab.url_bar = url.clone();
                    self.url_input = url;
                }
            }
        });
}
```

**Changes:**
- Added: `egui::Area::new(tab.id).fixed_pos([0.0, 0.0]).show(ui.ctx(), |ui| { ... });`
- Moved: webview rendering and event handling inside Area closure
- Preserved: All existing logic, just wrapped in Area

### Edge Cases

**Tab switching:**
- Switching tabs changes which Area ID is active
- egui automatically shows the new Area and hides the old one
- No manual visibility management needed

**Tab closing:**
- When a tab is dropped, its Area is automatically cleaned up
- No memory leaks or orphaned Areas

**New tab creation:**
- New tab gets new unique ID
- New Area created when first rendered
- Independent from all other tabs

**Multiple rapid switches:**
- egui handles Area visibility changes efficiently
- No flickering or visual artifacts

## Testing Strategy

### Manual Testing Checklist

**Basic tab switching:**
- [ ] Create Tab 1, load site A (e.g., https://dure.app)
- [ ] Create Tab 2, load site B (e.g., https://www.rust-lang.org)
- [ ] Click Tab 1 → verify webview shows site A ✓
- [ ] Click Tab 2 → verify webview shows site B ✓
- [ ] Repeat switching → both tabs show correct sites ✓

**State preservation:**
- [ ] Tab 1: Navigate to site A, then site A2
- [ ] Tab 2: Navigate to site B
- [ ] Switch to Tab 1 → verify still shows site A2 (not A) ✓
- [ ] Use back button on Tab 1 → verify goes to site A ✓
- [ ] Switch to Tab 2 → verify still shows site B ✓

**Multiple tabs (3+):**
- [ ] Create 5 tabs with different sites
- [ ] Switch between all tabs randomly
- [ ] Verify each shows its correct site ✓

**Regression checks:**
- [ ] Height fix still works (web content fills window) ✓
- [ ] Sidebar toggle still works ✓
- [ ] Tab bar still works ✓
- [ ] Tab close still works ✓
- [ ] URL bar still updates when switching tabs ✓
- [ ] Navigation (back/forward/Go) still works ✓

### Success Criteria

1. ✅ Webview content matches the selected tab's site
2. ✅ No flickering or visual glitches when switching
3. ✅ Browser history preserved per-tab
4. ✅ All existing functionality still works
5. ✅ Code compiles with no warnings
6. ✅ Manual testing checklist passes

## Migration Notes

**Breaking changes:** None - this is a bug fix, not an API change

**Behavioral changes:**
- Tab switching now correctly updates webview content
- Each tab's webview properly isolated from others

**Performance:** No significant impact - egui Areas are lightweight

## References

- egui Area documentation: https://docs.rs/egui/latest/egui/containers/area/struct.Area.html
- Current implementation: `examples/tabbrowser.rs` lines 287-303
- Related fix: Tab rendering height fix (commit 69d94a5)

## Next Steps

After design approval:
1. Invoke `writing-plans` skill to create implementation plan
2. Implement Area wrapping in `examples/tabbrowser.rs`
3. Build and test
4. Run manual testing checklist
5. Commit fix with detailed message
