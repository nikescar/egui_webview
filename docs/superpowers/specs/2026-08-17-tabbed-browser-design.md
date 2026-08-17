# Tabbed Browser with Sidebar Design

**Date:** 2026-08-17  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Transform the egui_webview example from individual floating windows into a Chrome/Firefox-style tabbed browser with a global sidebar containing favorites, categorized directory, and browsing history.

## Goals

1. **Tabbed Interface**: Single main window with tabs at the top, each tab displays a different webpage
2. **Global Sidebar**: Resizable and collapsible sidebar with three internal tabs (favorites, directory, history)
3. **Session Persistence**: Restore all tabs and state when the application restarts
4. **Full Tab Management**: Support tab reordering, duplication, and pinning
5. **Browser-like UX**: Familiar browser interactions and patterns

## Requirements Summary

From brainstorming session:

- **Tab Style**: Chrome/Firefox style - one main window with tabs, each tab contains a different webpage
- **Sidebar Scope**: Global - shows content relevant to ALL tabs (shared favorites/history across all tabs)
- **Sidebar Contents**:
  - Favorites: Bookmarked URLs that users can click to open
  - Directory: Categorized list of URLs (organized into categories/folders)
  - Histories: List of all visited URLs with timestamps, clickable to revisit
- **Sidebar Click Behavior**: Always opens in new tab
- **Startup Behavior**: Restore last session - reopen all tabs that were open when app closed
- **Last Tab Close**: Show empty state - main window with no tabs, user must click "New Tab" to continue
- **Sidebar Visibility**: Resizable and collapsible - users can resize width and hide/show it
- **Tab Operations**: Full featured - reorder + duplicate + pin tabs
- **Initial Content**: Pre-populated with useful Rust/egui resources

## Architecture

### Approach: Component-Based Separation

The application uses **Approach 2** (Component-Based Separation) with clear separation of concerns:

```rust
pub struct TabbedBrowser {
    tab_manager: TabManager,
    sidebar: Sidebar,
    ui_state: UiState,
}
```

**Component Responsibilities:**

- **`TabManager`**: Owns all browser tabs, manages active tab selection, handles tab lifecycle (create/close/reorder/pin/duplicate)
- **`Sidebar`**: Manages favorites, categorized directory, and history. Handles sidebar visibility and width
- **`UiState`**: Tracks UI-specific state like sidebar collapse state, window dimensions, and transient UI flags

**State Ownership:**
- Each `BrowserTab` owns its `EguiWebView` (from existing library)
- `Sidebar` owns all bookmark/history data structures
- `TabManager` owns the tab collection and knows which tab is active
- No shared ownership (`Arc`/`Rc`) needed - clean single ownership model

**Persistence Strategy:**
- All three components implement `Serialize`/`Deserialize` via serde
- On shutdown: serialize entire `TabbedBrowser` state to JSON file
- On startup: deserialize from JSON, recreate webviews for each restored tab

## Component Structure

### TabManager

```rust
pub struct TabManager {
    tabs: Vec<BrowserTab>,
    active_index: Option<usize>,  // None when no tabs open
    next_tab_id: u64,              // For unique IDs
}

pub struct BrowserTab {
    id: u64,
    title: String,                 // Page title or "Loading..."
    url: String,
    pinned: bool,
    
    #[serde(skip)]
    view: Option<EguiWebView>,    // Recreated on deserialize
}
```

**Methods:**
- `create_tab(url: String) -> u64` - Creates new tab, returns tab ID
- `close_tab(index: usize) -> bool` - Closes tab, returns true if last tab closed
- `reorder(from: usize, to: usize)` - Moves tab from one position to another
- `duplicate_tab(index: usize) -> u64` - Duplicates tab at index
- `toggle_pin(index: usize)` - Toggles pinned state
- `set_active(index: usize)` - Changes active tab
- `get_active_tab() -> Option<&BrowserTab>` - Returns active tab reference

### Sidebar

```rust
pub struct Sidebar {
    open: bool,
    width: f32,                    // Current width in pixels
    active_tab: SidebarTab,        // Which sidebar tab is selected
    
    favorites: Vec<Bookmark>,
    directory: Vec<Category>,
    history: Vec<HistoryEntry>,
}

pub enum SidebarTab {
    Favorites,
    Directory,
    History,
}

pub struct Bookmark {
    title: String,
    url: String,
}

pub struct Category {
    name: String,
    items: Vec<Bookmark>,
    expanded: bool,                // UI state for collapsible categories
}

pub struct HistoryEntry {
    url: String,
    title: String,
    timestamp: u64,                // Unix timestamp
}
```

**Methods:**
- `add_favorite(title: String, url: String)` - Adds bookmark to favorites
- `remove_favorite(index: usize)` - Removes favorite
- `add_history(url: String, title: String)` - Records page visit
- `prune_history()` - Keeps only last 500 entries
- `toggle_category(name: &str)` - Expands/collapses category

### UiState

```rust
pub struct UiState {
    sidebar_default_width: f32,    // Reset width when reopening
    show_new_tab_button: bool,     // Always true for now
}
```

## Data Flow & Interactions

### Tab Operations

**Creating a new tab:**
1. User clicks "New Tab" button or sidebar URL
2. `TabManager::create_tab(url)` generates unique ID, creates `BrowserTab`
3. `BrowserTab` initializes `EguiWebView` with the URL
4. `TabManager` sets this as active tab
5. If opened from sidebar, `Sidebar::add_history()` called with URL

**Closing a tab:**
1. User clicks close button on tab
2. `TabManager::close_tab(index)` removes tab from vector
3. `EguiWebView` automatically drops (cleanup handled by existing library)
4. If last tab closed: `active_index = None` (empty state)
5. Otherwise: activate adjacent tab (prefer right, then left)

**Navigation:**
1. `BrowserTab` receives navigation event from webview
2. Updates its `url` and `title` fields
3. Calls `sidebar.add_history()` to record visit
4. History limited to last 500 entries (configurable)

### Sidebar Operations

**Clicking favorite/directory/history item:**
1. `Sidebar` detects click on URL item
2. Calls `tab_manager.create_tab(url)` to open new tab
3. New tab becomes active automatically

### Tab Reordering

**Dragging a tab:**
1. egui drag-and-drop detects tab drag
2. `TabManager::reorder(from_index, to_index)` swaps positions
3. Pinned tabs stay in pinned section (left side)
4. `active_index` updated to track moved tab

### Tab Duplication

**Duplicating a tab:**
1. Right-click menu on tab
2. `TabManager::duplicate_tab(index)` clones URL/title
3. Creates new tab with same URL next to original
4. New webview loads the URL independently

## Persistence & Session Restoration

### Save on Exit

When application closes:
1. Serialize `TabbedBrowser` to JSON using serde
2. Save to `~/.local/share/egui_webview/session.json` (Linux/BSD)
3. `EguiWebView` fields marked `#[serde(skip)]` - only save URL/title/state

### Restore on Startup

When application starts:
1. Check if `session.json` exists
2. If yes: deserialize into `TabbedBrowser`
3. For each `BrowserTab`: recreate `EguiWebView` with saved URL
4. Restore active tab index, sidebar state, pinned tabs
5. If file doesn't exist or parse fails: create default state

### Default State (First Launch)

```rust
impl Default for TabbedBrowser {
    fn default() -> Self {
        Self {
            tab_manager: TabManager::with_defaults(),
            sidebar: Sidebar::with_rust_defaults(),
            ui_state: UiState::default(),
        }
    }
}
```

**Initial State:**
- Opens with 1 tab showing `https://www.rust-lang.org`
- Sidebar pre-populated with:
  - **Favorites**: Empty (user adds their own)
  - **Directory**: Categories for:
    - "Rust" → rust-lang.org, docs.rs, crates.io
    - "egui" → egui.rs, github.com/emilk/egui, egui demo
    - "Examples" → current default URLs from code
  - **History**: Empty

### Data Limits

- History: Keep last 500 entries, auto-prune oldest
- Favorites: No limit (user-managed)
- Directory: No limit (user-managed)
- Max tabs on restore: 100 (safety limit to prevent hang)

## UI Layout & Rendering

### Main Window Layout

```
┌─────────────────────────────────────────────────┐
│  [Tab 1] [Tab 2*] [Tab 3] [+]         [≡]      │ ← Top bar (tabs + new tab button)
├─────────┬───────────────────────────────────────┤
│         │                                       │
│ Sidebar │         Active Tab Content            │
│         │         (EguiWebView)                 │
│ [Fav]   │                                       │
│ [Dir]   │                                       │
│ [His]   │                                       │
│         │                                       │
│         │                                       │
└─────────┴───────────────────────────────────────┘
    ↑                                              
  Resizable                                        
```

### Rendering Order

**1. Top Panel (`egui::TopBottomPanel::top`):**
- Horizontal tab strip using primary tabs pattern from `tabs_window.rs`
- Pinned tabs first (with pin icon 📌), then regular tabs
- Tab drag-and-drop using egui's DnD API
- "+" button on right side for new tab
- Each tab shows: `[icon] Title [×]`

**2. Left Panel (`egui::SidePanel::left`):**
- Resizable (280-400px range, matches `backend_panel.rs` pattern)
- Collapsible via `show_collapsible()` API
- Secondary tabs for Fav/Dir/His using `tabs_secondary` from material3
- Scrollable content area below tabs

**3. Central Panel (`egui::CentralPanel`):**
- If `active_index.is_some()`: render active tab's webview
- If `active_index.is_none()`: show empty state with centered "New Tab" button

### Tab Strip Behavior

- Tabs are `48px` height (material3 standard)
- Pinned tabs: `48px` width (icon only)
- Regular tabs: flexible width, min `120px`, max `240px`
- Active tab: highlighted with accent color
- Tab close button (`×`): shows on hover, always visible on active tab

### Sidebar Tabs Content

- **Favorites**: Vertical list of bookmarks, click to open in new tab
- **Directory**: Collapsible tree view (categories expand/collapse), click to open in new tab
- **History**: Scrollable list, newest first, shows timestamp, click to open in new tab

## File Structure

New files to create:

```
src/
├── lib.rs                    # Existing - exports TabbedBrowser
├── tabbed_browser.rs         # Main TabbedBrowser struct
├── tab_manager.rs            # TabManager + BrowserTab
├── sidebar.rs                # Sidebar + Bookmark/Category/HistoryEntry
├── ui_state.rs               # UiState
└── persistence.rs            # Session save/load helpers

examples/
├── webview.rs                # Existing - update to use TabbedBrowser
└── tabbed_browser.rs         # New example showcasing tabbed interface
```

## Implementation Notes

### Dependencies (Already in Cargo.toml)

- `egui` - UI framework
- `egui_extras` - Additional UI components
- `wry` - Webview backend (via egui_webview)
- `serde` + `serde_json` - Serialization
- `eframe` - For native window (dev-dependencies)

### Reference Examples Used

- **Tabs**: `reference/egui-material3/examples/stories/tabs_window.rs`
  - Primary tabs pattern for tab strip
  - Secondary tabs pattern for sidebar tabs
  - Tab state management patterns

- **Sidebar**: `reference/egui/crates/egui_demo_app/src/wrap_app.rs` + `backend_panel.rs`
  - Resizable side panel pattern
  - Collapsible panel with `show_collapsible()`
  - Panel width management

- **Existing**: Current `examples/webview.rs`
  - WebBrowser struct pattern
  - EguiWebView integration
  - GTK event loop handling

### Key Technical Decisions

1. **No async needed**: All operations are synchronous, webview library handles async internally
2. **Vec for tabs**: Simple index-based access, reordering via `Vec::swap`
3. **Option<usize> for active**: Clean representation of "no tabs" state
4. **JSON for persistence**: Human-readable, easy to debug, serde support
5. **Component composition**: Each struct has single responsibility, testable in isolation

## Error Handling

- **Session load fails**: Log warning, use default state, don't crash
- **Webview creation fails**: Show error in tab content area, don't crash app
- **Invalid URL**: Show error message in tab, keep tab open
- **History limit exceeded**: Auto-prune to 400 entries when 500 limit reached
- **Corrupted session file**: Delete file, log warning, use default state

## Testing Strategy

### Unit Tests

- `TabManager`: Create, close, reorder, pin, duplicate operations
- `Sidebar`: Add/remove favorites, category expansion, history pruning
- Persistence: Serialize/deserialize round-trip

### Integration Tests

- Session save/restore with multiple tabs
- Tab navigation triggering history updates
- Sidebar URL clicks creating new tabs

### Manual Testing

- Open 50+ tabs, verify performance
- Test on OpenBSD (target platform)
- Verify GTK event loop integration
- Test drag-and-drop tab reordering
- Test sidebar resize and collapse

## Future Enhancements (Out of Scope)

- Tab groups/sessions
- Search within favorites/history
- Bookmark folders (beyond categories)
- Import/export bookmarks
- Keyboard shortcuts (Ctrl+T, Ctrl+W, etc.)
- Middle-click to close tabs
- Tab preview on hover
- Multiple windows

## Success Criteria

1. ✅ Single main window with tab strip at top
2. ✅ Can open, close, reorder, pin, duplicate tabs
3. ✅ Resizable and collapsible sidebar with 3 internal tabs
4. ✅ Favorites, categorized directory, and history work as specified
5. ✅ Clicking sidebar URLs opens new tabs
6. ✅ Session persists and restores correctly
7. ✅ Empty state shows when no tabs open
8. ✅ Pre-populated with Rust/egui resources on first launch
9. ✅ No crashes with 50+ tabs
10. ✅ Works correctly on OpenBSD with GTK backend

## Next Steps

After design approval:
1. Invoke `writing-plans` skill to create detailed implementation plan
2. Implement components in order: TabManager → Sidebar → TabbedBrowser → UI
3. Add persistence layer
4. Update example to use new tabbed interface
5. Test thoroughly on OpenBSD
