//! Rendering backend abstraction
//!
//! This module provides an abstraction over different rendering backends:
//! - Terminal backend: Uses crossterm for cross-platform terminal rendering
//! - Framebuffer backend: Uses direct Linux framebuffer for DOS-like modes

use crate::video_buffer::VideoBuffer;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use std::collections::VecDeque;
use std::io;

/// Render backend trait for abstracting terminal vs framebuffer rendering
pub trait RenderBackend {
    /// Present the video buffer to the screen
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()>;

    /// Get the dimensions of the rendering surface (cols, rows)
    fn dimensions(&self) -> (u16, u16);

    /// Check if the backend has been resized
    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>>;

    /// Scale mouse coordinates from TTY/input space to backend space
    /// For terminal backend: returns coordinates as-is
    /// For framebuffer backend: scales from TTY dimensions to framebuffer dimensions
    fn scale_mouse_coords(&self, col: u16, row: u16) -> (u16, u16) {
        (col, row) // Default: no scaling
    }

    /// Update cursor from raw input (framebuffer mode only)
    /// Returns true if cursor moved
    fn update_cursor(&mut self) -> bool {
        false // Default: no cursor updates
    }

    /// Draw cursor on screen (framebuffer mode only)
    fn draw_cursor(&mut self) {
        // Default: no-op
    }

    /// Restore cursor area (framebuffer mode only)
    fn restore_cursor_area(&mut self) {
        // Default: no-op
    }

    /// Get mouse event (framebuffer mode only)
    /// Returns event_type (0=Down, 1=Up, 2=Drag), button_id (0=left, 1=right, 2=middle), and cursor position
    #[allow(dead_code)]
    fn get_mouse_button_event(&mut self) -> Option<(u8, u8, u16, u16)> {
        None // Default: no mouse events (event_type, button_id, col, row)
    }

    /// Get mouse scroll event (framebuffer mode only)
    /// Returns (scroll_direction, col, row) where scroll_direction: 0=up, 1=down
    #[allow(dead_code)]
    fn get_mouse_scroll_event(&mut self) -> Option<(u8, u16, u16)> {
        None // Default: no scroll events
    }

    /// Check if the backend has native mouse input (e.g., framebuffer with /dev/input/mice)
    /// When true, GPM should be skipped to avoid duplicate/conflicting events
    #[allow(dead_code)] // Used on Linux for GPM conflict resolution
    fn has_native_mouse_input(&self) -> bool {
        false // Default: no native mouse input
    }

    /// Set TTY cursor position (for raw mouse input mode in terminal backend)
    /// This inverts the cell colors at the cursor position
    fn set_tty_cursor(&mut self, _col: u16, _row: u16) {
        // Default: no-op (framebuffer uses sprite cursor)
    }

    /// Clear TTY cursor
    fn clear_tty_cursor(&mut self) {
        // Default: no-op
    }
}

/// Terminal-based rendering backend (using crossterm)
pub struct TerminalBackend {
    cols: u16,
    rows: u16,
    stdout: io::Stdout,
    /// TTY cursor position for raw mouse input mode
    tty_cursor: Option<(u16, u16)>,
}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new() -> io::Result<Self> {
        use crossterm::terminal;

        let (cols, rows) = terminal::size()?;
        let stdout = io::stdout();

        Ok(Self {
            cols,
            rows,
            stdout,
            tty_cursor: None,
        })
    }
}

impl RenderBackend for TerminalBackend {
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()> {
        // Apply TTY cursor to buffer before presenting
        if let Some((col, row)) = self.tty_cursor {
            buffer.set_tty_cursor(col, row);
        } else {
            buffer.clear_tty_cursor();
        }
        buffer.present(&mut self.stdout)
    }

    fn set_tty_cursor(&mut self, col: u16, row: u16) {
        self.tty_cursor = Some((col, row));
    }

    fn clear_tty_cursor(&mut self) {
        self.tty_cursor = None;
    }

    fn dimensions(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>> {
        use crossterm::terminal;

        let (new_cols, new_rows) = terminal::size()?;

        if new_cols != self.cols || new_rows != self.rows {
            self.cols = new_cols;
            self.rows = new_rows;
            Ok(Some((new_cols, new_rows)))
        } else {
            Ok(None)
        }
    }
}

/// Framebuffer-based rendering backend (Linux console only)
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub struct FramebufferBackend {
    renderer: crate::framebuffer::FramebufferRenderer,
    tty_cols: u16, // Actual TTY dimensions for mouse coordinate scaling
    tty_rows: u16,
    mouse_input: Option<crate::framebuffer::MouseInput>,
    cursor_tracker: crate::framebuffer::CursorTracker,
    // Button state tracking for event generation
    current_left: bool,
    current_right: bool,
    current_middle: bool,
    // Previous cursor position for detecting movement
    prev_col: u16,
    prev_row: u16,
    // Queue of pending mouse events (event_type, button_id, col, row)
    // event_type: 0=Down, 1=Up, 2=Drag
    button_event_queue: VecDeque<(u8, u8, u16, u16)>,
    // Queue of pending scroll events (scroll_direction, col, row)
    // scroll_direction: 0=up, 1=down
    scroll_event_queue: VecDeque<(u8, u16, u16)>,
}

#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
impl FramebufferBackend {
    /// Create a new framebuffer backend with specified text mode, optional scale, optional font, optional mouse device, axis inversions, and sensitivity
    pub fn new(
        mode: crate::framebuffer::TextMode,
        scale: Option<usize>,
        font_name: Option<&str>,
        mouse_device: Option<&str>,
        invert_x: bool,
        invert_y: bool,
        sensitivity: Option<f32>,
    ) -> io::Result<Self> {
        use crossterm::terminal;

        let renderer = crate::framebuffer::FramebufferRenderer::new(mode, scale, font_name)?;

        // Get actual TTY dimensions for mouse coordinate scaling
        let (tty_cols, tty_rows) = terminal::size()?;

        // Try to open raw mouse input (uses specified device or auto-detects)
        let mouse_input = match crate::framebuffer::MouseInput::new(mouse_device) {
            Ok(input) => Some(input),
            Err(e) => {
                eprintln!("Warning: Failed to open mouse input device: {}", e);
                eprintln!("Mouse cursor will not be rendered.");
                eprintln!("Try: sudo chmod a+r /dev/input/mice /dev/input/event*");
                None
            }
        };

        // Get pixel dimensions for cursor tracker
        let (pixel_width, pixel_height) = renderer.pixel_dimensions();
        let mut cursor_tracker =
            crate::framebuffer::CursorTracker::new(pixel_width, pixel_height, invert_x, invert_y);

        // Apply sensitivity override if provided
        if let Some(sens) = sensitivity {
            cursor_tracker.set_sensitivity(sens);
        }

        // Calculate initial cell position from cursor tracker's pixel position
        // This fixes incorrect position_changed detection on first mouse movement
        // Note: cursor_tracker.x/y are in LOGICAL (unscaled) pixel coordinates
        // bounded by (base_width, base_height) from pixel_dimensions()
        let (cols, rows) = renderer.dimensions();
        let (base_width, base_height) = renderer.pixel_dimensions();

        let char_width = if cols > 0 { base_width / cols } else { 1 };
        let char_height = if rows > 0 { base_height / rows } else { 1 };

        // Convert logical pixel position directly to cell coordinates
        // No offset subtraction needed (cursor is bounded to content area)
        // No scale division needed (cursor is already in logical pixel space)
        let initial_col = if char_width > 0 {
            (cursor_tracker.x / char_width).min(cols.saturating_sub(1)) as u16
        } else {
            0
        };
        let initial_row = if char_height > 0 {
            (cursor_tracker.y / char_height).min(rows.saturating_sub(1)) as u16
        } else {
            0
        };

        Ok(Self {
            renderer,
            tty_cols,
            tty_rows,
            mouse_input,
            cursor_tracker,
            current_left: false,
            current_right: false,
            current_middle: false,
            prev_col: initial_col,
            prev_row: initial_row,
            button_event_queue: VecDeque::new(),
            scroll_event_queue: VecDeque::new(),
        })
    }

    /// Get current cursor position (pixel coordinates)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_tracker.x, self.cursor_tracker.y)
    }

    /// Set cursor visibility
    #[allow(dead_code)]
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.renderer.set_cursor_visible(visible);
    }

    /// Queue a mouse event with current cursor position
    /// event_type: 0=Down, 1=Up, 2=Drag
    /// Note: cursor_tracker.x/y are in LOGICAL (unscaled) pixel coordinates
    #[allow(dead_code)]
    fn queue_mouse_event(&mut self, event_type: u8, button_id: u8) {
        // Calculate coordinates at the time of the event
        let (cols, rows) = self.renderer.dimensions();
        let (base_width, base_height) = self.renderer.pixel_dimensions();

        // Add division-by-zero protection
        let char_width = if cols > 0 { base_width / cols } else { 1 };
        let char_height = if rows > 0 { base_height / rows } else { 1 };

        // Convert logical pixel position directly to cell coordinates
        // No offset subtraction needed (cursor is bounded to content area)
        // No scale division needed (cursor is already in logical pixel space)
        let col = if char_width > 0 {
            (self.cursor_tracker.x / char_width).min(cols.saturating_sub(1)) as u16
        } else {
            0
        };
        let row = if char_height > 0 {
            (self.cursor_tracker.y / char_height).min(rows.saturating_sub(1)) as u16
        } else {
            0
        };

        self.button_event_queue
            .push_back((event_type, button_id, col, row));
    }
}

#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
impl RenderBackend for FramebufferBackend {
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()> {
        self.renderer.render_buffer(buffer);
        Ok(())
    }

    fn dimensions(&self) -> (u16, u16) {
        let (cols, rows) = self.renderer.dimensions();
        (cols as u16, rows as u16)
    }

    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>> {
        // Framebuffer doesn't resize - mode is fixed
        Ok(None)
    }

    fn scale_mouse_coords(&self, col: u16, row: u16) -> (u16, u16) {
        // Scale mouse coordinates from TTY space to framebuffer space
        let (fb_cols, fb_rows) = self.dimensions();

        // If TTY dimensions match framebuffer dimensions, no scaling needed
        if self.tty_cols == fb_cols && self.tty_rows == fb_rows {
            return (col, row);
        }

        // Calculate scaling factors
        let col_scale = fb_cols as f32 / self.tty_cols as f32;
        let row_scale = fb_rows as f32 / self.tty_rows as f32;

        // Apply scaling
        let scaled_col = (col as f32 * col_scale).round() as u16;
        let scaled_row = (row as f32 * row_scale).round() as u16;

        // Clamp to framebuffer dimensions
        let clamped_col = scaled_col.min(fb_cols.saturating_sub(1));
        let clamped_row = scaled_row.min(fb_rows.saturating_sub(1));

        (clamped_col, clamped_row)
    }

    fn update_cursor(&mut self) -> bool {
        if let Some(ref mut mouse_input) = self.mouse_input {
            let mut moved = false;

            // Helper function to calculate cell coordinates from pixel position
            // Note: tracker.x/y are in LOGICAL (unscaled) pixel coordinates
            // bounded by (base_width, base_height) from pixel_dimensions()
            let calc_coords =
                |tracker: &crate::framebuffer::CursorTracker,
                 renderer: &crate::framebuffer::FramebufferRenderer| {
                    let (cols, rows) = renderer.dimensions();
                    let (base_width, base_height) = renderer.pixel_dimensions();

                    let char_width = if cols > 0 { base_width / cols } else { 1 };
                    let char_height = if rows > 0 { base_height / rows } else { 1 };

                    // Convert logical pixel position directly to cell coordinates
                    // No offset subtraction needed (cursor is bounded to content area)
                    // No scale division needed (cursor is already in logical pixel space)
                    let col = if char_width > 0 {
                        (tracker.x / char_width).min(cols.saturating_sub(1)) as u16
                    } else {
                        0
                    };
                    let row = if char_height > 0 {
                        (tracker.y / char_height).min(rows.saturating_sub(1)) as u16
                    } else {
                        0
                    };

                    (col, row)
                };

            // Process all pending mouse events
            while let Ok(Some(event)) = mouse_input.read_event() {
                self.cursor_tracker.update(event.dx, event.dy);

                // Calculate current cell position
                let (col, row) = calc_coords(&self.cursor_tracker, &self.renderer);

                // Check if cursor moved to a different cell
                let position_changed = col != self.prev_col || row != self.prev_row;

                // Process events in correct order: DOWN -> DRAG -> UP
                // This ensures proper event sequencing for clicks and drags

                // 1. Check for button DOWN events (press)
                if event.buttons.left && !self.current_left {
                    self.button_event_queue.push_back((0, 0, col, row)); // Down left
                    self.current_left = true;
                }
                if event.buttons.right && !self.current_right {
                    self.button_event_queue.push_back((0, 1, col, row)); // Down right
                    self.current_right = true;
                }
                if event.buttons.middle && !self.current_middle {
                    self.button_event_queue.push_back((0, 2, col, row)); // Down middle
                    self.current_middle = true;
                }

                // 2. Check for DRAG events (position changed while button held)
                if position_changed {
                    if self.current_left {
                        self.button_event_queue.push_back((2, 0, col, row)); // Drag left
                    }
                    if self.current_right {
                        self.button_event_queue.push_back((2, 1, col, row)); // Drag right
                    }
                    if self.current_middle {
                        self.button_event_queue.push_back((2, 2, col, row)); // Drag middle
                    }
                }

                // 3. Check for button UP events (release)
                if !event.buttons.left && self.current_left {
                    self.button_event_queue.push_back((1, 0, col, row)); // Up left
                    self.current_left = false;
                }
                if !event.buttons.right && self.current_right {
                    self.button_event_queue.push_back((1, 1, col, row)); // Up right
                    self.current_right = false;
                }
                if !event.buttons.middle && self.current_middle {
                    self.button_event_queue.push_back((1, 2, col, row)); // Up middle
                    self.current_middle = false;
                }

                // 4. Check for scroll events
                if event.scroll > 0 {
                    // Scroll up (positive = scroll wheel away from user)
                    for _ in 0..event.scroll {
                        self.scroll_event_queue.push_back((0, col, row));
                    }
                } else if event.scroll < 0 {
                    // Scroll down (negative = scroll wheel toward user)
                    for _ in 0..(-event.scroll) {
                        self.scroll_event_queue.push_back((1, col, row));
                    }
                }

                // Always update previous position to track cursor
                self.prev_col = col;
                self.prev_row = row;

                moved = true;
            }

            moved
        } else {
            false
        }
    }

    fn get_mouse_button_event(&mut self) -> Option<(u8, u8, u16, u16)> {
        // Return the next queued mouse event (event_type, button_id, col, row)
        // event_type: 0=Down, 1=Up, 2=Drag
        // Using pop_front() for O(1) dequeue instead of remove(0) which is O(n)
        self.button_event_queue.pop_front()
    }

    fn get_mouse_scroll_event(&mut self) -> Option<(u8, u16, u16)> {
        // Return the next queued scroll event (scroll_direction, col, row)
        // scroll_direction: 0=up, 1=down
        self.scroll_event_queue.pop_front()
    }

    fn has_native_mouse_input(&self) -> bool {
        // Framebuffer backend reads directly from /dev/input/mice
        // GPM should be skipped to avoid duplicate/conflicting events
        self.mouse_input.is_some()
    }

    fn draw_cursor(&mut self) {
        let (x, y) = self.cursor_position();
        self.renderer.draw_cursor(x, y);
    }

    fn restore_cursor_area(&mut self) {
        self.renderer.restore_cursor_area();
    }
}
