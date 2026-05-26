//! CSS tokenizer and rule parser
//!
//! Implements from-scratch CSS tokenization and parsing for the browser's style system.
#![allow(dead_code)]

use crate::error::Result;
use crate::html::Node;
use std::iter::Peekable;
use std::str::Chars;

/// A parsed stylesheet containing CSS rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// A CSS rule consisting of selectors and declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// A CSS selector with optional tag name, ID, and class.
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub class: Option<String>,
}

impl Selector {
    /// Check if this selector matches a given DOM node.
    pub fn matches_node(&self, node: &Node) -> bool {
        // Check tag name match
        if let Some(ref tag) = self.tag {
            if !matches_tag(node, tag) {
                return false;
            }
        }

        // Check ID match
        if let Some(ref id) = self.id {
            if let Some(node_id) = get_node_id(node) {
                if node_id != *id {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check class match
        if let Some(ref class) = self.class {
            if let Some(node_classes) = get_node_classes(node) {
                if !node_classes.contains(class) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Calculate the specificity of this selector.
    /// Returns a tuple (id_count, class_count, tag_count).
    pub fn specificity(&self) -> (u32, u32, u32) {
        let id_count = if self.id.is_some() { 1 } else { 0 };
        let class_count = if self.class.is_some() { 1 } else { 0 };
        let tag_count = if self.tag.is_some() { 1 } else { 0 };
        (id_count, class_count, tag_count)
    }
}

/// Extract the ID attribute from a DOM node.
fn get_node_id(node: &Node) -> Option<String> {
    match &node.kind {
        crate::html::NodeKind::Element { tag: _, attrs } => {
            for (name, value) in attrs {
                if name == "id" {
                    return Some(value.clone());
                }
            }
            None
        }
        _ => None,
    }
}

/// Extract all class names from a DOM node's class attribute.
fn get_node_classes(node: &Node) -> Option<Vec<String>> {
    match &node.kind {
        crate::html::NodeKind::Element { tag: _, attrs } => {
            for (name, value) in attrs {
                if name == "class" {
                    return Some(value.split_whitespace().map(String::from).collect());
                }
            }
            None
        }
        _ => None,
    }
}

/// Check if a DOM node matches a given tag name.
fn matches_tag(node: &Node, tag: &str) -> bool {
    match &node.kind {
        crate::html::NodeKind::Element {
            tag: node_tag,
            attrs: _,
        } => node_tag == tag,
        _ => false,
    }
}

/// A CSS declaration with a property and parsed value.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: CssValue,
}

/// A parsed CSS value.
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Keyword(String),
    Length(f32, LengthUnit),
    Color(u8, u8, u8, u8),
    Percentage(f32),
    Number(f32),
}

/// CSS length units supported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum LengthUnit {
    Px,
    Em,
    Rem,
    Vh,
    Vw,
}

/// Trait for parsing CSS input into a Stylesheet.
pub trait CssParser {
    fn parse(input: &str) -> Result<Stylesheet>;
}

// ════════════════════════════
// Tokenizer
// ════════════════════════════

#[derive(Debug, Clone, PartialEq)]
enum CssToken {
    Ident(String),
    Hash(String),
    StringLiteral(String),
    Number(f32),
    Delim(char),
    Colon,
    Semicolon,
    Comma,
    Whitespace,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    AtKeyword(String),
    Percentage(f32),
    Dimension(f32, String),
    Function(String),
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '-'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn consume_ident(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if is_ident_char(c) {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s
}

fn consume_digits(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s
}

fn tokenize(input: &str) -> Vec<CssToken> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(&c) = iter.peek() {
        if c.is_whitespace() {
            iter.next();
            while iter.peek().is_some_and(|c| c.is_whitespace()) {
                iter.next();
            }
            tokens.push(CssToken::Whitespace);
        } else if c == '/' {
            iter.next();
            if iter.peek() == Some(&'*') {
                iter.next();
                loop {
                    match iter.next() {
                        Some('*') if iter.peek() == Some(&'/') => {
                            iter.next();
                            break;
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
            } else {
                tokens.push(CssToken::Delim('/'));
            }
        } else if matches!(c, '{' | '}' | '(' | ')' | ':' | ';' | ',') {
            iter.next();
            match c {
                '{' => tokens.push(CssToken::OpenBrace),
                '}' => tokens.push(CssToken::CloseBrace),
                '(' => tokens.push(CssToken::OpenParen),
                ')' => tokens.push(CssToken::CloseParen),
                ':' => tokens.push(CssToken::Colon),
                ';' => tokens.push(CssToken::Semicolon),
                ',' => tokens.push(CssToken::Comma),
                _ => unreachable!(),
            }
        } else if c == '#' {
            iter.next();
            tokens.push(CssToken::Hash(consume_ident(&mut iter)));
        } else if c == '@' {
            iter.next();
            tokens.push(CssToken::AtKeyword(consume_ident(&mut iter)));
        } else if c == '"' || c == '\'' {
            let quote = c;
            iter.next();
            let mut s = String::new();
            while let Some(&ch) = iter.peek() {
                if ch == quote {
                    iter.next();
                    break;
                }
                s.push(ch);
                iter.next();
            }
            tokens.push(CssToken::StringLiteral(s));
        } else if c == '-' {
            iter.next();
            match iter.peek() {
                Some(&next) if next.is_ascii_digit() || next == '.' => {
                    let mut num_str = String::from('-');
                    if next == '.' {
                        num_str.push('.');
                        iter.next();
                    }
                    num_str.push_str(&consume_digits(&mut iter));
                    if iter.peek() == Some(&'.') {
                        num_str.push('.');
                        iter.next();
                        num_str.push_str(&consume_digits(&mut iter));
                    }
                    let n: f32 = num_str.parse().unwrap_or(0.0);
                    if iter.peek().is_some_and(|&c| is_ident_start(c)) {
                        tokens.push(CssToken::Dimension(n, consume_ident(&mut iter)));
                    } else if iter.peek() == Some(&'%') {
                        iter.next();
                        tokens.push(CssToken::Percentage(n));
                    } else {
                        tokens.push(CssToken::Number(n));
                    }
                }
                Some(&next) if is_ident_start(next) => {
                    let name = format!("-{}", consume_ident(&mut iter));
                    if iter.peek() == Some(&'(') {
                        iter.next();
                        tokens.push(CssToken::Function(name));
                    } else {
                        tokens.push(CssToken::Ident(name));
                    }
                }
                _ => {
                    tokens.push(CssToken::Delim('-'));
                }
            }
        } else if c == '+' {
            iter.next();
            tokens.push(CssToken::Delim('+'));
        } else if c.is_ascii_digit() {
            let mut num_str = consume_digits(&mut iter);
            if iter.peek() == Some(&'.') {
                num_str.push('.');
                iter.next();
                num_str.push_str(&consume_digits(&mut iter));
            }
            let n: f32 = num_str.parse().unwrap_or(0.0);
            if iter.peek().is_some_and(|&c| is_ident_start(c)) {
                tokens.push(CssToken::Dimension(n, consume_ident(&mut iter)));
            } else if iter.peek() == Some(&'%') {
                iter.next();
                tokens.push(CssToken::Percentage(n));
            } else {
                tokens.push(CssToken::Number(n));
            }
        } else if is_ident_start(c) {
            let name = consume_ident(&mut iter);
            if iter.peek() == Some(&'(') {
                iter.next();
                tokens.push(CssToken::Function(name));
            } else {
                tokens.push(CssToken::Ident(name));
            }
        } else {
            iter.next();
            tokens.push(CssToken::Delim(c));
        }
    }
    tokens
}

// ════════════════════════════
// CSS Value Parsing
// ════════════════════════════

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

fn parse_named_color(name: &str) -> Option<(u8, u8, u8, u8)> {
    match name.to_lowercase().as_str() {
        "black" => Some((0, 0, 0, 255)),
        "silver" => Some((192, 192, 192, 255)),
        "gray" | "grey" => Some((128, 128, 128, 255)),
        "white" => Some((255, 255, 255, 255)),
        "maroon" => Some((128, 0, 0, 255)),
        "red" => Some((255, 0, 0, 255)),
        "purple" => Some((128, 0, 128, 255)),
        "fuchsia" | "magenta" => Some((255, 0, 255, 255)),
        "green" => Some((0, 128, 0, 255)),
        "lime" => Some((0, 255, 0, 255)),
        "olive" => Some((128, 128, 0, 255)),
        "yellow" => Some((255, 255, 0, 255)),
        "navy" => Some((0, 0, 128, 255)),
        "blue" => Some((0, 0, 255, 255)),
        "teal" => Some((0, 128, 128, 255)),
        "aqua" | "cyan" => Some((0, 255, 255, 255)),
        "orange" => Some((255, 165, 0, 255)),
        "pink" => Some((255, 192, 203, 255)),
        "brown" => Some((165, 42, 42, 255)),
        "indigo" => Some((75, 0, 130, 255)),
        "violet" => Some((238, 130, 238, 255)),
        "gold" => Some((255, 215, 0, 255)),
        "coral" => Some((255, 127, 80, 255)),
        "tomato" => Some((255, 99, 71, 255)),
        "crimson" => Some((220, 20, 60, 255)),
        "salmon" => Some((250, 128, 114, 255)),
        "orchid" => Some((218, 112, 214, 255)),
        "plum" => Some((221, 160, 221, 255)),
        "khaki" => Some((240, 230, 140, 255)),
        "turquoise" => Some((64, 224, 208, 255)),
        "beige" => Some((245, 245, 220, 255)),
        "wheat" => Some((245, 222, 179, 255)),
        "tan" => Some((210, 180, 140, 255)),
        "lavender" => Some((230, 230, 250, 255)),
        "thistle" => Some((216, 191, 216, 255)),
        "honeydew" => Some((240, 255, 240, 255)),
        "azure" => Some((240, 255, 255, 255)),
        "mintcream" => Some((245, 255, 250, 255)),
        "snow" => Some((255, 250, 250, 255)),
        "linen" => Some((250, 240, 230, 255)),
        "ivory" => Some((255, 255, 240, 255)),
        "bisque" => Some((255, 228, 196, 255)),
        "cornsilk" => Some((255, 248, 220, 255)),
        "transparent" => Some((0, 0, 0, 0)),
        _ => None,
    }
}

fn parse_length_unit(s: &str) -> Option<LengthUnit> {
    match s.to_lowercase().as_str() {
        "px" => Some(LengthUnit::Px),
        "em" => Some(LengthUnit::Em),
        "rem" => Some(LengthUnit::Rem),
        "vh" => Some(LengthUnit::Vh),
        "vw" => Some(LengthUnit::Vw),
        _ => None,
    }
}

fn parse_css_value(tokens: &[CssToken], start: usize, end: usize) -> CssValue {
    if start >= end {
        return CssValue::Keyword(String::new());
    }
    let token = &tokens[start];
    if let CssToken::Hash(s) = token {
        if let Some(color) = parse_hex_color(s) {
            return CssValue::Color(color.0, color.1, color.2, color.3);
        }
        return CssValue::Keyword(format!("#{s}"));
    }
    if let CssToken::Dimension(num, unit) = token {
        if let Some(lu) = parse_length_unit(unit) {
            return CssValue::Length(*num, lu);
        }
        return CssValue::Keyword(format!("{num}{unit}"));
    }
    if let CssToken::Percentage(num) = token {
        return CssValue::Percentage(*num);
    }
    if let CssToken::Number(num) = token {
        return CssValue::Number(*num);
    }
    if let CssToken::Ident(s) = token {
        if let Some(color) = parse_named_color(s) {
            return CssValue::Color(color.0, color.1, color.2, color.3);
        }
        return CssValue::Keyword(s.clone());
    }
    CssValue::Keyword(format!("{token:?}"))
}

// ════════════════════════════
// Parser
// ════════════════════════════

fn parse_declarations(tokens: &[CssToken], pos: &mut usize) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    while *pos < tokens.len() {
        match &tokens[*pos] {
            CssToken::CloseBrace => {
                *pos += 1;
                break;
            }
            CssToken::Whitespace | CssToken::Semicolon => {
                *pos += 1;
            }
            CssToken::Ident(property) => {
                let property = property.clone();
                *pos += 1;
                while *pos < tokens.len() && matches!(&tokens[*pos], CssToken::Whitespace) {
                    *pos += 1;
                }
                if *pos >= tokens.len() || !matches!(&tokens[*pos], CssToken::Colon) {
                    skip_to_declaration_end(tokens, pos);
                    continue;
                }
                *pos += 1;
                while *pos < tokens.len() && matches!(&tokens[*pos], CssToken::Whitespace) {
                    *pos += 1;
                }
                let value_start = *pos;
                while *pos < tokens.len() {
                    if matches!(&tokens[*pos], CssToken::Semicolon | CssToken::CloseBrace) {
                        break;
                    }
                    *pos += 1;
                }
                let value = parse_css_value(tokens, value_start, *pos);
                declarations.push(Declaration { property, value });
            }
            _ => {
                skip_to_declaration_end(tokens, pos);
            }
        }
    }
    declarations
}

fn skip_to_declaration_end(tokens: &[CssToken], pos: &mut usize) {
    while *pos < tokens.len() {
        match &tokens[*pos] {
            CssToken::Semicolon => {
                *pos += 1;
                return;
            }
            CssToken::CloseBrace => {
                return;
            }
            _ => {
                *pos += 1;
            }
        }
    }
}

fn skip_whitespace(tokens: &[CssToken], pos: &mut usize) {
    while *pos < tokens.len() && matches!(&tokens[*pos], CssToken::Whitespace) {
        *pos += 1;
    }
}

fn skip_to_brace_end(tokens: &[CssToken], pos: &mut usize) {
    let mut depth = 1;
    while *pos < tokens.len() && depth > 0 {
        match &tokens[*pos] {
            CssToken::OpenBrace => depth += 1,
            CssToken::CloseBrace => depth -= 1,
            _ => {}
        }
        *pos += 1;
    }
}

fn parse_selector_at(tokens: &[CssToken], pos: &mut usize) -> Option<Selector> {
    skip_whitespace(tokens, pos);
    let mut selector = Selector {
        tag: None,
        id: None,
        class: None,
    };
    while *pos < tokens.len() {
        match &tokens[*pos] {
            CssToken::Ident(name) => {
                if selector.tag.is_none() && selector.id.is_none() && selector.class.is_none() {
                    selector.tag = Some(name.clone());
                }
                *pos += 1;
            }
            CssToken::Hash(name) => {
                selector.id = Some(name.clone());
                *pos += 1;
            }
            CssToken::Delim('.') => {
                *pos += 1;
                if let Some(CssToken::Ident(name)) = tokens.get(*pos) {
                    selector.class = Some(name.clone());
                    *pos += 1;
                }
            }
            CssToken::Delim('*') => {
                *pos += 1;
            }
            CssToken::Whitespace => {
                *pos += 1;
                break;
            }
            CssToken::Comma | CssToken::OpenBrace => {
                break;
            }
            _ => {
                *pos += 1;
            }
        }
    }
    if selector.tag.is_some() || selector.id.is_some() || selector.class.is_some() {
        Some(selector)
    } else {
        None
    }
}

fn parse_rule(tokens: &[CssToken], pos: &mut usize) -> Option<Rule> {
    loop {
        skip_whitespace(tokens, pos);
        if *pos >= tokens.len() {
            return None;
        }
        match &tokens[*pos] {
            CssToken::CloseBrace => {
                *pos += 1;
                return None;
            }
            CssToken::OpenBrace => {
                *pos += 1;
                skip_to_brace_end(tokens, pos);
                return None;
            }
            CssToken::Semicolon => {
                *pos += 1;
                return None;
            }
            _ => {}
        }
        let selector_start = *pos;
        if let Some(sel) = parse_selector_at(tokens, pos) {
            let mut selectors = vec![sel];
            loop {
                skip_whitespace(tokens, pos);
                if *pos >= tokens.len() {
                    return None;
                }
                match &tokens[*pos] {
                    CssToken::Comma => {
                        *pos += 1;
                        if let Some(next_sel) = parse_selector_at(tokens, pos) {
                            selectors.push(next_sel);
                        }
                    }
                    CssToken::OpenBrace => {
                        *pos += 1;
                        let declarations = parse_declarations(tokens, pos);
                        return Some(Rule {
                            selectors,
                            declarations,
                        });
                    }
                    _ => {
                        skip_to_brace_end(tokens, pos);
                        return None;
                    }
                }
            }
        } else if *pos == selector_start {
            *pos += 1;
        }
    }
}

/// Parse a full CSS stylesheet from a string.
pub fn parse(input: &str) -> Result<Stylesheet> {
    let tokens = tokenize(input);
    let mut rules = Vec::new();
    let mut pos = 0;
    while pos < tokens.len() {
        skip_whitespace(&tokens, &mut pos);
        if pos >= tokens.len() {
            break;
        }
        if let Some(rule) = parse_rule(&tokens, &mut pos) {
            rules.push(rule);
        }
    }
    Ok(Stylesheet { rules })
}

/// Concrete implementation of the CssParser trait.
pub struct CssParserImpl;

impl CssParser for CssParserImpl {
    fn parse(input: &str) -> Result<Stylesheet> {
        parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let css = "h1 { color: red; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        let rule = &stylesheet.rules[0];
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.selectors[0].tag, Some("h1".to_string()));
        assert_eq!(rule.selectors[0].id, None);
        assert_eq!(rule.selectors[0].class, None);
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, CssValue::Color(255, 0, 0, 255));
    }

    #[test]
    fn test_multiple_rules() {
        let css = "h1 { color: red; } h2 { color: blue; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 2);
        assert_eq!(stylesheet.rules[0].selectors[0].tag, Some("h1".to_string()));
        assert_eq!(stylesheet.rules[1].selectors[0].tag, Some("h2".to_string()));
    }

    #[test]
    fn test_multiple_selectors() {
        let css = "h1, h2 { color: red; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 2);
        assert_eq!(stylesheet.rules[0].selectors[0].tag, Some("h1".to_string()));
        assert_eq!(stylesheet.rules[0].selectors[1].tag, Some("h2".to_string()));
    }

    #[test]
    fn test_class_selector() {
        let css = ".myclass { font-size: 16px; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(
            stylesheet.rules[0].selectors[0].class,
            Some("myclass".to_string())
        );
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Length(16.0, LengthUnit::Px)
        );
    }

    #[test]
    fn test_id_selector() {
        let css = "#myid { margin: 0; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(
            stylesheet.rules[0].selectors[0].id,
            Some("myid".to_string())
        );
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Number(0.0)
        );
    }

    #[test]
    fn test_hex_colors() {
        let css = "a { color: #ff0000; } b { color: #f00; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 2);
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Color(255, 0, 0, 255)
        );
        assert_eq!(
            stylesheet.rules[1].declarations[0].value,
            CssValue::Color(255, 0, 0, 255)
        );
    }

    #[test]
    fn test_lengths() {
        let css = "a { font-size: 10px; line-height: 2em; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Length(10.0, LengthUnit::Px)
        );
        assert_eq!(
            stylesheet.rules[0].declarations[1].value,
            CssValue::Length(2.0, LengthUnit::Em)
        );
    }

    #[test]
    fn test_malformed_css_does_not_panic() {
        let inputs = vec![
            "h1 { color; }",
            "h1 { color }",
            "h1 { ;;; }",
            "!@#$%^",
            "{ { { } } }",
            "h1 { color: red;;; }",
        ];
        for input in inputs {
            let result = parse(input);
            assert!(result.is_ok(), "Failed on input: {input}");
        }
    }

    #[test]
    fn test_empty_input() {
        let stylesheet = parse("").unwrap();
        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn test_multiple_declarations() {
        let css = "h1 { color: red; font-size: 16px; background: blue; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 3);
        assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
        assert_eq!(stylesheet.rules[0].declarations[1].property, "font-size");
        assert_eq!(
            stylesheet.rules[0].declarations[1].value,
            CssValue::Length(16.0, LengthUnit::Px)
        );
        assert_eq!(stylesheet.rules[0].declarations[2].property, "background");
    }

    #[test]
    fn test_tag_with_id_and_class() {
        let css = "div#myid.myclass { color: red; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        let sel = &stylesheet.rules[0].selectors[0];
        assert_eq!(sel.tag, Some("div".to_string()));
        assert_eq!(sel.id, Some("myid".to_string()));
        assert_eq!(sel.class, Some("myclass".to_string()));
    }

    #[test]
    fn test_comment_ignored() {
        let css = "/* comment */ h1 { color: red; /* inner */ }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors[0].tag, Some("h1".to_string()));
    }

    #[test]
    fn test_named_color() {
        let css = "a { color: blue; background: transparent; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Color(0, 0, 255, 255)
        );
        assert_eq!(
            stylesheet.rules[0].declarations[1].value,
            CssValue::Color(0, 0, 0, 0)
        );
    }

    #[test]
    fn test_percentage_value() {
        let css = "div { width: 50%; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Percentage(50.0)
        );
    }

    #[test]
    fn test_css_parser_trait_impl() {
        let css = "p { margin: 10px; }";
        let stylesheet = <CssParserImpl as CssParser>::parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_vendor_prefix_ident() {
        let css = "-moz-box { color: red; }";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(
            stylesheet.rules[0].selectors[0].tag,
            Some("-moz-box".to_string())
        );
    }

    #[test]
    fn test_extra_whitespace() {
        let css = "  h1   {  color  :  red  ;  }  ";
        let stylesheet = parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors[0].tag, Some("h1".to_string()));
        assert_eq!(
            stylesheet.rules[0].declarations[0].value,
            CssValue::Color(255, 0, 0, 255)
        );
    }

    // ════════════════════════════
    // Selector Matching Tests
    // ════════════════════════════

    #[test]
    fn test_tag_selector_match() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };

        let div_node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        let span_node = Node {
            kind: NodeKind::Element {
                tag: "span".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&div_node));
        assert!(!selector.matches_node(&span_node));
    }

    #[test]
    fn test_id_selector_match() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: None,
            id: Some("myid".to_string()),
            class: None,
        };

        let node_with_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("id".to_string(), "myid".to_string())],
            },
            children: vec![],
        };

        let node_without_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        let node_with_different_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("id".to_string(), "otherid".to_string())],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&node_with_id));
        assert!(!selector.matches_node(&node_without_id));
        assert!(!selector.matches_node(&node_with_different_id));
    }

    #[test]
    fn test_class_selector_match() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: None,
            id: None,
            class: Some("myclass".to_string()),
        };

        let node_with_class = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "myclass".to_string())],
            },
            children: vec![],
        };

        let node_without_class = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![],
            },
            children: vec![],
        };

        let node_with_different_class = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "otherclass".to_string())],
            },
            children: vec![],
        };

        let node_with_multiple_classes = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "foo bar myclass baz".to_string())],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&node_with_class));
        assert!(!selector.matches_node(&node_without_class));
        assert!(!selector.matches_node(&node_with_different_class));
        assert!(selector.matches_node(&node_with_multiple_classes));
    }

    #[test]
    fn test_combined_selector_match() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: Some("myid".to_string()),
            class: Some("myclass".to_string()),
        };

        let matching_node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![
                    ("id".to_string(), "myid".to_string()),
                    ("class".to_string(), "myclass".to_string()),
                ],
            },
            children: vec![],
        };

        let wrong_tag = Node {
            kind: NodeKind::Element {
                tag: "span".to_string(),
                attrs: vec![
                    ("id".to_string(), "myid".to_string()),
                    ("class".to_string(), "myclass".to_string()),
                ],
            },
            children: vec![],
        };

        let wrong_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![
                    ("id".to_string(), "otherid".to_string()),
                    ("class".to_string(), "myclass".to_string()),
                ],
            },
            children: vec![],
        };

        let wrong_class = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![
                    ("id".to_string(), "myid".to_string()),
                    ("class".to_string(), "otherclass".to_string()),
                ],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&matching_node));
        assert!(!selector.matches_node(&wrong_tag));
        assert!(!selector.matches_node(&wrong_id));
        assert!(!selector.matches_node(&wrong_class));
    }

    #[test]
    fn test_selector_specificity() {
        let tag_only = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };
        assert_eq!(tag_only.specificity(), (0, 0, 1));

        let id_only = Selector {
            tag: None,
            id: Some("myid".to_string()),
            class: None,
        };
        assert_eq!(id_only.specificity(), (1, 0, 0));

        let class_only = Selector {
            tag: None,
            id: None,
            class: Some("myclass".to_string()),
        };
        assert_eq!(class_only.specificity(), (0, 1, 0));

        let combined = Selector {
            tag: Some("div".to_string()),
            id: Some("myid".to_string()),
            class: Some("myclass".to_string()),
        };
        assert_eq!(combined.specificity(), (1, 1, 1));

        let universal = Selector {
            tag: None,
            id: None,
            class: None,
        };
        assert_eq!(universal.specificity(), (0, 0, 0));
    }

    #[test]
    fn test_selector_does_not_match_text_node() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };

        let text_node = Node {
            kind: NodeKind::Text("Hello".to_string()),
            children: vec![],
        };

        assert!(!selector.matches_node(&text_node));
    }

    #[test]
    fn test_selector_does_not_match_document_node() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: None,
        };

        let document_node = Node {
            kind: NodeKind::Document,
            children: vec![],
        };

        assert!(!selector.matches_node(&document_node));
    }

    #[test]
    fn test_selector_with_tag_and_id() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: Some("myid".to_string()),
            class: None,
        };

        let matching_node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("id".to_string(), "myid".to_string())],
            },
            children: vec![],
        };

        let wrong_tag = Node {
            kind: NodeKind::Element {
                tag: "span".to_string(),
                attrs: vec![("id".to_string(), "myid".to_string())],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&matching_node));
        assert!(!selector.matches_node(&wrong_tag));
    }

    #[test]
    fn test_selector_with_tag_and_class() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: Some("div".to_string()),
            id: None,
            class: Some("myclass".to_string()),
        };

        let matching_node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("class".to_string(), "myclass".to_string())],
            },
            children: vec![],
        };

        let wrong_tag = Node {
            kind: NodeKind::Element {
                tag: "span".to_string(),
                attrs: vec![("class".to_string(), "myclass".to_string())],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&matching_node));
        assert!(!selector.matches_node(&wrong_tag));
    }

    #[test]
    fn test_selector_with_id_and_class() {
        use crate::html::{Node, NodeKind};

        let selector = Selector {
            tag: None,
            id: Some("myid".to_string()),
            class: Some("myclass".to_string()),
        };

        let matching_node = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![
                    ("id".to_string(), "myid".to_string()),
                    ("class".to_string(), "myclass".to_string()),
                ],
            },
            children: vec![],
        };

        let wrong_id = Node {
            kind: NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![
                    ("id".to_string(), "otherid".to_string()),
                    ("class".to_string(), "myclass".to_string()),
                ],
            },
            children: vec![],
        };

        assert!(selector.matches_node(&matching_node));
        assert!(!selector.matches_node(&wrong_id));
    }
}
