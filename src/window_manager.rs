use crate::charset::Charset;
use crate::terminal_window::TerminalWindow;
use crate::video_buffer::VideoBuffer;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;

/// Focus state - either desktop or a specific window
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FocusState {
    Desktop,
    Window(u32),
}

/// Window manager handles z-order, focus, and interactions
pub struct WindowManager {
    windows: Vec<TerminalWindow>,
    next_id: u32,
    focus: FocusState,

    // Interaction state
    dragging: Option<DragState>,
    resizing: Option<ResizeState>,
    last_click: Option<LastClick>,
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    window_id: u32,
    offset_x: i16,
    offset_y: i16,
}

#[derive(Clone, Copy, Debug)]
struct ResizeState {
    window_id: u32,
    start_x: u16,
    start_y: u16,
    start_width: u16,
    start_height: u16,
}

#[derive(Clone, Debug)]
struct LastClick {
    window_id: u32,
    x: u16,
    y: u16,
    time: Instant,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            focus: FocusState::Desktop,
            dragging: None,
            resizing: None,
            last_click: None,
        }
    }

    /// Create and add a new terminal window (returns window ID)
    pub fn create_window(&mut self, x: u16, y: u16, width: u16, height: u16, title: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        // Unfocus all windows
        for w in &mut self.windows {
            w.set_focused(false);
        }

        // Create terminal window (ignore errors for now)
        if let Ok(mut terminal_window) = TerminalWindow::new(id, x, y, width, height, title) {
            terminal_window.set_focused(true);
            self.windows.push(terminal_window);
            self.focus = FocusState::Window(id);
            id
        } else {
            // Failed to create terminal window
            id
        }
    }

    /// Bring window to front and focus it
    pub fn focus_window(&mut self, id: u32) {
        // Find window
        if let Some(pos) = self.windows.iter().position(|w| w.id() == id) {
            // Move to end (top of z-order)
            let mut window = self.windows.remove(pos);

            // Unfocus all windows
            for w in &mut self.windows {
                w.set_focused(false);
            }

            // Focus this window
            window.set_focused(true);
            self.windows.push(window);
            self.focus = FocusState::Window(id);
        }
    }

    /// Focus the desktop (unfocus all windows)
    pub fn focus_desktop(&mut self) {
        for w in &mut self.windows {
            w.set_focused(false);
        }
        self.focus = FocusState::Desktop;
    }

    /// Get the current focus state
    pub fn get_focus(&self) -> FocusState {
        self.focus
    }

    /// Find top-most window at coordinates
    pub fn window_at(&self, x: u16, y: u16) -> Option<u32> {
        // Iterate backwards (top to bottom)
        for window in self.windows.iter().rev() {
            if window.contains_point(x, y) {
                return Some(window.id());
            }
        }
        None
    }

    /// Handle mouse event
    pub fn handle_mouse_event(&mut self, buffer: &mut VideoBuffer, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_down(buffer, event.column, event.row);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.handle_mouse_drag(buffer, event.column, event.row);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.handle_mouse_up();
            }
            _ => {}
        }
    }

    fn handle_mouse_down(&mut self, buffer: &mut VideoBuffer, x: u16, y: u16) {
        // Find window at click position
        if let Some(window_id) = self.window_at(x, y) {
            // Get the window
            if let Some(terminal_window) = self.windows.iter().find(|w| w.id() == window_id) {
                let window = &terminal_window.window;

                // Check if clicking close button
                if terminal_window.is_in_close_button(x, y) {
                    self.close_window(window_id);
                    return;
                }

                // Check if clicking maximize button
                if window.is_in_maximize_button(x, y) {
                    let (buffer_width, buffer_height) = buffer.dimensions();

                    // Find the window mutably and toggle maximize
                    if let Some(win) = self.windows.iter_mut().find(|w| w.id() == window_id) {
                        win.window.toggle_maximize(buffer_width, buffer_height);
                        // Resize the terminal to match new window size
                        let _ = win.resize(win.window.width, win.window.height);
                    }
                    return;
                }

                // Check if clicking minimize button
                if window.is_in_minimize_button(x, y) {
                    // Find the window mutably and minimize it
                    if let Some(win) = self.windows.iter_mut().find(|w| w.id() == window_id) {
                        win.window.minimize();
                    }

                    // Find the next non-minimized window to focus (from top of z-order)
                    let next_window_id = self
                        .windows
                        .iter()
                        .rev()
                        .find(|w| !w.window.is_minimized && w.id() != window_id)
                        .map(|w| w.id());

                    if let Some(next_id) = next_window_id {
                        // Focus the next available window
                        self.focus_window(next_id);
                    } else {
                        // No other windows available, focus desktop
                        self.focus_desktop();
                    }
                    return;
                }

                // Check if clicking resize handle (only if not maximized)
                if !window.is_maximized && terminal_window.is_in_resize_handle(x, y) {
                    self.resizing = Some(ResizeState {
                        window_id,
                        start_x: x,
                        start_y: y,
                        start_width: window.width,
                        start_height: window.height,
                    });
                    return;
                }

                // Check if clicking title bar (for dragging or double-click maximize)
                if terminal_window.is_in_title_bar(x, y) {
                    let now = Instant::now();

                    // Check for double-click (within 500ms, same window and position)
                    let is_double_click = if let Some(ref last) = self.last_click {
                        last.window_id == window_id
                            && last.x == x
                            && last.y == y
                            && now.duration_since(last.time).as_millis() < 500
                    } else {
                        false
                    };

                    if is_double_click {
                        // Double-click detected - toggle maximize
                        let (buffer_width, buffer_height) = buffer.dimensions();
                        if let Some(win) = self.windows.iter_mut().find(|w| w.id() == window_id) {
                            win.window.toggle_maximize(buffer_width, buffer_height);
                            // Resize the terminal to match new window size
                            let _ = win.resize(win.window.width, win.window.height);
                        }
                        // Clear last click so we don't trigger triple-click
                        self.last_click = None;
                    } else {
                        // Single click - record it and start dragging if not maximized
                        self.last_click = Some(LastClick {
                            window_id,
                            x,
                            y,
                            time: now,
                        });

                        // Only start dragging if not maximized
                        if !window.is_maximized {
                            let offset_x = x as i16 - window.x as i16;
                            let offset_y = y as i16 - window.y as i16;

                            self.dragging = Some(DragState {
                                window_id,
                                offset_x,
                                offset_y,
                            });
                        }
                    }

                    // Don't bring to front yet for title bar clicks
                    // to avoid interfering with double-click detection
                }
            }

            // Bring window to front and focus it
            self.focus_window(window_id);
        } else {
            // Clicked on desktop - focus it
            self.focus_desktop();
        }
    }

    #[allow(clippy::collapsible_if)]
    fn handle_mouse_drag(&mut self, buffer: &mut VideoBuffer, x: u16, y: u16) {
        // Handle window dragging
        if let Some(drag) = self.dragging {
            if let Some(terminal_window) =
                self.windows.iter_mut().find(|w| w.id() == drag.window_id)
            {
                let (buffer_width, buffer_height) = buffer.dimensions();

                // Calculate desired position
                let desired_x = x as i16 - drag.offset_x;
                let desired_y = y as i16 - drag.offset_y;

                // Constrain x: keep entire window visible horizontally
                let max_x = buffer_width.saturating_sub(terminal_window.window.width);
                let new_x = (desired_x.max(0) as u16).min(max_x);

                // Constrain y: keep below top bar and entire window visible vertically
                let max_y = buffer_height
                    .saturating_sub(terminal_window.window.height)
                    .saturating_sub(1); // -1 for button bar
                let new_y = (desired_y.max(1) as u16).min(max_y);

                terminal_window.window.x = new_x;
                terminal_window.window.y = new_y;
            }
        }

        // Handle window resizing
        if let Some(resize) = self.resizing {
            if let Some(terminal_window) =
                self.windows.iter_mut().find(|w| w.id() == resize.window_id)
            {
                // Calculate new size
                let delta_x = x as i16 - resize.start_x as i16;
                let delta_y = y as i16 - resize.start_y as i16;

                let new_width = (resize.start_width as i16 + delta_x).max(20) as u16;
                let new_height = (resize.start_height as i16 + delta_y).max(5) as u16;

                terminal_window.window.width = new_width;
                terminal_window.window.height = new_height;

                // Resize the terminal to match new window size
                let _ = terminal_window.resize(new_width, new_height);
            }
        }
    }

    fn handle_mouse_up(&mut self) {
        self.dragging = None;
        self.resizing = None;
    }

    /// Render all windows in z-order (bottom to top)
    pub fn render_all(&mut self, buffer: &mut VideoBuffer, charset: &Charset) {
        let mut windows_to_close = Vec::new();

        for i in 0..self.windows.len() {
            // Process terminal output before rendering
            if let Ok(false) = self.windows[i].process_output() {
                // Shell process has exited, mark for closure
                windows_to_close.push(self.windows[i].id());
            }

            // Check if this window is being resized
            let is_resizing = self
                .resizing
                .is_some_and(|r| r.window_id == self.windows[i].id());
            self.windows[i].render(buffer, is_resizing, charset);
        }

        // Close windows whose shell processes have exited
        for window_id in windows_to_close {
            self.close_window(window_id);
        }
    }

    /// Get the number of windows
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Get window info for button bar rendering (id, title, is_focused, is_minimized)
    /// Returns windows sorted by creation order (ID), not z-order
    pub fn get_window_list(&self) -> Vec<(u32, &str, bool, bool)> {
        let mut list: Vec<(u32, &str, bool, bool)> = self
            .windows
            .iter()
            .map(|w| {
                (
                    w.id(),
                    w.window.title.as_str(),
                    w.window.is_focused,
                    w.window.is_minimized,
                )
            })
            .collect();

        // Sort by window ID to maintain creation order
        list.sort_by_key(|(id, _, _, _)| *id);
        list
    }

    /// Handle click on button bar - returns window ID if clicked on a button
    pub fn button_bar_click(&mut self, x: u16, bar_y: u16, click_y: u16) -> Option<u32> {
        // Only process if clicking on the button bar row
        if click_y != bar_y {
            return None;
        }

        // Get windows sorted by creation order (same as display order)
        let mut sorted_windows: Vec<&TerminalWindow> = self.windows.iter().collect();
        sorted_windows.sort_by_key(|w| w.id());

        let mut current_x = 1u16; // Start at position 1 (skip left edge)
        let mut clicked_window_id: Option<u32> = None;

        for terminal_window in sorted_windows {
            let window = &terminal_window.window;

            // Button format: [ Title ]
            // Max button width is 18 chars (including brackets and spaces)
            let max_title_len = 14; // Leaves room for [ ] and spaces
            let button_title = if window.title.len() > max_title_len {
                &window.title[..max_title_len]
            } else {
                &window.title
            };

            let button_width = button_title.len() as u16 + 4; // "[ " + title + " ]"
            let button_end = current_x + button_width;

            // Check if click is within this button
            if x >= current_x && x < button_end {
                clicked_window_id = Some(window.id);
                break;
            }

            // Move to next button position (with 1 space gap)
            current_x = button_end + 1;
        }

        // Focus the clicked window if found
        if let Some(window_id) = clicked_window_id {
            // If the window is minimized, restore it first
            #[allow(clippy::collapsible_if)]
            if let Some(win) = self.windows.iter_mut().find(|w| w.id() == window_id) {
                if win.window.is_minimized {
                    win.window.restore_from_minimize();
                }
            }

            self.focus_window(window_id);
            return Some(window_id);
        }

        None
    }

    /// Send input to the focused terminal window
    #[allow(clippy::collapsible_if)]
    pub fn send_to_focused(&mut self, s: &str) -> std::io::Result<()> {
        if let FocusState::Window(id) = self.focus {
            if let Some(terminal_window) = self.windows.iter_mut().find(|w| w.id() == id) {
                return terminal_window.send_str(s);
            }
        }
        Ok(())
    }

    /// Send a character to the focused terminal window
    #[allow(clippy::collapsible_if)]
    pub fn send_char_to_focused(&mut self, c: char) -> std::io::Result<()> {
        if let FocusState::Window(id) = self.focus {
            if let Some(terminal_window) = self.windows.iter_mut().find(|w| w.id() == id) {
                return terminal_window.send_char(c);
            }
        }
        Ok(())
    }

    /// Close window by ID
    pub fn close_window(&mut self, id: u32) {
        if let Some(pos) = self.windows.iter().position(|w| w.id() == id) {
            self.windows.remove(pos);

            // Update focus - if we closed the focused window, focus desktop
            if self.focus == FocusState::Window(id) {
                self.focus = FocusState::Desktop;
            }
        }
    }

    /// Maximize window by ID
    pub fn maximize_window(&mut self, id: u32, buffer_width: u16, buffer_height: u16) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id() == id) {
            // Only maximize if not already maximized
            if !win.window.is_maximized {
                win.window.toggle_maximize(buffer_width, buffer_height);
                // Resize the terminal to match new window size
                let _ = win.resize(win.window.width, win.window.height);
            }
        }
    }

    /// Cycle to the next window (for ALT+TAB)
    /// If the next window is minimized, restore it
    pub fn cycle_to_next_window(&mut self) {
        if self.windows.is_empty() {
            return;
        }

        // Get sorted list of windows by creation order (ID)
        let mut sorted_windows: Vec<u32> = self.windows.iter().map(|w| w.id()).collect();
        sorted_windows.sort();

        // Find current window index
        let current_index = if let FocusState::Window(id) = self.focus {
            sorted_windows.iter().position(|&w_id| w_id == id)
        } else {
            None
        };

        // Calculate next window index
        let next_index = match current_index {
            Some(idx) => (idx + 1) % sorted_windows.len(),
            None => 0, // If desktop is focused, go to first window
        };

        let next_window_id = sorted_windows[next_index];

        // If the window is minimized, restore it
        #[allow(clippy::collapsible_if)]
        if let Some(win) = self.windows.iter_mut().find(|w| w.id() == next_window_id) {
            if win.window.is_minimized {
                win.window.restore_from_minimize();
            }
        }

        // Focus the next window
        self.focus_window(next_window_id);
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
