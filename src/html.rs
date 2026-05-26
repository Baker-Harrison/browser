//! HTML parsing module for the browser
//!
//! Implements an HTML5 tokenizer state machine and DOM tree builder from scratch.
//! Replaces the previous scraper-based implementation.

use crate::error::Result;
use std::fmt;

// ============================================================
// Token types (internal to the tokenizer)
// ============================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    StartTag {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Doctype(String),
    Char(char),
    Eof,
}

// ============================================================
// Tokenizer — HTML5 state machine
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttrName,
    AttrName,
    AfterAttrName,
    BeforeAttrValue,
    AttrValueDoubleQuoted,
    AttrValueSingleQuoted,
    AttrValueUnquoted,
    SelfClosingStartTag,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentEndDash,
    CommentEnd,
    Doctype,
}

fn is_ascii_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0C' | '\r' | ' ')
}

fn is_ascii_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn tokenize(input: &str) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut pos = 0usize;
    let mut state = State::Data;
    let mut tokens = Vec::new();

    let mut current_name = String::new();
    let mut current_attrs: Vec<(String, String)> = Vec::new();
    let mut current_attr_name = String::new();
    let mut current_attr_value = String::new();
    let mut current_comment = String::new();
    let mut current_doctype = String::new();
    let mut is_end_tag = false;

    let mut reconsume = false;
    let mut ch: Option<char> = None;

    loop {
        if reconsume {
            reconsume = false;
        } else {
            ch = if pos < len {
                let c = chars[pos];
                pos += 1;
                Some(c)
            } else {
                None
            };
        }

        let ch = match ch {
            Some(ch) => ch,
            None => {
                match state {
                    State::Data => {
                        tokens.push(Token::Eof);
                    }
                    State::TagName
                    | State::BeforeAttrName
                    | State::AttrName
                    | State::AfterAttrName
                    | State::BeforeAttrValue
                    | State::AttrValueUnquoted => {
                        tokens.push(Token::StartTag {
                            name: std::mem::take(&mut current_name),
                            attrs: std::mem::take(&mut current_attrs),
                            self_closing: false,
                        });
                        tokens.push(Token::Eof);
                    }
                    State::AttrValueDoubleQuoted | State::AttrValueSingleQuoted => {
                        current_attrs.push((
                            std::mem::take(&mut current_attr_name),
                            std::mem::take(&mut current_attr_value),
                        ));
                        tokens.push(Token::StartTag {
                            name: std::mem::take(&mut current_name),
                            attrs: std::mem::take(&mut current_attrs),
                            self_closing: false,
                        });
                        tokens.push(Token::Eof);
                    }
                    State::SelfClosingStartTag => {
                        tokens.push(Token::StartTag {
                            name: std::mem::take(&mut current_name),
                            attrs: std::mem::take(&mut current_attrs),
                            self_closing: true,
                        });
                        tokens.push(Token::Eof);
                    }
                    State::Comment
                    | State::CommentStart
                    | State::CommentStartDash
                    | State::CommentEndDash
                    | State::CommentEnd => {
                        tokens.push(Token::Comment(std::mem::take(&mut current_comment)));
                        tokens.push(Token::Eof);
                    }
                    State::Doctype => {
                        tokens.push(Token::Doctype(std::mem::take(&mut current_doctype)));
                        tokens.push(Token::Eof);
                    }
                    State::TagOpen | State::EndTagOpen | State::MarkupDeclarationOpen => {
                        tokens.push(Token::Eof);
                    }
                }
                break;
            }
        };

        match state {
            State::Data => {
                if ch == '<' {
                    state = State::TagOpen;
                } else {
                    tokens.push(Token::Char(ch));
                }
            }

            State::TagOpen => {
                if ch == '/' {
                    state = State::EndTagOpen;
                } else if ch == '!' {
                    state = State::MarkupDeclarationOpen;
                } else if ch == '?' {
                    tokens.push(Token::Char('<'));
                    tokens.push(Token::Char('?'));
                    state = State::Data;
                } else if is_ascii_alpha(ch) {
                    current_name.clear();
                    current_attrs.clear();
                    is_end_tag = false;
                    current_name.push(ch.to_ascii_lowercase());
                    state = State::TagName;
                } else {
                    tokens.push(Token::Char('<'));
                    reconsume = true;
                    state = State::Data;
                }
            }

            State::EndTagOpen => {
                if is_ascii_alpha(ch) {
                    current_name.clear();
                    current_attrs.clear();
                    current_name.push(ch.to_ascii_lowercase());
                    is_end_tag = true;
                    state = State::TagName;
                } else {
                    state = State::Data;
                }
            }

            State::TagName => {
                if is_ascii_whitespace(ch) && is_end_tag {
                    // Parse end tag with whitespace before > is fine
                    // Skip whitespace until >
                } else if is_ascii_whitespace(ch) {
                    state = State::BeforeAttrName;
                } else if ch == '/' {
                    state = State::SelfClosingStartTag;
                } else if ch == '>' {
                    if is_end_tag {
                        tokens.push(Token::EndTag {
                            name: std::mem::take(&mut current_name),
                        });
                        is_end_tag = false;
                    } else {
                        tokens.push(Token::StartTag {
                            name: std::mem::take(&mut current_name),
                            attrs: std::mem::take(&mut current_attrs),
                            self_closing: false,
                        });
                    }
                    state = State::Data;
                } else {
                    current_name.push(ch.to_ascii_lowercase());
                }
            }

            State::BeforeAttrName => {
                if is_ascii_whitespace(ch) {
                } else if ch == '/' {
                    state = State::SelfClosingStartTag;
                } else if ch == '>' {
                    if is_end_tag {
                        tokens.push(Token::EndTag {
                            name: std::mem::take(&mut current_name),
                        });
                        is_end_tag = false;
                    } else {
                        tokens.push(Token::StartTag {
                            name: std::mem::take(&mut current_name),
                            attrs: std::mem::take(&mut current_attrs),
                            self_closing: false,
                        });
                    }
                    state = State::Data;
                } else if ch == '=' {
                    current_attr_name.clear();
                    current_attr_value.clear();
                    current_attr_name.push(ch);
                    state = State::AttrName;
                } else {
                    current_attr_name.clear();
                    current_attr_value.clear();
                    current_attr_name.push(ch.to_ascii_lowercase());
                    state = State::AttrName;
                }
            }

            State::AttrName => {
                if is_ascii_whitespace(ch) {
                    state = State::AfterAttrName;
                } else if ch == '=' {
                    state = State::BeforeAttrValue;
                } else if ch == '/' {
                    if !current_attr_name.is_empty() {
                        current_attrs.push((std::mem::take(&mut current_attr_name), String::new()));
                    }
                    state = State::SelfClosingStartTag;
                } else if ch == '>' {
                    if !current_attr_name.is_empty() {
                        current_attrs.push((std::mem::take(&mut current_attr_name), String::new()));
                    }
                    tokens.push(Token::StartTag {
                        name: std::mem::take(&mut current_name),
                        attrs: std::mem::take(&mut current_attrs),
                        self_closing: false,
                    });
                    state = State::Data;
                } else {
                    current_attr_name.push(ch.to_ascii_lowercase());
                }
            }

            State::AfterAttrName => {
                if is_ascii_whitespace(ch) {
                } else if ch == '=' {
                    state = State::BeforeAttrValue;
                } else if ch == '/' {
                    state = State::SelfClosingStartTag;
                } else if ch == '>' {
                    tokens.push(Token::StartTag {
                        name: std::mem::take(&mut current_name),
                        attrs: std::mem::take(&mut current_attrs),
                        self_closing: false,
                    });
                    state = State::Data;
                } else {
                    current_attr_name.clear();
                    current_attr_value.clear();
                    current_attr_name.push(ch.to_ascii_lowercase());
                    state = State::AttrName;
                }
            }

            State::BeforeAttrValue => {
                if is_ascii_whitespace(ch) {
                } else if ch == '"' {
                    current_attr_value.clear();
                    state = State::AttrValueDoubleQuoted;
                } else if ch == '\'' {
                    current_attr_value.clear();
                    state = State::AttrValueSingleQuoted;
                } else if ch == '>' {
                    current_attrs.push((std::mem::take(&mut current_attr_name), String::new()));
                    tokens.push(Token::StartTag {
                        name: std::mem::take(&mut current_name),
                        attrs: std::mem::take(&mut current_attrs),
                        self_closing: false,
                    });
                    state = State::Data;
                } else {
                    current_attr_value.clear();
                    current_attr_value.push(ch);
                    state = State::AttrValueUnquoted;
                }
            }

            State::AttrValueDoubleQuoted => {
                if ch == '"' {
                    current_attrs.push((
                        std::mem::take(&mut current_attr_name),
                        std::mem::take(&mut current_attr_value),
                    ));
                    state = State::BeforeAttrName;
                } else if ch == '&' {
                    current_attr_value.push('&');
                } else {
                    current_attr_value.push(ch);
                }
            }

            State::AttrValueSingleQuoted => {
                if ch == '\'' {
                    current_attrs.push((
                        std::mem::take(&mut current_attr_name),
                        std::mem::take(&mut current_attr_value),
                    ));
                    state = State::BeforeAttrName;
                } else if ch == '&' {
                    current_attr_value.push('&');
                } else {
                    current_attr_value.push(ch);
                }
            }

            State::AttrValueUnquoted => {
                if is_ascii_whitespace(ch) {
                    current_attrs.push((
                        std::mem::take(&mut current_attr_name),
                        std::mem::take(&mut current_attr_value),
                    ));
                    state = State::BeforeAttrName;
                } else if ch == '>' {
                    current_attrs.push((
                        std::mem::take(&mut current_attr_name),
                        std::mem::take(&mut current_attr_value),
                    ));
                    tokens.push(Token::StartTag {
                        name: std::mem::take(&mut current_name),
                        attrs: std::mem::take(&mut current_attrs),
                        self_closing: false,
                    });
                    state = State::Data;
                } else {
                    current_attr_value.push(ch);
                }
            }

            State::SelfClosingStartTag => {
                if ch == '>' {
                    tokens.push(Token::StartTag {
                        name: std::mem::take(&mut current_name),
                        attrs: std::mem::take(&mut current_attrs),
                        self_closing: true,
                    });
                    state = State::Data;
                } else {
                    state = State::BeforeAttrName;
                    reconsume = true;
                }
            }

            State::MarkupDeclarationOpen => {
                if ch == '-' {
                    if pos < len && chars[pos] == '-' {
                        pos += 1;
                        current_comment.clear();
                        state = State::CommentStart;
                    } else {
                        current_doctype.clear();
                        current_doctype.push('-');
                        state = State::Doctype;
                    }
                } else if matches!(ch, 'D' | 'd') {
                    let mut buf = String::from("D");
                    for _ in 0..6 {
                        if pos < len {
                            buf.push(chars[pos]);
                            pos += 1;
                        }
                    }
                    if buf.eq_ignore_ascii_case("DOCTYPE") {
                        current_doctype.clear();
                        state = State::Doctype;
                    } else {
                        current_doctype.clear();
                        for c in buf.chars() {
                            current_doctype.push(c);
                        }
                        state = State::Doctype;
                    }
                } else if ch == '[' {
                    while pos + 2 < len {
                        if chars[pos] == ']' && chars[pos + 1] == ']' && chars[pos + 2] == '>' {
                            pos += 3;
                            break;
                        }
                        pos += 1;
                    }
                    state = State::Data;
                } else {
                    current_comment.clear();
                    current_comment.push(ch);
                    while pos < len {
                        if chars[pos] == '>' {
                            pos += 1;
                            break;
                        }
                        current_comment.push(chars[pos]);
                        pos += 1;
                    }
                    tokens.push(Token::Comment(std::mem::take(&mut current_comment)));
                    state = State::Data;
                }
            }

            State::CommentStart => {
                if ch == '-' {
                    state = State::CommentStartDash;
                } else if ch == '>' {
                    tokens.push(Token::Comment(String::new()));
                    state = State::Data;
                } else {
                    current_comment.push(ch);
                    state = State::Comment;
                }
            }

            State::CommentStartDash => {
                if ch == '-' {
                    state = State::CommentEnd;
                } else if ch == '>' {
                    tokens.push(Token::Comment(String::new()));
                    state = State::Data;
                } else {
                    current_comment.push('-');
                    current_comment.push(ch);
                    state = State::Comment;
                }
            }

            State::Comment => {
                if ch == '-' {
                    state = State::CommentEndDash;
                } else {
                    current_comment.push(ch);
                }
            }

            State::CommentEndDash => {
                if ch == '-' {
                    state = State::CommentEnd;
                } else {
                    current_comment.push('-');
                    current_comment.push(ch);
                    state = State::Comment;
                }
            }

            State::CommentEnd => {
                if ch == '>' {
                    tokens.push(Token::Comment(std::mem::take(&mut current_comment)));
                    state = State::Data;
                } else if ch == '-' {
                    current_comment.push('-');
                } else {
                    current_comment.push_str("--");
                    current_comment.push(ch);
                    state = State::Comment;
                }
            }

            State::Doctype => {
                if ch == '>' {
                    tokens.push(Token::Doctype(std::mem::take(&mut current_doctype)));
                    state = State::Data;
                } else {
                    current_doctype.push(ch);
                }
            }
        }
    }

    tokens
}

// ============================================================
// Public DOM types (matching INTERFACES.md)
// ============================================================

/// A node in the DOM tree.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Document,
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
    },
    Text(String),
    Comment(String),
    Doctype(String),
}

/// A node in the DOM tree, with owned children.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            children: Vec::new(),
        }
    }
}

/// The DOM tree representing a parsed HTML document.
pub struct Dom {
    pub root: Node,
}

#[allow(dead_code)]
impl Dom {
    /// Returns the text content of the first `<title>` element, if any.
    pub fn title(&self) -> Option<&str> {
        fn find_title_text(node: &Node) -> Option<&str> {
            match &node.kind {
                NodeKind::Element { tag, .. } if tag == "title" => {
                    // Return concatenated text of all text children
                    for child in &node.children {
                        if let NodeKind::Text(t) = &child.kind {
                            return Some(t.as_str());
                        }
                    }
                    // If no direct text children, recursively search
                    for child in &node.children {
                        if let result @ Some(_) = find_title_text(child) {
                            return result;
                        }
                    }
                    None
                }
                _ => {
                    for child in &node.children {
                        if let result @ Some(_) = find_title_text(child) {
                            return result;
                        }
                    }
                    None
                }
            }
        }
        find_title_text(&self.root)
    }

    /// Collect all nodes matching a simple CSS selector.
    ///
    /// Supports:
    /// - Tag name selectors: `"title"`, `"a"`, `"img"`
    /// - ID selectors: `"#myid"`, `"div#myid"`
    /// - Class selectors: `".myclass"`, `"div.myclass"`
    /// - Attribute selectors: `"a[href]"`, `"img[src]"`
    /// - Combined selectors: `"div#myid.myclass[href]"`
    /// - Comma-separated: `"h1, h2, h3"`
    pub fn query_selector_all(&self, selector: &str) -> Vec<&Node> {
        let parts: Vec<&str> = selector
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let mut results = Vec::new();

        for part in &parts {
            let component = parse_simple_selector(part);
            collect_matching(&self.root, &component, &mut results);
        }

        results
    }

    /// Returns the first matching node for a selector, or an empty vec for simplicity
    /// (INTERFACES.md specifies this signature).
    pub fn query_selector(&self, selector: &str) -> Vec<&Node> {
        let all = self.query_selector_all(selector);
        all.into_iter().take(1).collect()
    }
}

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dom").field("root", &self.root).finish()
    }
}

impl fmt::Display for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_node(&self.root, f, 0)
    }
}

fn write_node(node: &Node, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    let prefix = "  ".repeat(indent);
    match &node.kind {
        NodeKind::Document => {
            writeln!(f, "{}Document", prefix)?;
            for child in &node.children {
                write_node(child, f, indent + 1)?;
            }
            Ok(())
        }
        NodeKind::Element { tag, attrs } => {
            if attrs.is_empty() {
                writeln!(f, "{}<{}>", prefix, tag)?;
            } else {
                let attr_str: Vec<String> = attrs
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                writeln!(f, "{}<{} {}>", prefix, tag, attr_str.join(" "))?;
            }
            for child in &node.children {
                write_node(child, f, indent + 1)?;
            }
            Ok(())
        }
        NodeKind::Text(t) => {
            writeln!(f, "{}\"{}\"", prefix, t.escape_debug())?;
            Ok(())
        }
        NodeKind::Comment(c) => {
            writeln!(f, "{}<!-- {} -->", prefix, c)?;
            Ok(())
        }
        NodeKind::Doctype(d) => {
            writeln!(f, "{}!<!DOCTYPE {}>", prefix, d)?;
            Ok(())
        }
    }
}

/// Represents a parsed CSS selector with tag, id, class, and attribute components.
#[derive(Debug, Clone, PartialEq, Default)]
struct SelectorComponent {
    tag: Option<String>,
    id: Option<String>,
    class: Option<String>,
    attr: Option<String>,
}

/// Parse a simple CSS selector supporting tag, ID, class, and attribute selectors.
///
/// Supported patterns:
/// - Tag: `"div"`, `"a"`
/// - ID: `"#myid"`
/// - Class: `".myclass"`
/// - Attribute: `"[href]"`, `"a[href]"`
/// - Combined: `"div#myid"`, `"div.myclass"`, `"#myid.myclass"`, `"div#myid.myclass[href]"`
fn parse_simple_selector(selector: &str) -> SelectorComponent {
    let selector = selector.trim();
    let mut component = SelectorComponent::default();

    // Handle attribute selector [attr]
    if let Some(bracket_start) = selector.find('[') {
        let before_attr = &selector[..bracket_start].trim();
        let inner = &selector[bracket_start + 1..];
        if let Some(bracket_end) = inner.find(']') {
            component.attr = Some(inner[..bracket_end].trim().to_string());
        }
        // Parse tag/id/class from the part before [attr]
        parse_selector_parts(before_attr, &mut component);
    } else {
        // No attribute selector, parse the whole string
        parse_selector_parts(selector, &mut component);
    }

    component
}

/// Parse tag, ID, and class from a selector string (without attribute part).
fn parse_selector_parts(s: &str, component: &mut SelectorComponent) {
    let mut chars = s.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            '#' => {
                // ID selector
                if !current.is_empty() {
                    component.tag = Some(std::mem::take(&mut current));
                }
                // Collect the ID value
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' || next == '_' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                component.id = Some(std::mem::take(&mut current));
            }
            '.' => {
                // Class selector
                if !current.is_empty() {
                    component.tag = Some(std::mem::take(&mut current));
                }
                // Collect the class value
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' || next == '_' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                component.class = Some(std::mem::take(&mut current));
            }
            _ => {
                current.push(c);
            }
        }
    }

    // If there's anything left in current, it's the tag name
    if !current.is_empty() {
        component.tag = Some(current);
    }
}

fn collect_matching<'a>(
    node: &'a Node,
    component: &SelectorComponent,
    results: &mut Vec<&'a Node>,
) {
    if let NodeKind::Element { tag, attrs } = &node.kind {
        // Check tag match
        let tag_matches = match &component.tag {
            None => true,
            Some(t) => tag == t,
        };

        // Check ID match
        let id_matches = match &component.id {
            None => true,
            Some(id) => attrs.iter().any(|(k, v)| k == "id" && v == id),
        };

        // Check class match
        let class_matches = match &component.class {
            None => true,
            Some(class) => attrs
                .iter()
                .any(|(k, v)| k == "class" && v.split_whitespace().any(|c| c == class)),
        };

        // Check attribute match
        let attr_matches = match &component.attr {
            None => true,
            Some(attr) => attrs.iter().any(|(k, _)| k == attr),
        };

        if tag_matches && id_matches && class_matches && attr_matches {
            results.push(node);
        }
    }
    for child in &node.children {
        collect_matching(child, component, results);
    }
}

// ============================================================
// HTML5 Tree Builder
// ============================================================

/// Internal node used during tree construction (uses index-based children).
#[derive(Debug, Clone)]
struct BuildNode {
    kind: NodeKind,
    children: Vec<usize>,
}

/// Internal tree builder that converts tokens into a DOM tree.
struct DomBuilder {
    nodes: Vec<BuildNode>,
    open_elements: Vec<usize>,
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

const HEAD_ELEMENTS: &[&str] = &[
    "title", "meta", "link", "style", "script", "noscript", "base",
];

const BODY_ELEMENTS: &[&str] = &[
    "a",
    "abbr",
    "address",
    "article",
    "aside",
    "b",
    "bdi",
    "bdo",
    "blockquote",
    "button",
    "canvas",
    "caption",
    "cite",
    "code",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "div",
    "dl",
    "dt",
    "em",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "i",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "li",
    "main",
    "map",
    "mark",
    "menu",
    "meter",
    "nav",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "picture",
    "pre",
    "progress",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "section",
    "select",
    "slot",
    "small",
    "source",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "tr",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
];

impl DomBuilder {
    fn new() -> Self {
        let doc = BuildNode {
            kind: NodeKind::Document,
            children: Vec::new(),
        };
        let nodes = vec![doc];
        DomBuilder {
            nodes,
            open_elements: vec![0],
        }
    }

    fn current(&self) -> usize {
        *self.open_elements.last().unwrap_or(&0)
    }

    fn has_element(&self, tag: &str) -> bool {
        self.open_elements.iter().any(
            |&idx| matches!(&self.nodes[idx].kind, NodeKind::Element { tag: t, .. } if t == tag),
        )
    }

    fn insert_element(&mut self, tag: &str, attrs: Vec<(String, String)>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(BuildNode {
            kind: NodeKind::Element {
                tag: tag.to_string(),
                attrs,
            },
            children: Vec::new(),
        });
        let parent = self.current();
        self.nodes[parent].children.push(idx);
        idx
    }

    fn insert_text(&mut self, text: &str) {
        let parent = self.current();
        // Merge with last child if it's a text node
        if let Some(&last_child) = self.nodes[parent].children.last()
            && let NodeKind::Text(ref mut existing) = self.nodes[last_child].kind
        {
            existing.push_str(text);
            return;
        }
        let idx = self.nodes.len();
        self.nodes.push(BuildNode {
            kind: NodeKind::Text(text.to_string()),
            children: Vec::new(),
        });
        self.nodes[parent].children.push(idx);
    }

    fn insert_comment(&mut self, data: &str) {
        let idx = self.nodes.len();
        self.nodes.push(BuildNode {
            kind: NodeKind::Comment(data.to_string()),
            children: Vec::new(),
        });
        let parent = self.current();
        self.nodes[parent].children.push(idx);
    }

    fn insert_doctype(&mut self, data: &str) {
        let idx = self.nodes.len();
        self.nodes.push(BuildNode {
            kind: NodeKind::Doctype(data.to_string()),
            children: Vec::new(),
        });
        let parent = self.current();
        self.nodes[parent].children.push(idx);
    }

    fn pop_until(&mut self, tag: &str) {
        while let Some(&idx) = self.open_elements.last() {
            let is_match =
                matches!(&self.nodes[idx].kind, NodeKind::Element { tag: t, .. } if t == tag);
            self.open_elements.pop();
            if is_match {
                break;
            }
        }
    }

    fn ensure_html(&mut self) {
        if !self.has_element("html") {
            let idx = self.insert_element("html", Vec::new());
            self.open_elements.push(idx);
        }
    }

    fn ensure_head(&mut self) {
        self.ensure_html();
        if !self.has_element("head") {
            let idx = self.insert_element("head", Vec::new());
            self.open_elements.push(idx);
        }
    }

    fn ensure_body(&mut self) {
        self.ensure_html();
        if !self.has_element("body") {
            let idx = self.insert_element("body", Vec::new());
            self.open_elements.push(idx);
        }
    }

    fn process(&mut self, token: Token) {
        match token {
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => {
                // Auto-insert html/head/body based on tag
                if name == "html" {
                    // If html already exists, just update attrs
                    if self.has_element("html") {
                        // Update attrs on existing html element
                        // For MVP, we skip this
                        return;
                    }
                    let idx = self.insert_element("html", attrs);
                    self.open_elements.push(idx);
                    return;
                }

                if name == "head" {
                    self.ensure_html();
                    if !self.has_element("head") {
                        let idx = self.insert_element("head", attrs);
                        self.open_elements.push(idx);
                    }
                    return;
                }

                if name == "body" {
                    self.ensure_html();
                    if !self.has_element("body") {
                        let idx = self.insert_element("body", attrs);
                        self.open_elements.push(idx);
                    }
                    return;
                }

                // Auto-insert html/head/body for elements that need them
                if HEAD_ELEMENTS.contains(&name.as_str()) && !self.has_element("head") {
                    self.ensure_head();
                } else if BODY_ELEMENTS.contains(&name.as_str()) && !self.has_element("body") {
                    self.ensure_body();
                }

                let is_void = VOID_ELEMENTS.contains(&name.as_str());

                let idx = self.insert_element(&name, attrs);
                if !is_void && !self_closing {
                    self.open_elements.push(idx);
                }
            }
            Token::EndTag { name } => {
                if VOID_ELEMENTS.contains(&name.as_str()) {
                    // Ignore end tags for void elements
                    return;
                }
                self.pop_until(&name);
            }
            Token::Char(ch) => {
                // Auto-insert html/body if needed for bare text
                if !self.has_element("body") && !self.has_element("head") {
                    self.ensure_body();
                }
                let s: String = ch.into();
                self.insert_text(&s);
            }
            Token::Comment(data) => {
                self.insert_comment(&data);
            }
            Token::Doctype(data) => {
                self.insert_doctype(&data);
            }
            Token::Eof => {
                // Close all open elements
                while self.open_elements.len() > 1 {
                    self.open_elements.pop();
                }
            }
        }
    }

    fn finalize(self) -> Dom {
        fn resolve(idx: usize, nodes: &[BuildNode]) -> Node {
            let kind = nodes[idx].kind.clone();
            let children: Vec<Node> = nodes[idx]
                .children
                .iter()
                .map(|&child_idx| resolve(child_idx, nodes))
                .collect();
            Node { kind, children }
        }

        Dom {
            root: resolve(0, &self.nodes),
        }
    }
}

/// Parse HTML string into a DOM tree.
fn parse_html(input: &str) -> Dom {
    let tokens = tokenize(input);
    let mut builder = DomBuilder::new();
    for token in tokens {
        builder.process(token);
    }
    builder.finalize()
}

// ============================================================
// HtmlParser trait (from INTERFACES.md)
// ============================================================

/// Trait for parsing HTML into a DOM tree.
#[allow(dead_code)]
pub trait HtmlParser {
    fn parse(input: &str) -> Result<Dom>;
}

/// Default HTML parser implementation.
#[allow(dead_code)]
pub struct DefaultHtmlParser;

impl HtmlParser for DefaultHtmlParser {
    fn parse(input: &str) -> Result<Dom> {
        Ok(parse_html(input))
    }
}

// ============================================================
// HtmlDocument — backward-compatible public API wrapper
// ============================================================

/// Represents a parsed HTML document
pub struct HtmlDocument {
    /// The parsed DOM tree
    dom: Dom,
    /// The original HTML string
    original: String,
}

impl HtmlDocument {
    /// Parse an HTML string into a structured document
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML string to parse
    ///
    /// # Returns
    ///
    /// Returns a parsed `HtmlDocument`
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::HtmlParseError` if parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = "<html><head><title>Test</title></head><body>Hello</body></html>";
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// assert_eq!(doc.title(), Some("Test".to_string()));
    /// ```
    pub fn parse(html: &str) -> Result<Self> {
        if html.is_empty() {
            return Ok(HtmlDocument {
                dom: Dom {
                    root: Node::new(NodeKind::Document),
                },
                original: String::new(),
            });
        }
        let dom = parse_html(html);
        Ok(HtmlDocument {
            dom,
            original: html.to_string(),
        })
    }

    /// Access the parsed DOM tree
    pub fn dom(&self) -> &Dom {
        &self.dom
    }

    /// Extract the title from the HTML document
    ///
    /// # Returns
    ///
    /// Returns the title text if found, otherwise None
    pub fn title(&self) -> Option<String> {
        self.dom.title().map(|s| s.to_string())
    }

    /// Extract all links (href attributes) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of link URLs found in the document
    pub fn links(&self) -> Vec<String> {
        let mut links = Vec::new();
        collect_links(&self.dom.root, &mut links);
        links
    }

    /// Extract all image sources (src attributes) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of image URLs found in the document
    pub fn images(&self) -> Vec<String> {
        let mut images = Vec::new();
        collect_images(&self.dom.root, &mut images);
        images
    }

    /// Extract the text content from the HTML document
    ///
    /// This returns the visible text content, stripping HTML tags
    pub fn text_content(&self) -> String {
        collect_text(&self.dom.root)
    }

    /// Extract all headings (h1-h6) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of (level, text) tuples for each heading
    pub fn headings(&self) -> Vec<(u8, String)> {
        let mut headings = Vec::new();
        collect_headings(&self.dom.root, &mut headings);
        headings
    }

    /// Get the original HTML string
    pub fn as_html(&self) -> &str {
        &self.original
    }

    /// Get the length of the original HTML
    pub fn len(&self) -> usize {
        self.original.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.original.is_empty()
    }
}

// ============================================================
// Helper functions for tree traversal
// ============================================================

fn collect_links(node: &Node, links: &mut Vec<String>) {
    if let NodeKind::Element { tag, attrs } = &node.kind
        && tag == "a"
    {
        for (name, value) in attrs {
            if name == "href" {
                links.push(value.clone());
            }
        }
    }
    for child in &node.children {
        collect_links(child, links);
    }
}

fn collect_images(node: &Node, images: &mut Vec<String>) {
    if let NodeKind::Element { tag, attrs } = &node.kind
        && tag == "img"
    {
        for (name, value) in attrs {
            if name == "src" {
                images.push(value.clone());
            }
        }
    }
    for child in &node.children {
        collect_images(child, images);
    }
}

fn collect_text(node: &Node) -> String {
    match &node.kind {
        NodeKind::Text(t) => t.clone(),
        NodeKind::Element { tag, .. } if tag == "script" || tag == "style" => String::new(),
        _ => {
            let mut s = String::new();
            for child in &node.children {
                s.push_str(&collect_text(child));
            }
            s
        }
    }
}

fn collect_headings(node: &Node, headings: &mut Vec<(u8, String)>) {
    if let NodeKind::Element { tag, .. } = &node.kind
        && let Some(level_str) = tag.strip_prefix('h')
        && let Ok(level) = level_str.parse::<u8>()
        && (1..=6).contains(&level)
    {
        headings.push((level, collect_text(node)));
    }
    for child in &node.children {
        collect_headings(child, headings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_html() {
        let html = "<html><body>Hello</body></html>";
        let doc = HtmlDocument::parse(html);
        assert!(doc.is_ok());
    }

    #[test]
    fn test_parse_empty_html() {
        let html = "";
        let doc = HtmlDocument::parse(html);
        assert!(doc.is_ok());
        assert!(doc.unwrap().is_empty());
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.title(), Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_title_none() {
        let html = "<html><head></head><body></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.title(), None);
    }

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html><body>
                <a href="https://example.com">Example</a>
                <a href="/about">About</a>
                <a>No href</a>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let links = doc.links();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com".to_string()));
        assert!(links.contains(&"/about".to_string()));
    }

    #[test]
    fn test_extract_images() {
        let html = r#"
            <html><body>
                <img src="image1.jpg" />
                <img src="/images/image2.png" />
                <img>No src</img>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let images = doc.images();
        assert_eq!(images.len(), 2);
        assert!(images.contains(&"image1.jpg".to_string()));
        assert!(images.contains(&"/images/image2.png".to_string()));
    }

    #[test]
    fn test_extract_text_content() {
        let html = "<html><body><p>Hello <strong>World</strong></p></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        let text = doc.text_content();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_extract_headings() {
        let html = r#"
            <html><body>
                <h1>Main Title</h1>
                <h2>Subtitle</h2>
                <h3>Section</h3>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let headings = doc.headings();
        assert_eq!(headings.len(), 3);
        assert!(headings.contains(&(1, "Main Title".to_string())));
        assert!(headings.contains(&(2, "Subtitle".to_string())));
        assert!(headings.contains(&(3, "Section".to_string())));
    }

    #[test]
    fn test_as_html() {
        let html = "<html><body>Test</body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.as_html(), html);
    }

    #[test]
    fn test_len() {
        let html = "<html><body>Test</body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.len(), html.len());
    }

    #[test]
    fn test_is_empty() {
        let doc = HtmlDocument::parse("").unwrap();
        assert!(doc.is_empty());

        let doc = HtmlDocument::parse("<html></html>").unwrap();
        assert!(!doc.is_empty());
    }

    // Additional tests for the tokenizer and DOM

    #[test]
    fn test_tokenizer_basic_tags() {
        let tokens = tokenize("<html><body>Hello</body></html>");
        let token_types: Vec<&str> = tokens
            .iter()
            .map(|t| match t {
                Token::StartTag { .. } => "StartTag",
                Token::EndTag { .. } => "EndTag",
                Token::Char(_) => "Char",
                Token::Eof => "Eof",
                Token::Comment(_) => "Comment",
                Token::Doctype(_) => "Doctype",
            })
            .collect();
        assert_eq!(
            token_types,
            vec![
                "StartTag", "StartTag", "Char", "Char", "Char", "Char", "Char", "EndTag", "EndTag",
                "Eof"
            ]
        );
    }

    #[test]
    fn test_tokenizer_attrs() {
        let tokens = tokenize(r#"<a href="https://example.com">text</a>"#);
        let start_tags: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t, Token::StartTag { .. }))
            .collect();
        assert_eq!(start_tags.len(), 1);
        if let Token::StartTag { name, attrs, .. } = &start_tags[0] {
            assert_eq!(name, "a");
            assert_eq!(attrs.len(), 1);
            assert_eq!(
                attrs[0],
                ("href".to_string(), "https://example.com".to_string())
            );
        } else {
            panic!("Expected StartTag");
        }
    }

    #[test]
    fn test_tokenizer_self_closing() {
        let tokens = tokenize(r#"<img src="test.jpg" />"#);
        let start_tags: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t, Token::StartTag { .. }))
            .collect();
        assert_eq!(start_tags.len(), 1);
        if let Token::StartTag {
            name,
            attrs,
            self_closing,
        } = &start_tags[0]
        {
            assert_eq!(name, "img");
            assert!(attrs.iter().any(|(k, v)| k == "src" && v == "test.jpg"));
            assert!(*self_closing);
        } else {
            panic!("Expected StartTag");
        }
    }

    #[test]
    fn test_tokenizer_comment() {
        let tokens = tokenize("<!-- hello world -->");
        let comments: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t, Token::Comment(_)))
            .collect();
        assert_eq!(comments.len(), 1);
        if let Token::Comment(data) = &comments[0] {
            assert_eq!(data, " hello world ");
        } else {
            panic!("Expected Comment");
        }
    }

    #[test]
    fn test_tokenizer_doctype() {
        let tokens = tokenize("<!DOCTYPE html>");
        let doctypes: Vec<&Token> = tokens
            .iter()
            .filter(|t| matches!(t, Token::Doctype(_)))
            .collect();
        assert_eq!(doctypes.len(), 1);
        if let Token::Doctype(data) = &doctypes[0] {
            assert!(data.trim().eq_ignore_ascii_case("html"));
        } else {
            panic!("Expected Doctype");
        }
    }

    #[test]
    fn test_dom_title() {
        let dom = parse_html("<html><head><title>My Page</title></head><body></body></html>");
        assert_eq!(dom.title(), Some("My Page"));
    }

    #[test]
    fn test_dom_title_none() {
        let dom = parse_html("<html><body></body></html>");
        assert_eq!(dom.title(), None);
    }

    #[test]
    fn test_dom_query_selector_all() {
        let dom = parse_html(
            r#"<html><body>
            <a href="https://example.com">Link 1</a>
            <a href="https://other.com">Link 2</a>
        </body></html>"#,
        );
        let links = dom.query_selector_all("a[href]");
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_dom_headings_selector() {
        let dom = parse_html("<html><body><h1>Title</h1><h2>Sub</h2><h3>SubSub</h3></body></html>");
        let headings = dom.query_selector_all("h1, h2, h3");
        assert_eq!(headings.len(), 3);
    }

    #[test]
    fn test_void_element_not_nested() {
        // Void elements should not have children pushed onto stack
        let dom = parse_html("<html><body><img src=\"test.jpg\" />after</body></html>");
        let text = dom.query_selector_all("body");
        assert_eq!(text.len(), 1);
        // "after" should be a sibling of img, not nested inside it
        let text_content = collect_text(&dom.root);
        assert!(text_content.contains("after"));
    }

    #[test]
    fn test_nested_elements() {
        let dom = parse_html("<html><body><div><p>Text</p></div></body></html>");
        let paragraphs = dom.query_selector_all("p");
        assert_eq!(paragraphs.len(), 1);
        if let NodeKind::Element { tag, .. } = &paragraphs[0].kind {
            assert_eq!(tag, "p");
        }
    }

    #[test]
    fn test_text_merging() {
        let tokens = tokenize("Hello World");
        // Should produce individual Char tokens
        let chars: String = tokens
            .iter()
            .filter_map(|t| {
                if let Token::Char(c) = t {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(chars, "Hello World");
    }

    #[test]
    fn test_html_parser_trait() {
        let dom =
            DefaultHtmlParser::parse("<html><head><title>Test</title></head></html>").unwrap();
        assert_eq!(dom.title(), Some("Test"));
    }

    #[test]
    fn test_count_paragraphs_in_tree() {
        fn count_p(node: &Node) -> usize {
            let mut n = if matches!(&node.kind, NodeKind::Element { tag, .. } if tag == "p") {
                1
            } else {
                0
            };
            for child in &node.children {
                n += count_p(child);
            }
            n
        }
        let dom = parse_html("<html><body><p>Hello</p><p>World</p></body></html>");
        assert_eq!(count_p(&dom.root), 2);
    }

    #[test]
    fn test_find_span_in_tree() {
        fn find_span(node: &Node) -> bool {
            if matches!(&node.kind, NodeKind::Element { tag, .. } if tag == "span") {
                return true;
            }
            for child in &node.children {
                if find_span(child) {
                    return true;
                }
            }
            false
        }
        let dom = parse_html("<html><body><div><span>Target</span></div></body></html>");
        assert!(find_span(&dom.root));
    }

    // ============================================================
    // DOM Query API Tests - Tag Selectors
    // ============================================================

    #[test]
    fn test_query_selector_tag_simple() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all("div");
        assert_eq!(results.len(), 1);
        if let NodeKind::Element { tag, .. } = &results[0].kind {
            assert_eq!(tag, "div");
        }
    }

    #[test]
    fn test_query_selector_tag_multiple() {
        let dom = parse_html("<html><body><p>First</p><p>Second</p><p>Third</p></body></html>");
        let results = dom.query_selector_all("p");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_query_selector_tag_none() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all("span");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_selector_tag_nested() {
        let dom = parse_html("<html><body><div><p>Nested</p></div></body></html>");
        let results = dom.query_selector_all("p");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_tag_comma_separated() {
        let dom =
            parse_html("<html><body><h1>Title</h1><h2>Subtitle</h2><p>Text</p></body></html>");
        let results = dom.query_selector_all("h1, h2");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_selector_returns_first() {
        let dom = parse_html("<html><body><p>First</p><p>Second</p></body></html>");
        let results = dom.query_selector("p");
        assert_eq!(results.len(), 1);
        if let NodeKind::Element { tag, .. } = &results[0].kind {
            assert_eq!(tag, "p");
        }
    }

    // ============================================================
    // DOM Query API Tests - ID Selectors
    // ============================================================

    #[test]
    fn test_query_selector_id_simple() {
        let dom = parse_html("<html><body><div id=\"main\">Content</div></body></html>");
        let results = dom.query_selector_all("#main");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_id_multiple() {
        let dom = parse_html(
            "<html><body><div id=\"first\">First</div><div id=\"second\">Second</div></body></html>",
        );
        let results = dom.query_selector_all("#first");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_id_none() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all("#nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_selector_id_nested() {
        let dom = parse_html("<html><body><div><p id=\"nested\">Nested</p></div></body></html>");
        let results = dom.query_selector_all("#nested");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_tag_with_id() {
        let dom = parse_html(
            "<html><body><div id=\"main\">Content</div><p id=\"main\">Text</p></body></html>",
        );
        let results = dom.query_selector_all("div#main");
        assert_eq!(results.len(), 1);
        if let NodeKind::Element { tag, .. } = &results[0].kind {
            assert_eq!(tag, "div");
        }
    }

    #[test]
    fn test_query_selector_tag_with_id_mismatch() {
        let dom = parse_html("<html><body><div id=\"main\">Content</div></body></html>");
        let results = dom.query_selector_all("p#main");
        assert_eq!(results.len(), 0);
    }

    // ============================================================
    // DOM Query API Tests - Class Selectors
    // ============================================================

    #[test]
    fn test_query_selector_class_simple() {
        let dom = parse_html("<html><body><div class=\"container\">Content</div></body></html>");
        let results = dom.query_selector_all(".container");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_class_multiple() {
        let dom = parse_html(
            "<html><body><div class=\"box\">Box 1</div><div class=\"box\">Box 2</div></body></html>",
        );
        let results = dom.query_selector_all(".box");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_selector_class_none() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all(".nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_selector_class_multiple_classes() {
        let dom =
            parse_html("<html><body><div class=\"box container\">Content</div></body></html>");
        let results = dom.query_selector_all(".container");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_tag_with_class() {
        let dom = parse_html(
            "<html><body><div class=\"box\">Div</div><p class=\"box\">Para</p></body></html>",
        );
        let results = dom.query_selector_all("div.box");
        assert_eq!(results.len(), 1);
        if let NodeKind::Element { tag, .. } = &results[0].kind {
            assert_eq!(tag, "div");
        }
    }

    #[test]
    fn test_query_selector_tag_with_class_mismatch() {
        let dom = parse_html("<html><body><div class=\"box\">Content</div></body></html>");
        let results = dom.query_selector_all("p.box");
        assert_eq!(results.len(), 0);
    }

    // ============================================================
    // DOM Query API Tests - Combined Selectors
    // ============================================================

    #[test]
    fn test_query_selector_tag_id_class() {
        let dom = parse_html(
            "<html><body><div id=\"main\" class=\"container\">Content</div></body></html>",
        );
        let results = dom.query_selector_all("div#main.container");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_id_class() {
        let dom = parse_html(
            "<html><body><div id=\"main\" class=\"container\">Content</div></body></html>",
        );
        let results = dom.query_selector_all("#main.container");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_tag_id_class_attr() {
        let dom = parse_html(
            "<html><body><div id=\"main\" class=\"container\" data-test=\"value\">Content</div></body></html>",
        );
        let results = dom.query_selector_all("div#main.container[data-test]");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_combined_mismatch() {
        let dom = parse_html(
            "<html><body><div id=\"main\" class=\"container\">Content</div></body></html>",
        );
        let results = dom.query_selector_all("div#other.container");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_selector_combined_partial_match() {
        let dom = parse_html(
            "<html><body><div id=\"main\" class=\"container\">Content</div></body></html>",
        );
        let results = dom.query_selector_all("div#main");
        assert_eq!(results.len(), 1);
    }

    // ============================================================
    // DOM Query API Tests - Attribute Selectors (existing functionality)
    // ============================================================

    #[test]
    fn test_query_selector_attr_simple() {
        let dom = parse_html("<html><body><a href=\"https://example.com\">Link</a></body></html>");
        let results = dom.query_selector_all("[href]");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_tag_attr() {
        let dom = parse_html("<html><body><a href=\"https://example.com\">Link</a></body></html>");
        let results = dom.query_selector_all("a[href]");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_selector_attr_none() {
        let dom = parse_html("<html><body><a>Link</a></body></html>");
        let results = dom.query_selector_all("[href]");
        assert_eq!(results.len(), 0);
    }

    // ============================================================
    // DOM Query API Tests - Complex Real-world Scenarios
    // ============================================================

    #[test]
    fn test_query_selector_complex_html() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <header id="header" class="main-header">
                        <nav class="navigation">
                            <a href="/home" class="nav-link">Home</a>
                            <a href="/about" class="nav-link">About</a>
                        </nav>
                    </header>
                    <main id="content" class="container">
                        <article class="post">
                            <h1 class="title">Article Title</h1>
                            <p class="text">Content here</p>
                        </article>
                    </main>
                    <footer id="footer" class="main-footer">
                        <p>Footer text</p>
                    </footer>
                </body>
            </html>
        "#;
        let dom = parse_html(html);

        // Test various selectors
        assert_eq!(dom.query_selector_all("a").len(), 2);
        assert_eq!(dom.query_selector_all(".nav-link").len(), 2);
        assert_eq!(dom.query_selector_all("#header").len(), 1);
        assert_eq!(dom.query_selector_all("article.post").len(), 1);
        assert_eq!(dom.query_selector_all("h1.title").len(), 1);
        assert_eq!(dom.query_selector_all("[href]").len(), 2);
    }

    #[test]
    fn test_query_selector_empty_selector() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all("");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_selector_whitespace_handling() {
        let dom = parse_html("<html><body><div>Content</div></body></html>");
        let results = dom.query_selector_all("  div  ");
        assert_eq!(results.len(), 1);
    }
}
