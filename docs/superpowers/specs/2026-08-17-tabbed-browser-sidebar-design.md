# Tabbed Browser with Sidebar Panel Design

**Date:** 2026-08-17  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Refactor `tabbrowser.rs` to move from multiple browser windows to a single main window with:
- Tabbed interface for multiple browsers
- Collapsible/resizable sidebar panel (left side)
- Shared toolbar with URL bar and navigation controls
- Minimal refactoring of existing WebBrowser code

## Requirements Summary

### Functional Requirements
1. **Tabs:** Compact tabs with horizontal scrolling, "+" button to add, "X" button to close
2. **Empty State:** Show "No tabs open" message when no tabs exist
3. **Initial State:** Start with no tabs (empty state)
4. **New Tab URL:** Default to `https://dure.app`
5. **URL Bar:** Shared toolbar above tabs showing active tab's URL
6. **Navigation:** Back/forward buttons in toolbar
7. **Sidebar:** Left side, collapsible, resizable (280-400px), displays demo text

### Non-Functional Requirements
- Maintain responsive webview input (continuous repaint)
- Preserve existing WebBrowser functionality
- Clean state management (no orphaned tabs)

## Design Approach

**Selected Approach:** Minimal Refactor (Approach 1)

**Rationale:**
- Quick to implement for demo purposes
- Preserves existing WebBrowser code with minimal changes
- Adds wrapper layer (BrowserApp) without deep refactoring

## Architecture Overview

### Core Idea
Wrap the existing `WebBrowser` in a new `BrowserApp` struct that manages multiple browsers as tabs.

### Key Changes
1. Modify `WebBrowser::ui()` to support rendering without Window wrapper
2. Create `BrowserApp` struct to manage tabs, toolbar, sidebar
3. Main UI layout:
   ```
   ┌─────────────┬──────────────────────────────┐
   │   Sidebar   │   [← →] URL: [________] [Go] │  ← Toolbar
   │   (text)    ├──────────────────────────────┤
   │             │  Tab1  Tab2  Tab3  [+]       │  ← Tab Bar
   │             ├──────────────────────────────┤
   │             │                              │
   │             │   Browser Content            │  ← Active Browser
   │             │   (WebBrowser rendered       │
   │             │    without Window)           │
   └─────────────┴──────────────────────────────┘
   ```

## Component Structure

### BrowserApp Struct
```rust
pub struct BrowserApp {
    tabs: Vec<WebBrowser>,           // All browser tabs
    active_tab: Option<usize>,       // None when no tabs open
    sidebar_open: bool,              // Toggle sidebar visibility
    sidebar_width: f32,              // Resizable width (280-400)
    url_input: String,               // Shared URL bar input
    next_tab_id: usize,              // Counter for unique tab IDs
}
```

### WebBrowser Changes
**Modified Signature:**
```rust
ui(&mut self, ctx: &Context, show_as_window: bool) -> bool
```

**New Method:**
```rust
fn content_ui(&mut self, ui: &mut egui::Ui)
```
- Renders browser content without Window wrapper
- Used when `show_as_window == false`

**Existing Behavior Preserved:**
- When `show_as_window == true`: renders as Window (current behavior)
- `id: Id`, `url_bar: String`, `view: EguiWebView` remain unchanged

## Data Flow

### 1. Creating a New Tab (Click "+" button)
```
User clicks "+" 
→ BrowserApp::add_tab() 
→ Create new WebBrowser with URL "https://dure.app"
→ Push to tabs Vec
→ Set active_tab = Some(tabs.len() - 1)
→ Update url_input = "https://dure.app"
→ Request repaint
```

### 2. Switching Tabs (Click tab button)
```
User clicks tab N
→ Update active_tab = Some(N)
→ Update url_input = tabs[N].url_bar
→ Request repaint (active browser content changes)
```

### 3. Closing a Tab (Click "X" button)
```
User clicks "X" on tab N 
→ Remove tabs[N] from Vec
→ If N == active_tab:
    → If tabs.empty(): active_tab = None (show empty state)
    → Else: active_tab = Some(min(N, tabs.len()-1))
→ Request repaint
```

### 4. URL Navigation (Type URL + Enter/Click Go)
```
User enters URL 
→ Update url_input
→ On Enter/Go click:
    → If active_tab.is_some():
        → tabs[active_tab].view.load_url(url_input)
        → Browser navigates
        → WebViewEvent::Loaded fires
        → Update tab.url_bar and url_input
```

### 5. Sidebar Toggle
```
User clicks sidebar toggle 
→ sidebar_open = !sidebar_open
→ SidePanel shows/hides
→ Request repaint
```

## UI Layout

### Panel Hierarchy (using egui Panel API)

```rust
fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    // 1. Left Sidebar Panel (collapsible, resizable)
    egui::SidePanel::left("browser_sidebar")
        .resizable(true)
        .size_range(280.0..=400.0)
        .show_collapsible(ui, &mut self.sidebar_open, |ui| {
            ui.heading("Sidebar");
            ui.label("This is some demo text in the sidebar.");
            ui.label("You can add more content here later.");
        });

    // 2. Top Toolbar Panel (URL bar + nav buttons)
    egui::TopBottomPanel::top("browser_toolbar")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Back/Forward buttons
                if ui.button("◀").clicked() { 
                    if let Some(idx) = self.active_tab {
                        self.tabs[idx].view.back();
                    }
                }
                if ui.button("▶").clicked() { 
                    if let Some(idx) = self.active_tab {
                        self.tabs[idx].view.forward();
                    }
                }
                
                // URL input
                ui.label("URL:");
                let response = ui.text_edit_singleline(&mut self.url_input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(idx) = self.active_tab {
                        self.tabs[idx].view.load_url(&self.url_input).ok();
                    }
                }
                
                if ui.button("Go").clicked() {
                    if let Some(idx) = self.active_tab {
                        self.tabs[idx].view.load_url(&self.url_input).ok();
                    }
                }
            });
        });

    // 3. Central Panel (Tab Bar + Browser Content)
    egui::CentralPanel::default().show(ui, |ui| {
        // Tab bar with horizontal scrolling
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut tab_to_close = None;
                
                for (idx, tab) in self.tabs.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(
                                self.active_tab == Some(idx), 
                                format!("Tab {}", idx + 1)
                            ).clicked() {
                                self.active_tab = Some(idx);
                                self.url_input = tab.url_bar.clone();
                            }
                            
                            if ui.small_button("×").clicked() {
                                tab_to_close = Some(idx);
                            }
                        });
                    });
                }
                
                if ui.button("+").clicked() {
                    self.add_tab(ui.ctx(), frame, "https://dure.app");
                }
                
                // Handle tab close after iteration
                if let Some(idx) = tab_to_close {
                    self.close_tab(idx);
                }
            });
        });
        
        ui.separator();
        
        // Browser content or empty state
        if let Some(active_idx) = self.active_tab {
            let tab = &mut self.tabs[active_idx];
            let events = tab.view.ui(ui, ui.available_size()).events;
            
            // Handle WebView events
            for event in events {
                if let WebViewEvent::Loaded(url) = event {
                    tab.url_bar = url.clone();
                    self.url_input = url;
                }
            }
        } else {
            // Empty state
            ui.centered_and_justified(|ui| {
                ui.heading("No tabs open");
                ui.label("Click '+' to create a new tab.");
            });
        }
    });
}
```

### Layout Components
- **Sidebar:** `SidePanel::left()` with toggle + resize (280-400px range)
- **Toolbar:** `TopBottomPanel::top()` for URL bar and navigation
- **Main Area:** `CentralPanel` for tabs + content
- **Tab Bar:** Horizontal `ScrollArea` for scrollable tabs

## Event Handling

### Browser Events (from WebView)
```rust
// Capture events from active browser
if let Some(active_idx) = self.active_tab {
    let tab = &mut self.tabs[active_idx];
    let events = tab.view.ui(ui, ui.available_size()).events;
    
    for event in events {
        match event {
            WebViewEvent::Loaded(url) => {
                // Update tab's URL bar
                tab.url_bar = url.clone();
                // Sync to shared URL input
                self.url_input = url;
            }
        }
    }
}
```

### Navigation Controls
```rust
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
```

### Tab Management
```rust
fn close_tab(&mut self, idx: usize) {
    self.tabs.remove(idx);
    
    if self.tabs.is_empty() {
        self.active_tab = None;
        self.url_input.clear();
    } else if let Some(active) = self.active_tab {
        if active >= self.tabs.len() {
            self.active_tab = Some(self.tabs.len() - 1);
        } else if idx <= active && active > 0 {
            self.active_tab = Some(active - 1);
        }
        
        if let Some(new_active) = self.active_tab {
            self.url_input = self.tabs[new_active].url_bar.clone();
        }
    }
}
```

### Continuous Repaint (Critical for WebView)
```rust
// In main UI loop
ctx.request_repaint();

// Process GTK events on Linux/BSD
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
```

## State Management

### Initialization (Application Start)
```rust
impl BrowserApp {
    pub fn new(ctx: &Context, frame: &impl HasWindowHandle) -> Self {
        init_webview(ctx);  // Initialize webview system once
        
        Self {
            tabs: Vec::new(),           // Start with no tabs (empty state)
            active_tab: None,           // No active tab
            sidebar_open: true,         // Sidebar visible by default
            sidebar_width: 280.0,       // Default width
            url_input: String::new(),   // Empty URL bar
            next_tab_id: 0,            // ID counter starts at 0
        }
    }
}
```

### Tab Creation
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

### State Consistency Rules
1. **Active Tab Invariant:**
   - `active_tab` is `Some(i)` ⟹ `i < tabs.len()`
   - `tabs.is_empty()` ⟹ `active_tab == None`

2. **URL Sync:**
   - When switching tabs: `url_input = tabs[active_tab].url_bar`
   - When browser navigates: `url_input = loaded_url`

3. **WebView Lifecycle:**
   - Call `init_webview(ctx)` once at app start
   - Call `webview_end_frame(ctx)` at end of each frame
   - Each WebBrowser manages its own `EguiWebView` instance

### Cleanup
```rust
// WebBrowser already has Drop trait that logs cleanup
impl Drop for WebBrowser {
    fn drop(&mut self) {
        println!("🗑️  Dropping WebBrowser {:?}", self.id);
    }
}
// This handles cleanup automatically when tabs are removed from Vec
```

## Implementation Files

### Files to Modify
- `examples/tabbrowser.rs` - Complete refactor with BrowserApp

### Files to Reference
- `reference/egui-material3/examples/stories/tabs_window.rs` - Tab UI patterns
- `reference/egui/crates/egui_demo_app/src/wrap_app.rs` - Panel layout patterns
- `reference/egui/crates/egui_demo_app/src/backend_panel.rs` - Sidebar implementation

## Testing Strategy

### Manual Testing Checklist
1. **Tab Management:**
   - [ ] Click "+" creates new tab with https://dure.app
   - [ ] New tab becomes active automatically
   - [ ] Click tab switches to that tab
   - [ ] Click "X" closes tab
   - [ ] Closing last tab shows empty state
   - [ ] Empty state displays message and "+" button

2. **Navigation:**
   - [ ] Back button works on active tab
   - [ ] Forward button works on active tab
   - [ ] URL bar shows active tab's URL
   - [ ] Typing URL + Enter navigates active tab
   - [ ] Click "Go" button navigates active tab
   - [ ] URL bar updates when page loads

3. **Sidebar:**
   - [ ] Sidebar visible by default
   - [ ] Toggle button shows/hides sidebar
   - [ ] Sidebar can be resized by dragging
   - [ ] Sidebar respects 280-400px size range
   - [ ] Sidebar displays demo text

4. **Edge Cases:**
   - [ ] Multiple tabs scroll horizontally
   - [ ] Closing middle tab preserves active tab
   - [ ] Closing active tab switches to adjacent tab
   - [ ] WebView input remains responsive (no lag)

### Performance Testing
- [ ] Create 10+ tabs - UI remains responsive
- [ ] Switch between tabs quickly - no lag
- [ ] Type in webview textboxes - responsive input
- [ ] Resize sidebar - smooth animation

## Open Questions

None - all questions resolved during brainstorming.

## Success Criteria

1. ✅ Browsers render in main window (not subwindows)
2. ✅ Tabbed interface with horizontal scrolling
3. ✅ Sidebar panel (collapsible, resizable)
4. ✅ Shared toolbar with URL bar and navigation
5. ✅ Empty state when no tabs open
6. ✅ Default URL: https://dure.app
7. ✅ Minimal refactoring of WebBrowser code

## References

- egui SidePanel: https://docs.rs/egui/latest/egui/containers/panel/struct.SidePanel.html
- egui TopBottomPanel: https://docs.rs/egui/latest/egui/containers/panel/struct.TopBottomPanel.html
- egui ScrollArea: https://docs.rs/egui/latest/egui/containers/scroll_area/struct.ScrollArea.html
- Material 3 Tabs: https://material-web.dev/components/tabs/
