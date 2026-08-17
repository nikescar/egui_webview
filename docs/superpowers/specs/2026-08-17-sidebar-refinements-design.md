# Tabbed Browser Sidebar Refinements Design

**Date:** 2026-08-17  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This spec defines UI refinements to the existing tabbed browser implementation in `examples/tabbrowser.rs`. The changes improve toolbar ergonomics by moving the sidebar toggle button and maximize browser viewport by explicitly claiming available vertical space.

## Requirements

1. **Sidebar toggle relocation**: Move show/hide sidebar button from inside/outside sidebar to toolbar, positioned leftmost before back button
2. **Browser height maximization**: Make browser content fill 100% of available vertical space in its container

## Design Decisions

### Question 1: Sidebar Button Placement
**Decision:** In toolbar area, as leftmost button before back (◀) button  
**Rationale:** Unifies all navigation controls in toolbar, makes sidebar control discoverable and accessible

### Question 2: Implementation Approach
**Decision:** Minimal change approach (Approach A)  
**Rationale:** Simplest solution, leverages egui's layout system naturally, minimal code changes

## Architecture

### Current Structure

```
BrowserApp::ui
├── Horizontal layout
│   ├── Sidebar (conditional, with toggle inside)
│   └── Main content (vertical)
│       ├── Toolbar (back/forward/URL/go)
│       ├── Tab bar or empty state
│       └── Browser content (uses available_size)
```

### Changes

1. **Sidebar toggle relocates** from inside sidebar to toolbar (leftmost position)
2. **Browser content** explicitly claims remaining vertical space using `ui.set_height(ui.available_height())`

**Why:** 
- All navigation controls unified in toolbar (sidebar toggle, back, forward, URL)
- Browser content guaranteed to fill available height instead of relying on parent layout calculations
- No structural changes to layout hierarchy - just button relocation and explicit height claim

## Component Changes

### File: examples/tabbrowser.rs

#### Change 1: Toolbar - Add Sidebar Toggle

**Location:** Lines ~189-204 (toolbar section)

**Implementation:**
```rust
ui.horizontal(|ui| {
    // Sidebar toggle (NEW - leftmost position)
    if self.sidebar_open {
        if ui.button("◀◀").clicked() {
            self.sidebar_open = false;
        }
    } else {
        if ui.button("▶▶").clicked() {
            self.sidebar_open = true;
        }
    }
    
    ui.separator();  // Visual separation from navigation
    
    // Back button (existing)
    if ui.button("◀").clicked() {
        if let Some(idx) = self.active_tab {
            self.tabs[idx].view.back();
        }
    }
    
    // ... rest of toolbar (forward, URL, go)
});
```

#### Change 2: Remove Old Sidebar Toggles

**Location:** Lines ~156-158 (Hide Sidebar) and ~180-182 (Show Sidebar)

**Action:** Delete these conditional toggle buttons. Sidebar content becomes static display only.

**Before:**
```rust
if self.sidebar_open {
    if ui.button("Hide Sidebar").clicked() {
        self.sidebar_open = false;
    }
    // ... sidebar content
} else {
    if ui.button("Show Sidebar").clicked() {
        self.sidebar_open = true;
    }
}
```

**After:**
```rust
if self.sidebar_open {
    // ... sidebar content only (no toggle button)
} else {
    // Nothing - toolbar has the toggle
}
```

#### Change 3: Browser Height

**Location:** Line ~292 (browser content rendering)

**Implementation:**
```rust
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];
    
    // Claim full available height (NEW)
    ui.set_height(ui.available_height());
    
    // Render browser content
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

### State Management

**No changes needed** - `sidebar_open: bool` already exists in `BrowserApp` struct and is used for conditional rendering. Button relocation reuses this existing state.

## Visual Design

### Toolbar Layout (Left to Right)

```
[◀◀] | [◀] [▶] | URL: [________________] [Go]
 ^      ^   ^               ^              ^
 |      |   |               |              |
Toggle Back Fwd          URL input        Go
sidebar
```

### Button Behavior

- **Sidebar open** → button shows "◀◀" (collapse hint, arrows pointing left)
- **Sidebar closed** → button shows "▶▶" (expand hint, arrows pointing right)
- Click toggles `self.sidebar_open` boolean

### Sidebar Content

- No longer has internal toggle button
- Just displays heading + demo text
- Cleaner, focused on content display
- Width constraints remain unchanged (280-400px)

### Browser Viewport

- Tab bar stays minimal height (auto-sized by content)
- Browser content explicitly claims `ui.available_height()` 
- Maximizes viewable area for web content
- No white space gaps between tab bar and browser

## Implementation Files

### Files to Modify

- `examples/tabbrowser.rs` - Apply all 3 changes (add toolbar toggle, remove sidebar toggles, set browser height)

### Files to Reference

- Current implementation in `examples/tabbrowser.rs` (recent commits 553d2d6 through 8095799)

## Testing Strategy

### Manual Testing Checklist

1. **Sidebar toggle in toolbar:**
   - [ ] Button appears leftmost in toolbar
   - [ ] Click "◀◀" when sidebar open → sidebar collapses, button changes to "▶▶"
   - [ ] Click "▶▶" when sidebar closed → sidebar expands, button changes to "◀◀"
   - [ ] Sidebar content has no toggle button inside it

2. **Browser height:**
   - [ ] Create a tab, navigate to a webpage
   - [ ] Browser content fills all vertical space below tab bar
   - [ ] No white space gaps between tab bar and browser
   - [ ] Resize window → browser content adjusts to maintain full height

3. **Regression testing:**
   - [ ] All existing functionality works (tab switching, URL navigation, back/forward)
   - [ ] Multiple tabs still scroll horizontally
   - [ ] Empty state still shows centered message
   - [ ] Tab closing works correctly
   - [ ] WebView events (URL updates) still fire

### Build Verification

```bash
# Build example
cargo build --example tabbrowser

# Run example
cargo run --example tabbrowser
```

**Expected result:** Clean build with no warnings. Browser window shows:
- Toolbar with sidebar toggle at far left
- Browser fills viewport below tab bar
- All navigation controls functional

## Open Questions

None - all design decisions validated with user.

## Success Criteria

1. ✅ Sidebar toggle button moved to toolbar (leftmost position)
2. ✅ Sidebar toggle uses "◀◀" / "▶▶" icons
3. ✅ Old sidebar toggle buttons removed
4. ✅ Browser content fills 100% of available vertical space
5. ✅ No regression in existing functionality
6. ✅ Clean build with no warnings

## Next Steps

1. Invoke `writing-plans` skill to create implementation plan
2. Implement changes in `examples/tabbrowser.rs`
3. Test manually against checklist
4. Verify build passes
5. Commit changes

## References

- Original implementation commits: 553d2d6 through 8095799
- Design spec: `docs/superpowers/specs/2026-08-17-tabbed-browser-sidebar-design.md`
- Implementation plan: `docs/superpowers/plans/2026-08-17-tabbed-browser-sidebar.md`
