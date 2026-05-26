//! Style engine for applying CSS stylesheets to DOM trees
//!
//! Implements the StyleEngine trait from INTERFACES.md, which applies CSS
//! stylesheets to a DOM tree to produce a styled tree with computed styles.

use crate::css::{CssValue, Declaration, Rule, Selector, Stylesheet};
use crate::html::{Dom, Node, NodeKind};
use std::collections::HashMap;

/// A styled DOM node with computed CSS styles.
#[derive(Debug, Clone, PartialEq)]
pub struct StyledNode<'dom> {
    /// Reference to the original DOM node
    pub node: &'dom Node,
    /// Computed CSS styles for this node
    pub styles: HashMap<String, CssValue>,
    /// Styled children of this node
    pub children: Vec<StyledNode<'dom>>,
}

/// Specificity of a CSS selector for style cascade ordering.
///
/// Higher specificity values take precedence. Specificity is calculated as:
/// - (id_count, class_count, tag_count)
/// - Compared lexicographically: id > class > tag
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Specificity(u32, u32, u32);

impl Specificity {
    /// Calculate specificity for a selector.
    fn from_selector(selector: &Selector) -> Self {
        let id_count = if selector.id.is_some() { 1 } else { 0 };
        let class_count = if selector.class.is_some() { 1 } else { 0 };
        let tag_count = if selector.tag.is_some() { 1 } else { 0 };
        Specificity(id_count, class_count, tag_count)
    }

    /// Add two specificities together (for combined selectors).
    fn add(self, other: Specificity) -> Self {
        Specificity(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

/// Style engine that applies CSS stylesheets to DOM trees.
pub struct StyleEngine;

impl StyleEngine {
    /// Create a new style engine.
    pub fn new() -> Self {
        StyleEngine
    }

    /// Check if a selector matches a DOM node.
    fn matches_selector(node: &Node, selector: &Selector) -> bool {
        match &node.kind {
            NodeKind::Element { tag, attrs } => {
                // Check tag match
                if let Some(selector_tag) = &selector.tag {
                    if tag != selector_tag {
                        return false;
                    }
                }

                // Check id match
                if let Some(selector_id) = &selector.id {
                    let node_id = attrs.iter().find(|(k, _)| k == "id").map(|(_, v)| v);
                    if node_id != Some(selector_id) {
                        return false;
                    }
                }

                // Check class match
                if let Some(selector_class) = &selector.class {
                    let node_classes = attrs.iter().find(|(k, _)| k == "class").map(|(_, v)| v);
                    if node_classes.is_none() {
                        return false;
                    }
                    let classes = node_classes.unwrap().split_whitespace().collect::<Vec<_>>();
                    if !classes.contains(&selector_class.as_str()) {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    /// Extract the id attribute from a DOM node.
    fn get_node_id(node: &Node) -> Option<String> {
        match &node.kind {
            NodeKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone()),
            _ => None,
        }
    }

    /// Extract the class attribute from a DOM node as a vector of class names.
    fn get_node_classes(node: &Node) -> Vec<String> {
        match &node.kind {
            NodeKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == "class")
                .map(|(_, v)| v.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// Extract the tag name from a DOM node.
    fn get_node_tag(node: &Node) -> Option<String> {
        match &node.kind {
            NodeKind::Element { tag, .. } => Some(tag.clone()),
            _ => None,
        }
    }

    /// Apply matching rules to a node, collecting declarations with specificity.
    fn apply_rules<'a>(node: &Node, rules: &[&'a Rule]) -> Vec<((u32, u32, u32), &'a Declaration)> {
        let mut matched = Vec::new();

        for rule in rules {
            // Check if any selector in this rule matches the node
            for selector in &rule.selectors {
                if Self::matches_selector(node, selector) {
                    let specificity = Specificity::from_selector(selector);
                    for declaration in &rule.declarations {
                        matched.push(((specificity.0, specificity.1, specificity.2), declaration));
                    }
                    // Only need one selector to match for the rule to apply
                    break;
                }
            }
        }

        matched
    }

    /// Apply inline styles from the style attribute.
    fn apply_inline_styles(node: &Node, styles: &mut HashMap<String, CssValue>) {
        match &node.kind {
            NodeKind::Element { attrs, .. } => {
                if let Some(style_value) = attrs.iter().find(|(k, _)| k == "style").map(|(_, v)| v)
                {
                    // Parse inline style: "property: value; property: value;"
                    for part in style_value.split(';') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        if let Some((property, value)) = part.split_once(':') {
                            let property = property.trim().to_string();
                            let value_str = value.trim();
                            // Parse the value as a CSS value
                            let css_value = Self::parse_css_value(value_str);
                            // Inline styles have highest specificity (1,0,0)
                            styles.insert(property, css_value);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Parse a simple CSS value string into a CssValue.
    fn parse_css_value(s: &str) -> CssValue {
        let s = s.trim();

        // Try to parse as a color
        if s.starts_with('#') {
            if let Some((r, g, b, a)) = Self::parse_hex_color(s) {
                return CssValue::Color(r, g, b, a);
            }
        }

        // Try to parse as a named color
        if let Some((r, g, b, a)) = Self::parse_named_color(s) {
            return CssValue::Color(r, g, b, a);
        }

        // Try to parse as a length
        if let Some((num, unit)) = Self::parse_length(s) {
            return CssValue::Length(num, unit);
        }

        // Try to parse as a percentage
        if let Some(num_str) = s.strip_suffix('%') {
            if let Ok(num) = num_str.parse::<f32>() {
                return CssValue::Percentage(num);
            }
        }

        // Try to parse as a number
        if let Ok(num) = s.parse::<f32>() {
            return CssValue::Number(num);
        }

        // Default to keyword
        CssValue::Keyword(s.to_string())
    }

    /// Parse a hex color string.
    fn parse_hex_color(s: &str) -> Option<(u8, u8, u8, u8)> {
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                let r = u8::from_str_radix(&s[0..1], 16).ok()?;
                let g = u8::from_str_radix(&s[1..2], 16).ok()?;
                let b = u8::from_str_radix(&s[2..3], 16).ok()?;
                Some((r * 17, g * 17, b * 17, 255))
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some((r, g, b, 255))
            }
            _ => None,
        }
    }

    /// Parse a named color.
    fn parse_named_color(name: &str) -> Option<(u8, u8, u8, u8)> {
        match name.to_lowercase().as_str() {
            "black" => Some((0, 0, 0, 255)),
            "white" => Some((255, 255, 255, 255)),
            "red" => Some((255, 0, 0, 255)),
            "green" => Some((0, 128, 0, 255)),
            "blue" => Some((0, 0, 255, 255)),
            "yellow" => Some((255, 255, 0, 255)),
            "cyan" => Some((0, 255, 255, 255)),
            "magenta" => Some((255, 0, 255, 255)),
            "gray" | "grey" => Some((128, 128, 128, 255)),
            "transparent" => Some((0, 0, 0, 0)),
            _ => None,
        }
    }

    /// Parse a length value with unit.
    fn parse_length(s: &str) -> Option<(f32, crate::css::LengthUnit)> {
        if let Some(num_str) = s.strip_suffix("px") {
            let num: f32 = num_str.parse().ok()?;
            Some((num, crate::css::LengthUnit::Px))
        } else if let Some(num_str) = s.strip_suffix("em") {
            let num: f32 = num_str.parse().ok()?;
            Some((num, crate::css::LengthUnit::Em))
        } else if let Some(num_str) = s.strip_suffix("rem") {
            let num: f32 = num_str.parse().ok()?;
            Some((num, crate::css::LengthUnit::Rem))
        } else if let Some(num_str) = s.strip_suffix("vh") {
            let num: f32 = num_str.parse().ok()?;
            Some((num, crate::css::LengthUnit::Vh))
        } else if let Some(num_str) = s.strip_suffix("vw") {
            let num: f32 = num_str.parse().ok()?;
            Some((num, crate::css::LengthUnit::Vw))
        } else {
            None
        }
    }

    /// Recursively style a DOM tree.
    fn style_tree<'dom>(node: &'dom Node, stylesheets: &[Stylesheet]) -> StyledNode<'dom> {
        // Collect all rules from all stylesheets
        let mut all_rules: Vec<&Rule> = Vec::new();
        for stylesheet in stylesheets {
            for rule in &stylesheet.rules {
                all_rules.push(rule);
            }
        }

        // Apply matching rules to this node
        let matched_rules = Self::apply_rules(node, &all_rules);

        // Sort by specificity (lower specificity first, so higher overrides lower)
        let mut matched_rules: Vec<_> = matched_rules.into_iter().collect();
        matched_rules.sort_by_key(|(spec, _)| *spec);

        // Build style map (later rules override earlier ones of same specificity)
        let mut styles = HashMap::new();
        for (_, declaration) in matched_rules {
            styles.insert(declaration.property.clone(), declaration.value.clone());
        }

        // Apply inline styles (highest priority)
        Self::apply_inline_styles(node, &mut styles);

        // Recursively style children
        let children: Vec<StyledNode<'dom>> = node
            .children
            .iter()
            .map(|child| Self::style_tree(child, stylesheets))
            .collect();

        StyledNode {
            node,
            styles,
            children,
        }
    }
}

/// Trait for applying CSS stylesheets to DOM trees.
///
/// This is the main interface defined in INTERFACES.md for the style engine.
pub trait StyleEngineTrait {
    /// Apply stylesheets to a DOM tree, producing a styled tree.
    fn style<'dom>(&self, dom: &'dom Dom, stylesheets: &[Stylesheet]) -> StyledNode<'dom>;
}

impl StyleEngineTrait for StyleEngine {
    fn style<'dom>(&self, dom: &'dom Dom, stylesheets: &[Stylesheet]) -> StyledNode<'dom> {
        StyleEngine::style_tree(&dom.root, stylesheets)
    }
}

impl Default for StyleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::{CssParser, CssParserImpl, LengthUnit};
    use crate::html::{DefaultHtmlParser, HtmlParser, NodeKind};

    fn find_styled_node<'a>(
        node: &'a StyledNode<'a>,
        tag_name: &str,
    ) -> Option<&'a StyledNode<'a>> {
        if let NodeKind::Element { tag, .. } = &node.node.kind {
            let is_match = tag == tag_name;
            if is_match {
                return Some(node);
            }
        }
        for child in &node.children {
            if let Some(found) = find_styled_node(child, tag_name) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_specificity() {
        let tag_only = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };
        let class_only = Selector {
            tag: None,
            id: None,
            class: Some("foo".to_string()),
        };
        let id_only = Selector {
            tag: None,
            id: Some("bar".to_string()),
            class: None,
        };

        let spec_tag = Specificity::from_selector(&tag_only);
        let spec_class = Specificity::from_selector(&class_only);
        let spec_id = Specificity::from_selector(&id_only);

        assert!(spec_id > spec_class);
        assert!(spec_class > spec_tag);
        assert!(spec_tag < spec_class);
    }

    #[test]
    fn test_matches_selector_tag() {
        let node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        let selector = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };

        assert!(StyleEngine::matches_selector(&node, &selector));

        let wrong_selector = Selector {
            tag: Some("span".to_string()),
            id: None,
            class: None,
        };

        assert!(!StyleEngine::matches_selector(&node, &wrong_selector));
    }

    #[test]
    fn test_matches_selector_id() {
        let node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("id".to_string(), "my-id".to_string())],
            },
            children: vec![],
        };

        let selector = Selector {
            tag: None,
            id: Some("my-id".to_string()),
            class: None,
        };

        assert!(StyleEngine::matches_selector(&node, &selector));

        let wrong_selector = Selector {
            tag: None,
            id: Some("other-id".to_string()),
            class: None,
        };

        assert!(!StyleEngine::matches_selector(&node, &wrong_selector));
    }

    #[test]
    fn test_matches_selector_class() {
        let node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "foo bar".to_string())],
            },
            children: vec![],
        };

        let selector = Selector {
            tag: None,
            id: None,
            class: Some("foo".to_string()),
        };

        assert!(StyleEngine::matches_selector(&node, &selector));

        let selector2 = Selector {
            tag: None,
            id: None,
            class: Some("bar".to_string()),
        };

        assert!(StyleEngine::matches_selector(&node, &selector2));

        let wrong_selector = Selector {
            tag: None,
            id: None,
            class: Some("baz".to_string()),
        };

        assert!(!StyleEngine::matches_selector(&node, &wrong_selector));
    }

    #[test]
    fn test_get_node_id() {
        let node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("id".to_string(), "my-id".to_string())],
            },
            children: vec![],
        };

        assert_eq!(StyleEngine::get_node_id(&node), Some("my-id".to_string()));

        let node_no_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        assert_eq!(StyleEngine::get_node_id(&node_no_id), None);
    }

    #[test]
    fn test_get_node_classes() {
        let node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "foo bar baz".to_string())],
            },
            children: vec![],
        };

        let classes = StyleEngine::get_node_classes(&node);
        assert_eq!(classes, vec!["foo", "bar", "baz"]);

        let node_no_class = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        assert!(StyleEngine::get_node_classes(&node_no_class).is_empty());
    }

    #[test]
    fn test_parse_css_value_color() {
        let value = StyleEngine::parse_css_value("#ff0000");
        assert_eq!(value, CssValue::Color(255, 0, 0, 255));

        let value = StyleEngine::parse_css_value("#f00");
        assert_eq!(value, CssValue::Color(255, 0, 0, 255));

        let value = StyleEngine::parse_css_value("red");
        assert_eq!(value, CssValue::Color(255, 0, 0, 255));
    }

    #[test]
    fn test_parse_css_value_length() {
        let value = StyleEngine::parse_css_value("10px");
        assert_eq!(value, CssValue::Length(10.0, LengthUnit::Px));

        let value = StyleEngine::parse_css_value("1.5em");
        assert_eq!(value, CssValue::Length(1.5, LengthUnit::Em));
    }

    #[test]
    fn test_parse_css_value_percentage() {
        let value = StyleEngine::parse_css_value("50%");
        assert_eq!(value, CssValue::Percentage(50.0));
    }

    #[test]
    fn test_parse_css_value_number() {
        let value = StyleEngine::parse_css_value("42");
        assert_eq!(value, CssValue::Number(42.0));
    }

    #[test]
    fn test_parse_css_value_keyword() {
        let value = StyleEngine::parse_css_value("block");
        assert_eq!(value, CssValue::Keyword("block".to_string()));
    }

    #[test]
    fn test_style_engine_simple() {
        let html = r#"<div id="test">Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css = r#"div { color: red; }"#;
        let stylesheet = CssParserImpl::parse(css).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet]);

        assert_eq!(styled.node.kind, dom.root.kind);
        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert!(div.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_id_selector() {
        let html = r#"<div id="test">Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css = r#"#test { color: blue; }"#;
        let stylesheet = CssParserImpl::parse(css).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet]);

        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert!(div.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_class_selector() {
        let html = r#"<div class="foo">Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css = r#".foo { color: green; }"#;
        let stylesheet = CssParserImpl::parse(css).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet]);

        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert!(div.styles.contains_key("color"));
    }

    #[test]
    fn test_style_engine_inline_styles() {
        let html = r#"<div style="color: red; background: blue;">Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[]);

        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert!(div.styles.contains_key("color"));
        assert!(div.styles.contains_key("background"));
    }

    #[test]
    fn test_style_engine_cascade() {
        let html = r#"<div id="test" class="foo">Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css = r#"
            div { color: red; }
            .foo { color: green; }
            #test { color: blue; }
        "#;
        let stylesheet = CssParserImpl::parse(css).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet]);

        // ID selector has highest specificity, so color should be blue
        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert_eq!(
            div.styles.get("color"),
            Some(&CssValue::Color(0, 0, 255, 255))
        );
    }

    #[test]
    fn test_style_engine_multiple_stylesheets() {
        let html = r#"<div>Hello</div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css1 = r#"div { color: red; }"#;
        let css2 = r#"div { background: blue; }"#;

        let stylesheet1 = CssParserImpl::parse(css1).unwrap();
        let stylesheet2 = CssParserImpl::parse(css2).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet1, stylesheet2]);

        let div = find_styled_node(&styled, "div").expect("div element not found");
        assert!(div.styles.contains_key("color"));
        assert!(div.styles.contains_key("background"));
    }

    #[test]
    fn test_style_engine_nested_elements() {
        let html = r#"<div><p>Hello</p></div>"#;
        let dom = DefaultHtmlParser::parse(html).unwrap();

        let css = r#"
            div { color: red; }
            p { color: blue; }
        "#;
        let stylesheet = CssParserImpl::parse(css).unwrap();

        let engine = StyleEngine::new();
        let styled = engine.style(&dom, &[stylesheet]);

        let div = find_styled_node(&styled, "div").expect("div element not found");
        let p = find_styled_node(&styled, "p").expect("p element not found");
        assert_eq!(div.children.len(), 1);
        assert!(div.styles.contains_key("color"));
        assert!(p.styles.contains_key("color"));
    }
}
