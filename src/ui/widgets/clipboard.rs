//! Clipboard widget for the top bar
//!
//! Contains Copy, Clear Selection, Paste, and Clear Clipboard buttons.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::ui::button::ButtonState;
use crate::window::manager::FocusState;

/// Widget grouping clipboard-related buttons
pub struct ClipboardWidget {
    // Button states
    copy_state: ButtonState,
    clear_selection_state: ButtonState,
    paste_state: ButtonState,
    clear_clipboard_state: ButtonState,

    // Visibility flags
    show_copy: bool,
    show_paste: bool,
}

impl ClipboardWidget {
    const COPY_LABEL: &'static str = "Copy";
    const CLEAR_SEL_LABEL: &'static str = "X";
    const PASTE_LABEL: &'static str = "Paste";
    const CLEAR_CLIP_LABEL: &'static str = "X";

    pub fn new() -> Self {
        Self {
            copy_state: ButtonState::Normal,
            clear_selection_state: ButtonState::Normal,
            paste_state: ButtonState::Normal,
            clear_clipboard_state: ButtonState::Normal,
            show_copy: false,
            show_paste: false,
        }
    }

    fn button_width(label: &str) -> u16 {
        (label.len() as u16) + 4 // "[ Label ]"
    }

    fn copy_width(&self) -> u16 {
        Self::button_width(Self::COPY_LABEL)
    }

    fn clear_sel_width(&self) -> u16 {
        Self::button_width(Self::CLEAR_SEL_LABEL)
    }

    fn paste_width(&self) -> u16 {
        Self::button_width(Self::PASTE_LABEL)
    }

    fn clear_clip_width(&self) -> u16 {
        Self::button_width(Self::CLEAR_CLIP_LABEL)
    }

    fn render_button(
        buffer: &mut VideoBuffer,
        x: u16,
        label: &str,
        state: &ButtonState,
        theme: &Theme,
    ) -> u16 {
        let (fg_color, bg_color) = match state {
            ButtonState::Normal => (theme.button_normal_fg, theme.button_normal_bg),
            ButtonState::Hovered => (theme.button_hovered_fg, theme.button_hovered_bg),
            ButtonState::Pressed => (theme.button_pressed_fg, theme.button_pressed_bg),
        };

        let mut current_x = x;

        // Render "[ "
        buffer.set(current_x, 0, Cell::new_unchecked('[', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        // Render label
        for ch in label.chars() {
            buffer.set(current_x, 0, Cell::new_unchecked(ch, fg_color, bg_color));
            current_x += 1;
        }

        // Render " ]"
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(']', fg_color, bg_color));

        Self::button_width(label)
    }
}

impl Default for ClipboardWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ClipboardWidget {
    fn width(&self) -> u16 {
        let mut width = 0u16;

        if self.show_copy {
            width += self.copy_width() + self.clear_sel_width();
        }

        if self.show_paste {
            if width > 0 {
                width += 1; // Gap between copy group and paste group
            }
            width += self.paste_width() + self.clear_clip_width();
        }

        width
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, _focus: FocusState) {
        let mut current_x = x;

        if self.show_copy {
            // Render Copy button
            current_x +=
                Self::render_button(buffer, current_x, Self::COPY_LABEL, &self.copy_state, theme);

            // Render Clear Selection [X] button
            current_x += Self::render_button(
                buffer,
                current_x,
                Self::CLEAR_SEL_LABEL,
                &self.clear_selection_state,
                theme,
            );
        }

        if self.show_paste {
            if self.show_copy {
                current_x += 1; // Gap
            }

            // Render Paste button
            current_x += Self::render_button(
                buffer,
                current_x,
                Self::PASTE_LABEL,
                &self.paste_state,
                theme,
            );

            // Render Clear Clipboard [X] button
            Self::render_button(
                buffer,
                current_x,
                Self::CLEAR_CLIP_LABEL,
                &self.clear_clipboard_state,
                theme,
            );
        }
    }

    fn is_visible(&self, ctx: &WidgetContext) -> bool {
        ctx.has_clipboard_content || ctx.has_selection
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        // Reset all states first
        self.reset_state();

        if mouse_y != 0 {
            return;
        }

        let mut current_x = widget_x;

        if self.show_copy {
            // Check Copy button
            let copy_end = current_x + self.copy_width();
            if mouse_x >= current_x && mouse_x < copy_end {
                self.copy_state = ButtonState::Hovered;
                return;
            }
            current_x = copy_end;

            // Check Clear Selection button
            let clear_sel_end = current_x + self.clear_sel_width();
            if mouse_x >= current_x && mouse_x < clear_sel_end {
                self.clear_selection_state = ButtonState::Hovered;
                return;
            }
            current_x = clear_sel_end;
        }

        if self.show_paste {
            if self.show_copy {
                current_x += 1; // Gap
            }

            // Check Paste button
            let paste_end = current_x + self.paste_width();
            if mouse_x >= current_x && mouse_x < paste_end {
                self.paste_state = ButtonState::Hovered;
                return;
            }
            current_x = paste_end;

            // Check Clear Clipboard button
            let clear_clip_end = current_x + self.clear_clip_width();
            if mouse_x >= current_x && mouse_x < clear_clip_end {
                self.clear_clipboard_state = ButtonState::Hovered;
            }
        }
    }

    fn handle_click(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) -> WidgetClickResult {
        if mouse_y != 0 {
            return WidgetClickResult::NotHandled;
        }

        let mut current_x = widget_x;

        if self.show_copy {
            // Check Copy button
            let copy_end = current_x + self.copy_width();
            if mouse_x >= current_x && mouse_x < copy_end {
                self.copy_state = ButtonState::Pressed;
                return WidgetClickResult::CopySelection;
            }
            current_x = copy_end;

            // Check Clear Selection button
            let clear_sel_end = current_x + self.clear_sel_width();
            if mouse_x >= current_x && mouse_x < clear_sel_end {
                self.clear_selection_state = ButtonState::Pressed;
                return WidgetClickResult::ClearSelection;
            }
            current_x = clear_sel_end;
        }

        if self.show_paste {
            if self.show_copy {
                current_x += 1; // Gap
            }

            // Check Paste button
            let paste_end = current_x + self.paste_width();
            if mouse_x >= current_x && mouse_x < paste_end {
                self.paste_state = ButtonState::Pressed;
                return WidgetClickResult::Paste;
            }
            current_x = paste_end;

            // Check Clear Clipboard button
            let clear_clip_end = current_x + self.clear_clip_width();
            if mouse_x >= current_x && mouse_x < clear_clip_end {
                self.clear_clipboard_state = ButtonState::Pressed;
                return WidgetClickResult::ClearClipboard;
            }
        }

        WidgetClickResult::NotHandled
    }

    fn reset_state(&mut self) {
        self.copy_state = ButtonState::Normal;
        self.clear_selection_state = ButtonState::Normal;
        self.paste_state = ButtonState::Normal;
        self.clear_clipboard_state = ButtonState::Normal;
    }

    fn update(&mut self, ctx: &WidgetContext) {
        self.show_copy = ctx.has_selection;
        self.show_paste = ctx.has_clipboard_content;
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Center
    }
}
