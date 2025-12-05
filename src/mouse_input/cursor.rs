//! Cursor position tracking (platform-independent)

/// Calculate adaptive mouse sensitivity based on screen size
/// Larger screens need higher sensitivity to traverse quickly
/// Smaller screens need lower sensitivity for precision
pub fn calculate_sensitivity(cols: usize, rows: usize) -> f32 {
    // Reference: 80x25 = 2000 cells, feels good at ~0.35 sensitivity
    let reference_cells = 80.0 * 25.0; // 2000 cells
    let current_cells = (cols * rows) as f32;

    // Base sensitivity for reference screen
    let base_sensitivity = 0.35;

    // Scale: larger screens get proportionally higher sensitivity
    // Use sqrt to prevent extreme values
    let scale = (current_cells / reference_cells).sqrt();

    // Clamp to reasonable range (0.2 minimum for precision, 1.0 maximum)
    (base_sensitivity * scale).clamp(0.2, 1.0)
}

/// Cursor position tracker
/// Uses usize for pixel coordinates (compatible with framebuffer renderer)
pub struct CursorTracker {
    pub x: usize,
    pub y: usize,
    max_x: usize,
    max_y: usize,
    sensitivity: f32,
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    invert_x: bool,
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    invert_y: bool,
    // Accumulators for fractional movement (prevents jumpy behavior)
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    accum_x: f32,
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    accum_y: f32,
}

impl CursorTracker {
    pub fn new(max_x: usize, max_y: usize, invert_x: bool, invert_y: bool) -> Self {
        // Calculate adaptive sensitivity based on screen size
        let sensitivity = calculate_sensitivity(max_x, max_y);
        Self {
            x: max_x / 2,
            y: max_y / 2,
            max_x,
            max_y,
            sensitivity,
            invert_x,
            invert_y,
            accum_x: 0.0,
            accum_y: 0.0,
        }
    }

    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    pub fn update(&mut self, dx: i8, dy: i8) {
        let dx = if self.invert_x { -dx } else { dx };
        // PS/2 mouse reports positive dy as "up" (toward user), but screen Y increases downward
        // So we invert by default, and the invert_y flag un-inverts it
        let dy = if self.invert_y { dy } else { -dy };

        // Accumulate fractional movement
        self.accum_x += dx as f32 * self.sensitivity;
        self.accum_y += dy as f32 * self.sensitivity;

        // Only move when accumulator reaches a full cell (±1.0)
        let move_x = self.accum_x.trunc() as i32;
        let move_y = self.accum_y.trunc() as i32;

        // Keep the fractional part for next update
        self.accum_x -= move_x as f32;
        self.accum_y -= move_y as f32;

        // Apply movement
        let new_x = (self.x as i32 + move_x).max(0).min(self.max_x as i32 - 1);
        let new_y = (self.y as i32 + move_y).max(0).min(self.max_y as i32 - 1);

        self.x = new_x as usize;
        self.y = new_y as usize;
    }

    #[allow(dead_code)]
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    /// Get position as u16 (for crossterm event compatibility)
    pub fn position_u16(&self) -> (u16, u16) {
        (self.x as u16, self.y as u16)
    }

    #[allow(dead_code)]
    pub fn set_bounds(&mut self, max_x: usize, max_y: usize) {
        self.max_x = max_x;
        self.max_y = max_y;
        self.x = self.x.min(max_x.saturating_sub(1));
        self.y = self.y.min(max_y.saturating_sub(1));
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x.min(self.max_x.saturating_sub(1));
        self.y = y.min(self.max_y.saturating_sub(1));
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.1, 5.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitivity_calculation() {
        // Reference size should give base sensitivity
        let sens = calculate_sensitivity(80, 25);
        assert!((sens - 0.35).abs() < 0.01);

        // Larger screen should have higher sensitivity
        let sens_large = calculate_sensitivity(160, 50);
        assert!(sens_large > sens);

        // Smaller screen should have lower sensitivity (but clamped)
        let sens_small = calculate_sensitivity(40, 12);
        assert!(sens_small >= 0.2);
    }

    #[test]
    fn test_cursor_tracker_movement() {
        let mut cursor = CursorTracker::new(80, 25, false, false);

        // Initial position should be center
        assert_eq!(cursor.x, 40);
        assert_eq!(cursor.y, 12);

        // Move right
        cursor.update(10, 0);
        // Position should increase (accounting for sensitivity)
        assert!(cursor.x >= 40);
    }

    #[test]
    fn test_cursor_tracker_bounds() {
        let mut cursor = CursorTracker::new(80, 25, false, false);

        // Move far right - should clamp
        for _ in 0..1000 {
            cursor.update(127, 0);
        }
        assert!(cursor.x < 80);

        // Move far left - should clamp
        for _ in 0..1000 {
            cursor.update(-128, 0);
        }
        assert_eq!(cursor.x, 0);
    }
}
