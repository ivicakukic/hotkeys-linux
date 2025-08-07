/// Linux layout system for HotKeys UI
/// Provides window positioning and styling abstractions

use std::fmt::{self, Display, Formatter};


/// Cross-platform rectangle structure
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Rect {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self { left, top, right, bottom }
    }

    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    pub fn x(&self) -> f64 {
        self.left
    }

    pub fn y(&self) -> f64 {
        self.top
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}


/// Window layout configuration
#[derive(Clone, Debug, PartialEq)]
pub struct WindowLayout {
    pub style: WindowStyle,
    pub size: Size,
}

impl Default for WindowLayout {
    fn default() -> Self {
        WindowLayout {
            style: WindowStyle::default(),
            size: Size { width: 800.0, height: 600.0 },
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum WindowStyle {
    /// Regular window with title bar and borders (shows in taskbar)
    Window,
    /// Borderless window that shows in taskbar
    Taskbar,
}

impl Default for WindowStyle {
    fn default() -> Self {
        WindowStyle::Window
    }
}

impl WindowStyle {
    pub fn from_string(s: &str) -> Self {
        match s {
            "Taskbar" => WindowStyle::Taskbar,
            "Window" => WindowStyle::Window,
            _ => WindowStyle::Window, // Fallback variant
        }
    }

    /// Whether this window style should have decorations (title bar, borders)
    pub fn has_decorations(&self) -> bool {
        match self {
            WindowStyle::Window => true,
            WindowStyle::Taskbar => false,
        }
    }

}

impl Display for WindowStyle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// 3x3 board layout calculations
#[derive(Clone)]
pub struct BoardLayout {
    window_rect: Rect,
    header_rect: Rect,
    grid_rect: Rect,
    tile_size: Size,
}

/// Board layout is a 3x3 grid with a header at the top.
///
/// Grid rows are inverted (tile_id's 1,2,3 are bottom row, identical to numpad layout):
///    7 8 9
///    4 5 6
///    1 2 3
///
/// Tiles are 1/3 of grid width and height.
///
/// Header is 10% of window height, grid takes remaining 90%.
impl BoardLayout {
    pub fn new(window_width: f64, window_height: f64) -> Self {
        // Header takes top 10% of window
        let header_height = window_height / 10.0;
        let header_rect = Rect::new(0.0, 0.0, window_width, header_height);

        // Grid takes remaining 90% of window
        let grid_rect = Rect::new(0.0, header_height, window_width, window_height);
        let grid_height = grid_rect.height();

        // Each tile is 1/3 width, 1/3 of grid height
        let tile_width = window_width / 3.0;
        let tile_height = grid_height / 3.0;

        Self {
            window_rect: Rect::new(0.0, 0.0, window_width, window_height),
            header_rect,
            grid_rect,
            tile_size: Size { width: tile_width, height: tile_height },
        }
    }

    /// Get rectangle for a specific tile (1-9, row-major order starting from bottom left)
    pub fn get_tile_rect(&self, tile_id: u8) -> Option<Rect> {
        if tile_id < 1 || tile_id > 9 {
            return None;
        }

        let index = (tile_id - 1) as i32;
        let row = 2 - index / 3; // because rows start from bottom
        let col = index % 3;

        let size = self.tile_size.clone();
        let left = col as f64 * size.width;
        let top = row as f64 * size.height + self.grid_rect.top;
        let right = left + size.width;
        let bottom = top + size.height;

        Some(Rect::new(left, top, right, bottom))
    }

    /// Get the window rectangle
    pub fn get_window_rect(&self) -> Rect {
        self.window_rect
    }

    /// Get the header rectangle
    pub fn get_header_rect(&self) -> Rect {
        self.header_rect
    }

    /// Get the grid rectangle
    pub fn get_grid_rect(&self) -> Rect {
        self.grid_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10.0, 20.0, 110.0, 120.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 100.0);
    }

    #[test]
    fn test_window_layout_default() {
        let layout = WindowLayout::default();
        assert_eq!(layout.style, WindowStyle::Window);
        assert_eq!(layout.size.width, 800.0);
        assert_eq!(layout.size.height, 600.0);
    }

    #[test]
    fn test_board_layout_creation() {
        let board = BoardLayout::new(900.0, 600.0);

        // Header should be 1/10 of height
        assert_eq!(board.header_rect.height(), 60.0);

        // Grid should be 9/10 of height
        assert_eq!(board.grid_rect.height(), 540.0);

        // Each tile should be 300x180
        assert_eq!(board.tile_size, Size { width: 300.0, height: 180.0 });
    }

    #[test]
    fn test_tile_rectangles() {
        let board = BoardLayout::new(900.0, 600.0);

        // Test top-left tile (7)
        let tile1 = board.get_tile_rect(7).unwrap();
        assert_eq!(tile1, Rect::new(0.0, 60.0, 300.0, 240.0));

        // Test middle tile (5)
        let tile5 = board.get_tile_rect(5).unwrap();
        assert_eq!(tile5, Rect::new(300.0, 240.0, 600.0, 420.0));

        // Test bottom-right tile (3)
        let tile9 = board.get_tile_rect(3).unwrap();
        assert_eq!(tile9, Rect::new(600.0, 420.0, 900.0, 600.0));

        // Test invalid tile
        assert!(board.get_tile_rect(0).is_none());
        assert!(board.get_tile_rect(10).is_none());
    }

}