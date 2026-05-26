//! Layout engine for the browser
//!
//! Implements block and inline layout algorithms following CSS specification.
//! Handles text wrapping, line breaking, and box model calculations.

use crate::css::{CssValue, LengthUnit, Selector, Stylesheet};
use crate::html::{Node, NodeKind};
use std::collections::HashMap;

// ════════════════════════════
// Layout Types (from INTERFACES.md)
// ════════════════════════════

/// A rectangle representing the position and size of a layout box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        LayoutRect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

/// The type of layout box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Anonymous,
}

/// A layout box representing a positioned element in the layout tree.
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

    pub fn with_children(mut self, children: Vec<LayoutBox>) -> Self {
        self.children = children;
        self
    }
}

// ════════════════════════════
// Styled Node (for style matching)
// ════════════════════════════

/// A DOM node with its computed styles.
#[derive(Debug, Clone)]
pub struct StyledNode<'dom> {
    pub node: &'dom Node,
    pub styles: HashMap<String, CssValue>,
    pub children: Vec<StyledNode<'dom>>,
}

impl<'dom> StyledNode<'dom> {
    pub fn new(node: &'dom Node) -> Self {
        StyledNode {
            node,
            styles: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Get a style value by property name.
    pub fn get_value(&self, property: &str) -> Option<&CssValue> {
        self.styles.get(property)
    }

    /// Get the display property (defaults to 'inline').
    pub fn display(&self) -> Display {
        match self.get_value("display") {
            Some(CssValue::Keyword(k)) => match k.as_str() {
                "block" => Display::Block,
                "inline" => Display::Inline,
                "inline-block" => Display::InlineBlock,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    /// Get the computed width in pixels.
    pub fn width(&self, containing_width: f32) -> f32 {
        match self.get_value("width") {
            Some(CssValue::Length(val, LengthUnit::Px)) => *val,
            Some(CssValue::Percentage(pct)) => containing_width * pct / 100.0,
            Some(CssValue::Length(val, LengthUnit::Em)) => *val * 16.0, // Assume 16px base
            Some(CssValue::Length(val, LengthUnit::Rem)) => *val * 16.0,
            Some(CssValue::Length(val, LengthUnit::Vw)) => containing_width * val / 100.0,
            Some(CssValue::Length(val, LengthUnit::Vh)) => *val * 16.0, // Simplified
            _ => containing_width,                                      // Default to full width
        }
    }

    /// Get the margin values.
    pub fn margin(&self) -> (f32, f32, f32, f32) {
        let top = self.get_length("margin-top", 0.0);
        let right = self.get_length("margin-right", 0.0);
        let bottom = self.get_length("margin-bottom", 0.0);
        let left = self.get_length("margin-left", 0.0);
        (top, right, bottom, left)
    }

    /// Get the padding values.
    pub fn padding(&self) -> (f32, f32, f32, f32) {
        let top = self.get_length("padding-top", 0.0);
        let right = self.get_length("padding-right", 0.0);
        let bottom = self.get_length("padding-bottom", 0.0);
        let left = self.get_length("padding-left", 0.0);
        (top, right, bottom, left)
    }

    /// Get a length value in pixels.
    fn get_length(&self, property: &str, default: f32) -> f32 {
        match self.get_value(property) {
            Some(CssValue::Length(val, LengthUnit::Px)) => *val,
            Some(CssValue::Length(val, LengthUnit::Em)) => *val * 16.0,
            Some(CssValue::Length(val, LengthUnit::Rem)) => *val * 16.0,
            Some(CssValue::Percentage(pct)) => 16.0 * pct / 100.0, // Simplified
            Some(CssValue::Number(n)) => *n,
            _ => default,
        }
    }

    /// Get the font size in pixels.
    pub fn font_size(&self) -> f32 {
        self.get_length("font-size", 16.0)
    }

    /// Get the line height.
    pub fn line_height(&self) -> f32 {
        match self.get_value("line-height") {
            Some(CssValue::Number(n)) => self.font_size() * n,
            Some(CssValue::Length(val, LengthUnit::Px)) => *val,
            Some(CssValue::Length(val, LengthUnit::Em)) => self.font_size() * val,
            _ => self.font_size() * 1.2, // Default line-height
        }
    }
}

/// Display type for CSS display property.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    None,
}

// ════════════════════════════
// Style Engine
// ════════════════════════════

/// Style engine for applying CSS to DOM nodes.
pub struct StyleEngine;

impl StyleEngine {
    /// Apply stylesheets to a DOM tree, producing a styled tree.
    pub fn style<'dom>(&self, dom: &'dom Node, stylesheets: &[Stylesheet]) -> StyledNode<'dom> {
        let mut styled = StyledNode::new(dom);

        // Apply matching rules
        for stylesheet in stylesheets {
            for rule in &stylesheet.rules {
                if Self::selector_matches(&rule.selectors, dom) {
                    for declaration in &rule.declarations {
                        styled
                            .styles
                            .insert(declaration.property.clone(), declaration.value.clone());
                    }
                }
            }
        }

        // Recursively style children
        for child in &dom.children {
            styled.children.push(self.style(child, stylesheets));
        }

        styled
    }

    /// Check if any selector matches a node.
    fn selector_matches(selectors: &[Selector], node: &Node) -> bool {
        selectors
            .iter()
            .any(|sel| Self::single_selector_matches(sel, node))
    }

    /// Check if a single selector matches a node.
    fn single_selector_matches(selector: &Selector, node: &Node) -> bool {
        match &node.kind {
            NodeKind::Element { tag, attrs } => {
                // Check tag
                if let Some(ref sel_tag) = selector.tag {
                    if sel_tag != tag {
                        return false;
                    }
                }

                // Check id
                if let Some(ref sel_id) = selector.id {
                    let id_attr = attrs.iter().find(|(k, _)| k == "id");
                    match id_attr {
                        Some((_, val)) if val == sel_id => {}
                        _ => return false,
                    }
                }

                // Check class
                if let Some(ref sel_class) = selector.class {
                    let class_attr = attrs.iter().find(|(k, _)| k == "class");
                    match class_attr {
                        Some((_, val)) if val.contains(sel_class) => {}
                        _ => return false,
                    }
                }

                true
            }
            _ => false,
        }
    }
}

// ════════════════════════════
// Layout Engine
// ════════════════════════════

/// Layout engine for computing positions and sizes.
pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute positions and sizes for every node.
    pub fn layout(&self, styled: &StyledNode<'_>, containing_width: f32) -> LayoutBox {
        let mut root = LayoutBox::new(LayoutRect::zero(), BoxType::Block);

        match styled.display() {
            Display::Block => {
                root = self.layout_block(styled, containing_width);
            }
            Display::Inline => {
                root = self.layout_inline(styled, containing_width);
            }
            Display::InlineBlock => {
                // Treat as block for now
                root = self.layout_block(styled, containing_width);
            }
            Display::None => {
                // Don't layout
            }
        }

        root
    }

    /// Layout a block-level element.
    fn layout_block(&self, styled: &StyledNode<'_>, containing_width: f32) -> LayoutBox {
        let (margin_top, margin_right, margin_bottom, margin_left) = styled.margin();
        let (padding_top, padding_right, padding_bottom, padding_left) = styled.padding();

        // Calculate width
        let content_width = styled.width(containing_width);
        let _total_width =
            margin_left + padding_left + content_width + padding_right + margin_right;

        let mut rect = LayoutRect::new(
            margin_left,
            margin_top,
            content_width,
            0.0, // Height will be calculated
        );

        let mut children = Vec::new();
        let mut cursor_y = padding_top;

        // Layout children
        for child in &styled.children {
            if child.display() == Display::None {
                continue;
            }

            let child_layout = self.layout(child, content_width);

            // Position child
            let mut child_rect = child_layout.rect;
            child_rect.x = padding_left;
            child_rect.y = cursor_y;

            children.push(LayoutBox {
                rect: child_rect,
                box_type: child_layout.box_type,
                children: child_layout.children,
            });

            cursor_y += child_rect.height;
        }

        // Calculate height
        let content_height = cursor_y + padding_bottom;
        rect.height = content_height + margin_top + margin_bottom;

        LayoutBox::new(rect, BoxType::Block).with_children(children)
    }

    /// Layout an inline-level element with text wrapping.
    fn layout_inline(&self, styled: &StyledNode<'_>, containing_width: f32) -> LayoutBox {
        let (margin_top, _margin_right, margin_bottom, margin_left) = styled.margin();
        let (padding_top, padding_right, padding_bottom, padding_left) = styled.padding();

        let content_width = styled.width(containing_width);
        let font_size = styled.font_size();
        let line_height = styled.line_height();

        let mut rect = LayoutRect::new(margin_left, margin_top, content_width, 0.0);

        let mut children = Vec::new();
        let mut cursor_x = padding_left;
        let mut cursor_y = padding_top;
        let mut max_line_height = line_height;

        // Layout children (text and inline boxes)
        for child in &styled.children {
            if child.display() == Display::None {
                continue;
            }

            match &child.node.kind {
                NodeKind::Text(text) => {
                    // Layout text with wrapping
                    let text_layouts = self.layout_text(
                        text,
                        font_size,
                        line_height,
                        content_width - padding_left - padding_right,
                        cursor_x,
                    );

                    for text_layout in text_layouts {
                        if text_layout.rect.x == padding_left {
                            // New line
                            cursor_x = padding_left;
                            cursor_y += max_line_height;
                            max_line_height = line_height;
                        }

                        let mut text_rect = text_layout.rect;
                        text_rect.x = cursor_x;
                        text_rect.y = cursor_y;

                        children.push(LayoutBox {
                            rect: text_rect,
                            box_type: BoxType::Inline,
                            children: Vec::new(),
                        });

                        cursor_x += text_rect.width;
                        max_line_height = max_line_height.max(text_rect.height);
                    }
                }
                _ => {
                    // Layout inline child
                    let child_layout = self.layout(child, content_width);

                    // Check if we need to wrap
                    if cursor_x + child_layout.rect.width > content_width - padding_right {
                        cursor_x = padding_left;
                        cursor_y += max_line_height;
                        max_line_height = line_height;
                    }

                    let mut child_rect = child_layout.rect;
                    child_rect.x = cursor_x;
                    child_rect.y = cursor_y;

                    children.push(LayoutBox {
                        rect: child_rect,
                        box_type: child_layout.box_type,
                        children: child_layout.children,
                    });

                    cursor_x += child_rect.width;
                    max_line_height = max_line_height.max(child_rect.height);
                }
            }
        }

        // Calculate height
        let content_height = cursor_y + max_line_height + padding_bottom;
        rect.height = content_height + margin_top + margin_bottom;

        LayoutBox::new(rect, BoxType::Inline).with_children(children)
    }

    /// Layout text with line breaking and wrapping.
    fn layout_text(
        &self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: f32,
        start_x: f32,
    ) -> Vec<LayoutBox> {
        let mut layouts = Vec::new();

        // Simple character-based wrapping (approximate)
        let char_width = font_size * 0.6; // Approximate character width
        let chars_per_line = (max_width / char_width) as usize;

        if chars_per_line == 0 {
            // No space for text
            return layouts;
        }

        let mut cursor_x = start_x;
        let mut current_line = String::new();
        let mut line_start = true;

        for ch in text.chars() {
            if ch == '\n' {
                // Explicit line break
                if !current_line.is_empty() {
                    let width = current_line.len() as f32 * char_width;
                    layouts.push(LayoutBox::new(
                        LayoutRect::new(0.0, 0.0, width, line_height),
                        BoxType::Inline,
                    ));
                    current_line.clear();
                }
                cursor_x = 0.0;
                line_start = true;
                continue;
            }

            if ch.is_whitespace() {
                // Word boundary
                if !current_line.is_empty() {
                    let width = current_line.len() as f32 * char_width;
                    layouts.push(LayoutBox::new(
                        LayoutRect::new(
                            if line_start { 0.0 } else { cursor_x },
                            0.0,
                            width,
                            line_height,
                        ),
                        BoxType::Inline,
                    ));
                    cursor_x += width;
                    current_line.clear();
                    line_start = false;
                }
                cursor_x += char_width; // Space character
                continue;
            }

            current_line.push(ch);

            // Check if we need to wrap
            let current_width = current_line.len() as f32 * char_width;
            if cursor_x + current_width > max_width && !current_line.is_empty() {
                // Wrap the word
                let width = current_line.len() as f32 * char_width;
                layouts.push(LayoutBox::new(
                    LayoutRect::new(0.0, 0.0, width, line_height),
                    BoxType::Inline,
                ));
                current_line.clear();
                cursor_x = 0.0;
                line_start = true;
            }
        }

        // Add remaining text
        if !current_line.is_empty() {
            let width = current_line.len() as f32 * char_width;
            layouts.push(LayoutBox::new(
                LayoutRect::new(
                    if line_start { 0.0 } else { cursor_x },
                    0.0,
                    width,
                    line_height,
                ),
                BoxType::Inline,
            ));
        }

        layouts
    }
}

// ════════════════════════════
// Trait Implementations (from INTERFACES.md)
// ════════════════════════════

/// Trait for layout engines.
pub trait LayoutEngineTrait {
    /// Compute positions and sizes for every node.
    fn layout(&self, styled: &StyledNode<'_>, containing_width: f32) -> LayoutBox;
}

impl LayoutEngineTrait for LayoutEngine {
    fn layout(&self, styled: &StyledNode<'_>, containing_width: f32) -> LayoutBox {
        self.layout(styled, containing_width)
    }
}

/// Trait for style engines.
pub trait StyleEngineTrait {
    /// Apply stylesheets to a DOM tree, producing a styled tree.
    fn style<'dom>(&self, dom: &'dom Node, stylesheets: &[Stylesheet]) -> StyledNode<'dom>;
}

impl StyleEngineTrait for StyleEngine {
    fn style<'dom>(&self, dom: &'dom Node, stylesheets: &[Stylesheet]) -> StyledNode<'dom> {
        self.style(dom, stylesheets)
    }
}

// ════════════════════════════
// Tests
// ════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::CssParser;

    fn parse_css(css: &str) -> Stylesheet {
        crate::css::CssParserImpl::parse(css).unwrap()
    }

    #[test]
    fn test_layout_rect_creation() {
        let rect = LayoutRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_layout_rect_zero() {
        let rect = LayoutRect::zero();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }

    #[test]
    fn test_layout_box_creation() {
        let rect = LayoutRect::new(0.0, 0.0, 100.0, 50.0);
        let box_ = LayoutBox::new(rect, BoxType::Block);
        assert_eq!(box_.rect, rect);
        assert_eq!(box_.box_type, BoxType::Block);
        assert!(box_.children.is_empty());
    }

    #[test]
    fn test_layout_box_with_children() {
        let parent = LayoutBox::new(LayoutRect::zero(), BoxType::Block);
        let child = LayoutBox::new(LayoutRect::new(10.0, 10.0, 50.0, 30.0), BoxType::Inline);
        let parent = parent.with_children(vec![child]);
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_styled_node_creation() {
        let node = Node::new(NodeKind::Text("Hello".to_string()));
        let styled = StyledNode::new(&node);
        assert_eq!(styled.node, &node);
        assert!(styled.styles.is_empty());
        assert!(styled.children.is_empty());
    }

    #[test]
    fn test_styled_node_display_default() {
        let node = Node::new(NodeKind::Text("Hello".to_string()));
        let styled = StyledNode::new(&node);
        assert_eq!(styled.display(), Display::Inline);
    }

    #[test]
    fn test_styled_node_display_block() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        assert_eq!(styled.display(), Display::Block);
    }

    #[test]
    fn test_styled_node_display_none() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled
            .styles
            .insert("display".to_string(), CssValue::Keyword("none".to_string()));
        assert_eq!(styled.display(), Display::None);
    }

    #[test]
    fn test_styled_node_width_pixels() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled
            .styles
            .insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        assert_eq!(styled.width(1000.0), 100.0);
    }

    #[test]
    fn test_styled_node_width_percentage() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled
            .styles
            .insert("width".to_string(), CssValue::Percentage(50.0));
        assert_eq!(styled.width(1000.0), 500.0);
    }

    #[test]
    fn test_styled_node_margin() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled
            .styles
            .insert("margin".to_string(), CssValue::Length(10.0, LengthUnit::Px));
        let (top, right, bottom, left) = styled.margin();
        assert_eq!(top, 0.0); // margin shorthand not implemented yet
        assert_eq!(right, 0.0);
        assert_eq!(bottom, 0.0);
        assert_eq!(left, 0.0);
    }

    #[test]
    fn test_styled_node_font_size() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "font-size".to_string(),
            CssValue::Length(20.0, LengthUnit::Px),
        );
        assert_eq!(styled.font_size(), 20.0);
    }

    #[test]
    fn test_styled_node_line_height() {
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "font-size".to_string(),
            CssValue::Length(16.0, LengthUnit::Px),
        );
        styled
            .styles
            .insert("line-height".to_string(), CssValue::Number(1.5));
        assert_eq!(styled.line_height(), 24.0);
    }

    #[test]
    fn test_style_engine_selector_match_tag() {
        let engine = StyleEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let stylesheet = parse_css("div { color: red; }");
        let styled = engine.style(&node, &[stylesheet]);
        assert!(styled.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_selector_match_id() {
        let engine = StyleEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![("id".to_string(), "myid".to_string())],
        });
        let stylesheet = parse_css("#myid { color: blue; }");
        let styled = engine.style(&node, &[stylesheet]);
        assert!(styled.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_selector_match_class() {
        let engine = StyleEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![("class".to_string(), "myclass".to_string())],
        });
        let stylesheet = parse_css(".myclass { color: green; }");
        let styled = engine.style(&node, &[stylesheet]);
        assert!(styled.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_no_match() {
        let engine = StyleEngine;
        let node = Node::new(NodeKind::Element {
            tag: "span".to_string(),
            attrs: vec![],
        });
        let stylesheet = parse_css("div { color: red; }");
        let styled = engine.style(&node, &[stylesheet]);
        assert!(!styled.styles.contains_key("color"));
    }

    #[test]
    fn test_layout_engine_block() {
        let engine = LayoutEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        styled
            .styles
            .insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));

        let layout = engine.layout(&styled, 1000.0);
        assert_eq!(layout.box_type, BoxType::Block);
        assert_eq!(layout.rect.width, 100.0);
    }

    #[test]
    fn test_layout_engine_inline() {
        let engine = LayoutEngine;
        let node = Node::new(NodeKind::Element {
            tag: "span".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "display".to_string(),
            CssValue::Keyword("inline".to_string()),
        );

        let layout = engine.layout(&styled, 1000.0);
        assert_eq!(layout.box_type, BoxType::Inline);
    }

    #[test]
    fn test_layout_engine_none() {
        let engine = LayoutEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled
            .styles
            .insert("display".to_string(), CssValue::Keyword("none".to_string()));

        let layout = engine.layout(&styled, 1000.0);
        assert_eq!(layout.rect.height, 0.0);
    }

    #[test]
    fn test_layout_text_wrapping() {
        let engine = LayoutEngine;
        let text = "Hello World";
        let layouts = engine.layout_text(text, 16.0, 19.2, 50.0, 0.0);
        // Should wrap due to limited width
        assert!(!layouts.is_empty());
    }

    #[test]
    fn test_layout_text_no_wrapping() {
        let engine = LayoutEngine;
        let text = "Hi";
        let layouts = engine.layout_text(text, 16.0, 19.2, 1000.0, 0.0);
        // Should not wrap
        assert_eq!(layouts.len(), 1);
    }

    #[test]
    fn test_layout_text_explicit_newline() {
        let engine = LayoutEngine;
        let text = "Hello\nWorld";
        let layouts = engine.layout_text(text, 16.0, 19.2, 1000.0, 0.0);
        // Should create two lines
        assert!(!layouts.is_empty());
    }

    #[test]
    fn test_layout_block_with_children() {
        let engine = LayoutEngine;
        let mut parent = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let child = Node::new(NodeKind::Element {
            tag: "p".to_string(),
            attrs: vec![],
        });
        parent.children.push(child);

        let mut styled_parent = StyledNode::new(&parent);
        styled_parent.styles.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        styled_parent
            .styles
            .insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));

        let mut styled_child = StyledNode::new(&parent.children[0]);
        styled_child.styles.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        styled_child
            .styles
            .insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));

        styled_parent.children.push(styled_child);

        let layout = engine.layout(&styled_parent, 1000.0);
        assert_eq!(layout.box_type, BoxType::Block);
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn test_layout_engine_trait() {
        let engine = LayoutEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let mut styled = StyledNode::new(&node);
        styled.styles.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );

        let layout: LayoutBox = engine.layout(&styled, 1000.0);
        assert_eq!(layout.box_type, BoxType::Block);
    }

    #[test]
    fn test_style_engine_trait() {
        let engine = StyleEngine;
        let node = Node::new(NodeKind::Element {
            tag: "div".to_string(),
            attrs: vec![],
        });
        let stylesheet = parse_css("div { color: red; }");
        let styled = engine.style(&node, &[stylesheet]);
        assert!(styled.styles.contains_key("color"));
    }
}
