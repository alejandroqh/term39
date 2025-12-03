/// Selection type determines how text is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    /// Character-by-character selection (click and drag)
    Character,
    /// Word selection (double-click)
    Word,
    /// Line selection (triple-click)
    Line,
    /// Block/Rectangle selection (Alt+drag)
    Block,
}

/// Selection state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionState {
    /// No active selection
    #[allow(dead_code)]
    None,
    /// Selection in progress (dragging)
    Active,
    /// Selection complete (mouse released)
    Complete,
}

/// Represents a position in the terminal grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

impl Position {
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

/// Represents a text selection in the terminal
#[derive(Debug, Clone)]
pub struct Selection {
    /// Starting position (where mouse was pressed)
    pub start: Position,
    /// Current/ending position (where mouse is/was released)
    pub end: Position,
    /// Type of selection
    pub selection_type: SelectionType,
    /// Current state
    pub state: SelectionState,
}

impl Selection {
    /// Create a new selection starting at a position
    pub fn new(start: Position, selection_type: SelectionType) -> Self {
        Self {
            start,
            end: start,
            selection_type,
            state: SelectionState::Active,
        }
    }

    /// Update the end position (during drag)
    pub fn update_end(&mut self, end: Position) {
        self.end = end;
    }

    /// Mark selection as complete
    pub fn complete(&mut self) {
        self.state = SelectionState::Complete;
    }

    /// Check if a position is within the selection
    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.normalized_bounds();

        match self.selection_type {
            SelectionType::Block => {
                // Rectangle selection: check if within the box
                let min_col = start.col.min(end.col);
                let max_col = start.col.max(end.col);
                let min_row = start.row.min(end.row);
                let max_row = start.row.max(end.row);

                pos.col >= min_col && pos.col <= max_col && pos.row >= min_row && pos.row <= max_row
            }
            _ => {
                // Linear selection (character, word, line)
                let start_idx = (start.row as usize * 1000) + start.col as usize;
                let end_idx = (end.row as usize * 1000) + end.col as usize;
                let pos_idx = (pos.row as usize * 1000) + pos.col as usize;

                pos_idx >= start_idx && pos_idx <= end_idx
            }
        }
    }

    /// Get normalized bounds (ensures start <= end)
    pub fn normalized_bounds(&self) -> (Position, Position) {
        match self.selection_type {
            SelectionType::Block => {
                // For block selection, keep original orientation
                (self.start, self.end)
            }
            _ => {
                // For linear selection, normalize to ensure start <= end
                let start_idx = (self.start.row as usize * 1000) + self.start.col as usize;
                let end_idx = (self.end.row as usize * 1000) + self.end.col as usize;

                if start_idx <= end_idx {
                    (self.start, self.end)
                } else {
                    (self.end, self.start)
                }
            }
        }
    }

    /// Expand selection to word boundaries
    pub fn expand_to_word(&mut self, get_char: impl Fn(Position) -> Option<char>) {
        self.selection_type = SelectionType::Word;

        // Expand start backward to word boundary
        let mut start = self.start;
        while start.col > 0 {
            let prev_pos = Position::new(start.col - 1, start.row);
            if let Some(ch) = get_char(prev_pos) {
                if is_word_char(ch) {
                    start = prev_pos;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Expand end forward to word boundary
        let mut end = self.end;
        while let Some(ch) = get_char(end) {
            if is_word_char(ch) {
                end.col += 1;
            } else {
                break;
            }
        }

        self.start = start;
        self.end = end;
    }

    /// Expand selection to line boundaries
    pub fn expand_to_line(&mut self, width: u16) {
        self.selection_type = SelectionType::Line;

        // Expand to full line(s)
        let (start, end) = self.normalized_bounds();

        self.start = Position::new(0, start.row);
        self.end = Position::new(width.saturating_sub(1), end.row);
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Check if selection is too small (less than 2 characters)
    /// This is used to avoid showing single-character selections from accidental clicks
    pub fn is_too_small(&self) -> bool {
        let (start, end) = self.normalized_bounds();
        match self.selection_type {
            SelectionType::Block => {
                // For block selection, require at least 2 cells in any dimension
                let width = (end.col as i32 - start.col as i32).abs() + 1;
                let height = (end.row as i32 - start.row as i32).abs() + 1;
                width * height < 2
            }
            _ => {
                // For linear selection, calculate total characters
                if start.row == end.row {
                    // Same row: just check column difference
                    (end.col as i32 - start.col as i32).abs() < 1
                } else {
                    // Different rows: always has at least 2 characters
                    false
                }
            }
        }
    }
}

/// Check if a character is part of a word (for word selection)
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_contains() {
        let mut sel = Selection::new(Position::new(5, 0), SelectionType::Character);
        sel.update_end(Position::new(10, 0));

        assert!(sel.contains(Position::new(5, 0)));
        assert!(sel.contains(Position::new(7, 0)));
        assert!(sel.contains(Position::new(10, 0)));
        assert!(!sel.contains(Position::new(4, 0)));
        assert!(!sel.contains(Position::new(11, 0)));
    }

    #[test]
    fn test_block_selection() {
        let mut sel = Selection::new(Position::new(5, 5), SelectionType::Block);
        sel.update_end(Position::new(10, 10));

        assert!(sel.contains(Position::new(5, 5)));
        assert!(sel.contains(Position::new(7, 7)));
        assert!(sel.contains(Position::new(10, 10)));
        assert!(!sel.contains(Position::new(4, 7)));
        assert!(!sel.contains(Position::new(11, 7)));
    }

    #[test]
    fn test_normalized_bounds() {
        let mut sel = Selection::new(Position::new(10, 0), SelectionType::Character);
        sel.update_end(Position::new(5, 0));

        let (start, end) = sel.normalized_bounds();
        assert_eq!(start.col, 5);
        assert_eq!(end.col, 10);
    }

    #[test]
    fn test_is_too_small() {
        // Single character selection (click without drag) should be too small
        let sel = Selection::new(Position::new(5, 0), SelectionType::Character);
        assert!(sel.is_too_small());

        // Two character selection should not be too small
        let mut sel2 = Selection::new(Position::new(5, 0), SelectionType::Character);
        sel2.update_end(Position::new(6, 0));
        assert!(!sel2.is_too_small());

        // Multi-row selection should not be too small
        let mut sel3 = Selection::new(Position::new(5, 0), SelectionType::Character);
        sel3.update_end(Position::new(5, 1));
        assert!(!sel3.is_too_small());

        // Single cell block selection should be too small
        let sel_block = Selection::new(Position::new(5, 5), SelectionType::Block);
        assert!(sel_block.is_too_small());

        // 2-cell block selection should not be too small
        let mut sel_block2 = Selection::new(Position::new(5, 5), SelectionType::Block);
        sel_block2.update_end(Position::new(6, 5));
        assert!(!sel_block2.is_too_small());
    }
}
