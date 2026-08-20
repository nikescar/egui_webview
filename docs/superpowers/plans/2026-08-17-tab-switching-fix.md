# Tab Switching Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix tab switching bug where clicking a tab updates UI state but webview content remains stuck on previous tab's site

**Architecture:** Wrap active webview rendering in `egui::Area` with unique tab ID to isolate each tab's webview in its own UI layer

**Tech Stack:** Rust, egui 0.36.1, wry webview

## Global Constraints

- Target file: `examples/tabbrowser.rs` lines 287-303
- Preserve all existing logic (URL bar updates, event handling)
- No new dependencies
- Manual testing required (no automated tests for webview switching)

---

### Task 1: Wrap Webview in egui::Area for Tab Isolation

**Files:**
- Modify: `examples/tabbrowser.rs:287-303`

**Interfaces:**
- Consumes: Current webview rendering in CentralPanel (lines 287-303)
- Produces: egui::Area-wrapped webview with tab.id for isolation

- [ ] **Step 1: Read current implementation**

```bash
head -n 303 examples/tabbrowser.rs | tail -n 17
```

Expected output:
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

- [ ] **Step 2: Replace with egui::Area wrapper**

Replace lines 287-303 with:

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

Use Edit tool with exact old_string and new_string.

- [ ] **Step 3: Build the project**

```bash
cd /home/wj/work/dure_sijang/reference/egui_webview
cargo build --example tabbrowser
```

Expected: Build succeeds with no errors

- [ ] **Step 4: Run the application**

```bash
cargo run --example tabbrowser
```

Expected: Application starts successfully

- [ ] **Step 5: Manual testing - Basic tab switching**

Test procedure:
1. Click "+ New Tab" button (creates Tab 1 with https://dure.app)
2. Wait for site to load
3. Click "+" button again (creates Tab 2 with https://dure.app)
4. In Tab 2 URL bar, enter: https://www.rust-lang.org
5. Click "Go" button
6. Wait for rust-lang.org to load (verify you see Rust website)
7. Click "Tab 1" button
8. **VERIFY:** Webview shows dure.app (NOT rust-lang.org) ✓
9. Click "Tab 2" button
10. **VERIFY:** Webview shows rust-lang.org (NOT dure.app) ✓
11. Repeat steps 7-10 several times
12. **VERIFY:** Each tab consistently shows its correct site ✓

- [ ] **Step 6: Manual testing - State preservation**

Test procedure:
1. In Tab 1, navigate to https://dure.app
2. Click a link to go to another page within dure.app
3. Note the current URL shown in Tab 1
4. Switch to Tab 2
5. In Tab 2, navigate to https://www.rust-lang.org
6. Switch back to Tab 1
7. **VERIFY:** Tab 1 still shows the second page (not homepage) ✓
8. Click back button (◀)
9. **VERIFY:** Tab 1 navigates to previous page ✓
10. Switch to Tab 2
11. **VERIFY:** Tab 2 still shows rust-lang.org ✓

- [ ] **Step 7: Manual testing - Multiple tabs (3+)**

Test procedure:
1. Create 5 tabs with different sites:
   - Tab 1: https://dure.app
   - Tab 2: https://www.rust-lang.org
   - Tab 3: https://github.com
   - Tab 4: https://crates.io
   - Tab 5: https://doc.rust-lang.org
2. Click through all 5 tabs in random order
3. **VERIFY:** Each tab shows its correct site ✓
4. Close Tab 3 (×)
5. **VERIFY:** Remaining tabs still show correct sites ✓

- [ ] **Step 8: Regression testing**

Test procedure:
1. **Height fix still works:**
   - Create new tab
   - Load any website
   - **VERIFY:** Web content fills full window height (no black area) ✓
2. **Sidebar toggle still works:**
   - Click ◀◀ button
   - **VERIFY:** Sidebar collapses ✓
   - Click ▶▶ button
   - **VERIFY:** Sidebar expands ✓
3. **Tab bar still works:**
   - Create 3 tabs
   - **VERIFY:** Tab buttons visible ✓
   - Click tab close (×) button
   - **VERIFY:** Tab closes ✓
4. **URL bar still updates:**
   - Switch between tabs
   - **VERIFY:** URL bar shows active tab's URL ✓
5. **Navigation still works:**
   - Enter URL in URL bar and press Enter
   - **VERIFY:** Page loads ✓
   - Click back button (◀)
   - **VERIFY:** Navigates back ✓
   - Click forward button (▶)
   - **VERIFY:** Navigates forward ✓

- [ ] **Step 9: Verify success criteria**

From spec section "Success Criteria":
- ✅ Webview content matches the selected tab's site
- ✅ No flickering or visual glitches when switching
- ✅ Browser history preserved per-tab
- ✅ All existing functionality still works
- ✅ Code compiles with no warnings
- ✅ Manual testing checklist passes

- [ ] **Step 10: Commit the fix**

```bash
git add examples/tabbrowser.rs
git commit -m "$(cat <<'EOF'
fix: isolate webview per tab with egui::Area wrapping

Wraps active webview in egui::Area with unique tab ID to fix
tab switching bug where selecting a tab updated UI state but
webview content remained stuck on previous tab's site.

Root cause: Multiple EguiWebView instances rendered in same
CentralPanel without proper isolation. egui couldn't distinguish
between different webview instances.

Solution: egui::Area creates isolated UI layer for each webview.
Only one Area with given ID is visible at a time. Switching tabs
changes which Area is active, properly updating webview content.

Testing:
- Basic tab switching: Tab 1 (site A) ↔ Tab 2 (site B) ✓
- State preservation: Navigation history per-tab ✓
- Multiple tabs: 5 tabs with different sites ✓
- Regression: Height fix, sidebar, navigation all work ✓

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit succeeds

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-17-tab-switching-fix.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
