//! Paint module for display list generation
//!
//! This module implements the Painter trait and DisplayCommand enum for
//! converting layout trees into draw commands that can be executed by the renderer.

use std::sync::Arc;

/// A rectangle in layout coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        LayoutRect {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &LayoutRect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Check if a point is inside this rectangle
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// Box type for layout nodes
#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous,
}

/// A box in the layout tree
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    pub rect: LayoutRect,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    pub fn new(rect: LayoutRect, box_type: BoxType) -> Self {
        LayoutBox {
            rect,
            box_type,
            children: Vec::new(),
        }
    }

    /// Add a child box
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    /// Get the total number of boxes in this subtree
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

/// Display commands for rendering
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayCommand {
    /// Fill a rectangle with a solid color (RGBA)
    FillRect {
        rect: LayoutRect,
        color: (u8, u8, u8, u8),
    },
    /// Draw text at a position
    DrawText {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: (u8, u8, u8, u8),
    },
    /// Draw an image from raw pixel data
    DrawImage {
        rect: LayoutRect,
        data: Arc<Vec<u8>>,
    },
    /// Set a clipping rectangle for subsequent drawing operations
    ClipRect(LayoutRect),
    /// Pop the current clipping rectangle
    PopClip,
}

/// A display list is a sequence of draw commands
pub type DisplayList = Vec<DisplayCommand>;

/// Painter trait for converting layout trees to display lists
pub trait Painter {
    /// Walk the layout tree and emit draw commands.
    fn paint(&self, layout: &LayoutBox) -> DisplayList;
}

/// Default implementation of the Painter trait
pub struct DefaultPainter;

impl Painter for DefaultPainter {
    fn paint(&self, layout: &LayoutBox) -> DisplayList {
        let mut commands = Vec::new();
        self.paint_recursive(layout, &mut commands);
        commands
    }
}

impl DefaultPainter {
    /// Recursively walk the layout tree and build display commands
    fn paint_recursive(&self, layout: &LayoutBox, commands: &mut DisplayList) {
        // Push clip for this box
        commands.push(DisplayCommand::ClipRect(layout.rect));

        // For now, we'll emit a simple fill rect for each box
        // In a full implementation, this would use the styled properties
        // to determine colors, text, images, etc.
        let color = match layout.box_type {
            BoxType::Block => (200, 200, 200, 255),     // Gray for blocks
            BoxType::Inline => (255, 255, 255, 255),    // White for inline
            BoxType::Anonymous => (240, 240, 240, 255), // Light gray for anonymous
        };

        commands.push(DisplayCommand::FillRect {
            rect: layout.rect,
            color,
        });

        // Recursively paint children
        for child in &layout.children {
            self.paint_recursive(child, commands);
        }

        // Pop clip for this box
        commands.push(DisplayCommand::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_rect_creation() {
        let rect = LayoutRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_layout_rect_contains() {
        let rect = LayoutRect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(15.0, 25.0));
        assert!(rect.contains(10.0, 20.0));
        assert!(!rect.contains(5.0, 25.0));
        assert!(!rect.contains(15.0, 75.0));
    }

    #[test]
    fn test_layout_rect_intersects() {
        let rect1 = LayoutRect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = LayoutRect::new(50.0, 50.0, 100.0, 100.0);
        assert!(rect1.intersects(&rect2));

        let rect3 = LayoutRect::new(200.0, 200.0, 50.0, 50.0);
        assert!(!rect1.intersects(&rect3));
    }

    #[test]
    fn test_layout_box_creation() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 100.0);
        let box_ = LayoutBox::new(rect, BoxType::Block);
        assert_eq!(box_.rect, rect);
        assert_eq!(box_.box_type, BoxType::Block);
        assert!(box_.children.is_empty());
    }

    #[test]
    fn test_layout_box_add_child() {
        let mut parent = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        let child = LayoutBox::new(LayoutRect::new(10.0, 10.0, 50.0, 50.0), BoxType::Inline);
        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_layout_box_count() {
        let mut parent = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        assert_eq!(parent.count(), 1);

        parent.add_child(LayoutBox::new(
            LayoutRect::new(10.0, 10.0, 50.0, 50.0),
            BoxType::Inline,
        ));
        assert_eq!(parent.count(), 2);

        parent.add_child(LayoutBox::new(
            LayoutRect::new(10.0, 70.0, 50.0, 20.0),
            BoxType::Inline,
        ));
        assert_eq!(parent.count(), 3);
    }

    #[test]
    fn test_display_command_fill_rect() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 100.0);
        let cmd = DisplayCommand::FillRect {
            rect,
            color: (255, 0, 0, 255),
        };
        assert_eq!(
            cmd,
            DisplayCommand::FillRect {
                rect,
                color: (255, 0, 0, 255)
            }
        );
    }

    #[test]
    fn test_display_command_draw_text() {
        let cmd = DisplayCommand::DrawText {
            x: 10.0,
            y: 20.0,
            text: "Hello".to_string(),
            size: 16.0,
            color: (0, 0, 0, 255),
        };
        assert_eq!(
            cmd,
            DisplayCommand::DrawText {
                x: 10.0,
                y: 20.0,
                text: "Hello".to_string(),
                size: 16.0,
                color: (0, 0, 0, 255)
            }
        );
    }

    #[test]
    fn test_display_command_draw_image() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 100.0);
        let data = Arc::new(vec![0u8; 10000]);
        let cmd = DisplayCommand::DrawImage {
            rect,
            data: data.clone(),
        };
        assert_eq!(
            cmd,
            DisplayCommand::DrawImage {
                rect,
                data: data.clone()
            }
        );
    }

    #[test]
    fn test_display_command_clip_rect() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 100.0);
        let cmd = DisplayCommand::ClipRect(rect);
        assert_eq!(cmd, DisplayCommand::ClipRect(rect));
    }

    #[test]
    fn test_display_command_pop_clip() {
        let cmd = DisplayCommand::PopClip;
        assert_eq!(cmd, DisplayCommand::PopClip);
    }

    #[test]
    fn test_display_list_type() {
        let list: DisplayList = vec![
            DisplayCommand::ClipRect(LayoutRect::new(0.0, 0.0, 100.0, 100.0)),
            DisplayCommand::FillRect {
                rect: LayoutRect::new(0.0, 0.0, 100.0, 100.0),
                color: (255, 255, 255, 255),
            },
            DisplayCommand::PopClip,
        ];
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_default_paint_single_box() {
        let painter = DefaultPainter;
        let layout = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        let display_list = painter.paint(&layout);

        // Should have: ClipRect, FillRect, PopClip
        assert_eq!(display_list.len(), 3);
        assert!(matches!(display_list[0], DisplayCommand::ClipRect(_)));
        assert!(matches!(display_list[1], DisplayCommand::FillRect { .. }));
        assert!(matches!(display_list[2], DisplayCommand::PopClip));
    }

    #[test]
    fn test_default_paint_nested_boxes() {
        let painter = DefaultPainter;
        let mut parent = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        parent.add_child(LayoutBox::new(
            LayoutRect::new(10.0, 10.0, 50.0, 50.0),
            BoxType::Inline,
        ));

        let display_list = painter.paint(&parent);

        // Should have: ClipRect(parent), FillRect(parent), ClipRect(child), FillRect(child), PopClip(child), PopClip(parent)
        assert_eq!(display_list.len(), 6);
    }

    #[test]
    fn test_default_paint_multiple_children() {
        let painter = DefaultPainter;
        let mut parent = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        parent.add_child(LayoutBox::new(
            LayoutRect::new(10.0, 10.0, 50.0, 50.0),
            BoxType::Inline,
        ));
        parent.add_child(LayoutBox::new(
            LayoutRect::new(10.0, 70.0, 50.0, 20.0),
            BoxType::Inline,
        ));

        let display_list = painter.paint(&parent);

        // Parent: ClipRect, FillRect
        // Child 1: ClipRect, FillRect, PopClip
        // Child 2: ClipRect, FillRect, PopClip
        // Parent: PopClip
        assert_eq!(display_list.len(), 9);
    }

    #[test]
    fn test_default_paint_deep_tree() {
        let painter = DefaultPainter;
        let mut root = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        let mut child1 = LayoutBox::new(LayoutRect::new(10.0, 10.0, 80.0, 80.0), BoxType::Block);
        child1.add_child(LayoutBox::new(
            LayoutRect::new(20.0, 20.0, 30.0, 30.0),
            BoxType::Inline,
        ));
        root.add_child(child1);

        let display_list = painter.paint(&root);

        // Each box contributes: ClipRect, FillRect, PopClip
        // Total: 3 boxes * 3 commands = 9
        assert_eq!(display_list.len(), 9);
    }

    #[test]
    fn test_box_type_colors() {
        let painter = DefaultPainter;

        let block = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Block);
        let block_list = painter.paint(&block);
        if let DisplayCommand::FillRect { color, .. } = &block_list[1] {
            assert_eq!(*color, (200, 200, 200, 255));
        } else {
            panic!("Expected FillRect command");
        }

        let inline = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Inline);
        let inline_list = painter.paint(&inline);
        if let DisplayCommand::FillRect { color, .. } = &inline_list[1] {
            assert_eq!(*color, (255, 255, 255, 255));
        } else {
            panic!("Expected FillRect command");
        }

        let anonymous = LayoutBox::new(LayoutRect::new(0.0, 0.0, 100.0, 100.0), BoxType::Anonymous);
        let anonymous_list = painter.paint(&anonymous);
        if let DisplayCommand::FillRect { color, .. } = &anonymous_list[1] {
            assert_eq!(*color, (240, 240, 240, 255));
        } else {
            panic!("Expected FillRect command");
        }
    }
}
