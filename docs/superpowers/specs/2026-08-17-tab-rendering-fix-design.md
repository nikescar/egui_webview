# Tab Rendering Height Fix Design

**Date:** 2026-08-17  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Fix the tab rendering issue where web content height is capped at ~150px with the rest of the screen appearing black. The root cause is manual layout not properly allocating vertical space. The solution is to replace manual `ui.horizontal/vertical` layout with egui's Panel system for automatic height management.

## Problem Statement

When opening a tab and loading a webpage in the tabbed browser:
- Tab controls are not visible or partially visible
- Web content height is only ~150px (should fill window)
- Rest of screen below web content is black
- Issue occurs with all websites (not content-specific)

Previous fix attempt added `ui.set_height(ui.available_height())` at line 300, but this happens too late in the layout process after parent layout has already computed sizes.

## Root Cause Analysis

**Current implementation** (lines 155-318 in `examples/tabbrowser.rs`):

```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    ui.horizontal(|ui| {
        // Sidebar: manual ui.vertical with ui.set_width(100.0)
        // Main content: ui.vertical containing toolbar, tabs, browser
    });
}
```

**Problem:** Manual layout using `ui.horizontal` → `ui.vertical` does not automatically expand to fill window. When nested layouts are used:
1. Parent `ui.horizontal` doesn't claim full window height
2. Child `ui.vertical` for main content gets constrained height
3. By the time `ui.set_height(ui.available_height())` is called (line 300), the parent layout has already computed ~150px as "available"
4. Webview renders at 150px instead of full window height

**Why 150px specifically:** This is likely the height of toolbar + tab bar + some padding/margins. The remaining space is never allocated.

## Solution Design

Replace manual layout with egui's Panel system:

1. **SidePanel::left()** - Collapsible sidebar (replaces manual `ui.vertical`)
2. **TopBottomPanel::top()** - Toolbar with navigation controls
3. **CentralPanel::default()** - Tab bar + browser content (auto-fills remaining space)

**Key insight:** Panels calculate layout automatically in order. CentralPanel claims all remaining vertical space after SidePanel and TopBottomPanel, ensuring webview gets full height allocation.

## Architecture

### Panel Hierarchy

Panels render in order (this order matters):

```rust
pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    // 1. Left Sidebar Panel (claims left edge first)
    SidePanel::left("browser_sidebar")
        .resizable(true)
        .default_width(100.0)
        .width_range(80.0..=200.0)
        .show_animated(ui.ctx(), self.sidebar_open, |ui| {
            // Sidebar content
        });
    
    // 2. Top Toolbar Panel (claims top edge in remaining space)
    TopBottomPanel::top("browser_toolbar")
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                // Sidebar toggle, back/forward, URL bar, Go button
            });
        });
    
    // 3. Central Panel (gets everything left over - auto-fills height ✅)
    CentralPanel::default().show(ui.ctx(), |ui| {
        if self.tabs.is_empty() {
            // Empty state
        } else {
            // Tab bar (ScrollArea::horizontal)
            // Browser content (webview fills remaining height automatically)
        }
    });
}
```

### Layout Calculation

**Panel order determines space allocation:**

1. SidePanel claims left edge (80-200px wide, full height)
2. TopBottomPanel claims top edge in remaining horizontal space (auto-height based on content)
3. CentralPanel gets all remaining space (auto-fills both width and height)

**Result:** Webview in CentralPanel gets actual window height minus toolbar height, not just 150px.

## Implementation Details

### 1. Sidebar Panel

**Before (manual layout):**
```rust
if self.sidebar_open {
    ui.vertical(|ui| {
        ui.set_width(100.0);
        egui::Frame::new()
            .fill(ui.style().visuals.faint_bg_color)
            .show(ui, |ui| {
                // ... sidebar content
            });
    });
}
ui.separator();
```

**After (SidePanel):**
```rust
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
```

**Changes:**
- `.show_animated()` handles smooth slide animation (replaces manual `if` check)
- `.resizable(true)` enables drag-to-resize (same as current)
- `.width_range(80.0..=200.0)` constrains width (same as current min/max)
- No manual `ui.separator()` needed (Panel API handles it)

### 2. Toolbar Panel

**Before (nested in manual layout):**
```rust
ui.vertical(|ui| {
    // Toolbar
    ui.horizontal(|ui| {
        // ... toolbar buttons
    });
    
    ui.separator();
    
    // ... tabs and content below
});
```

**After (TopBottomPanel):**
```rust
TopBottomPanel::top("browser_toolbar")
    .show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            // Sidebar toggle buttons (◀◀ / ▶▶)
            // Back/forward navigation (◀ / ▶)
            // URL input field
            // Go button
        });
    });
```

**Changes:**
- Toolbar is now top-level panel (not nested inside `ui.vertical`)
- Toolbar content unchanged (same buttons, same logic)
- No manual `ui.separator()` needed after toolbar

### 3. Browser Content (CentralPanel)

**Before (manual layout with explicit height):**
```rust
ui.vertical(|ui| {
    // ... toolbar above
    
    // Browser content area
    if let Some(active_idx) = self.active_tab {
        let tab = &mut self.tabs[active_idx];
        
        // Claim full available height (DOESN'T WORK - too late!)
        ui.set_height(ui.available_height());
        
        // Render browser content
        let response = tab.view.ui(ui, ui.available_size());
        // ...
    }
});
```

**After (CentralPanel):**
```rust
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

            // ✅ CRITICAL FIX: No need for ui.set_height() anymore!
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

**Changes:**
- **REMOVED:** `ui.set_height(ui.available_height())` line (no longer needed!)
- **Why:** CentralPanel automatically fills remaining space
- **Tab bar:** Unchanged (same ScrollArea::horizontal logic)
- **Webview:** Same `tab.view.ui(ui, ui.available_size())` call, but now gets correct height
- **Events:** Unchanged (same WebViewEvent::Loaded handling)

### 4. What Stays the Same

**No changes to:**
- `BrowserApp` struct fields
- `add_tab()` method
- `close_tab()` method
- Tab management logic
- URL navigation logic
- WebView event handling
- Sidebar toggle state (`sidebar_open: bool`)

**Only change:** `BrowserApp::ui()` method layout structure (~40 lines modified)

## Testing Strategy

### Manual Testing Checklist

**Height Allocation:**
- [ ] Create new tab - web content fills window height (not 150px)
- [ ] Resize window - web content scales accordingly
- [ ] No black area below web content
- [ ] Tab controls fully visible at all times

**Sidebar:**
- [ ] Sidebar visible by default
- [ ] Collapse button (◀◀) hides sidebar with smooth animation
- [ ] Expand button (▶▶) shows sidebar with smooth animation
- [ ] Sidebar resizable by dragging right edge
- [ ] Sidebar respects 80-200px width range

**Toolbar:**
- [ ] Toolbar visible and functional
- [ ] Back/forward buttons work
- [ ] URL bar accepts input and Enter key navigates
- [ ] Go button navigates to URL

**Tab Management:**
- [ ] Create multiple tabs (+ button)
- [ ] Switch between tabs (click tab button)
- [ ] Close tabs (× button)
- [ ] Last tab closed shows empty state
- [ ] Tab bar scrolls horizontally with 10+ tabs

**Edge Cases:**
- [ ] Open very tall webpage - vertical scrolling works
- [ ] Open very wide webpage - horizontal scrolling works
- [ ] Rapid tab switching - no flickering or layout issues
- [ ] Rapid sidebar toggle - smooth animation, no glitches

### Visual Regression Test

Before fix:
- Web content: ~150px height
- Black area: rest of window below web content
- Tab controls: partially hidden or invisible

After fix:
- Web content: full window height (minus toolbar)
- Black area: none
- Tab controls: fully visible

## Success Criteria

1. ✅ Web content fills full window height (not 150px)
2. ✅ No black area below web content
3. ✅ Tab controls fully visible at all times
4. ✅ Sidebar toggle works with smooth animation
5. ✅ Sidebar resizing works (80-200px range)
6. ✅ All existing functionality preserved (tabs, navigation, URL bar)
7. ✅ Window resizing updates web content height dynamically
8. ✅ Code compiles with no warnings
9. ✅ Manual testing checklist passes

## Migration Notes

**Breaking changes:** None - this is a layout fix, not an API change

**Behavioral changes:**
- Sidebar collapse now uses smooth animation (was instant)
- Panels handle layout spacing automatically (no manual separators needed in some places)

**Performance:** No significant impact - Panels are egui's recommended layout system

## References

- egui Panel documentation: https://docs.rs/egui/latest/egui/containers/panel/index.html
- Original implementation plan: `docs/superpowers/plans/2026-08-17-tabbed-browser-sidebar.md`
- Previous fix attempt: Commit `0557bfd - feat: maximize browser viewport with explicit height`

## Next Steps

After design approval:
1. Invoke `writing-plans` skill to create implementation plan
2. Implement panel-based layout in `examples/tabbrowser.rs`
3. Run manual testing checklist
4. Commit fix with detailed commit message
