# Browser Subsystem Interfaces

This document defines the **stable Rust trait boundaries** between every major
subsystem. Each subsystem can be built and tested independently by a separate
agent or developer as long as it satisfies the trait defined here.

> **Rule for agents**: Implement the trait. Do not change the trait signature
> without opening a PR that updates this document first.

---

## Data flow overview

```
URL string
    │
    ▼
[url]  BrowserUrl::parse()
    │
    ▼
[network]  Fetcher::fetch()  →  RawResponse { body: Vec<u8>, headers, status }
    │
    ▼
[html]  HtmlParser::parse()  →  Dom
    │
    ▼
[css]  StyleEngine::style()  →  StyledTree
    │
    ▼
[layout]  LayoutEngine::layout()  →  LayoutTree
    │
    ▼
[paint]  Painter::paint()  →  DisplayList
    │
    ▼
[renderer]  Renderer::composite()  →  pixel buffer  →  softbuffer / wgpu
    │
    ▼
[window]  BrowserWindow  (winit event loop, chrome UI, input dispatch)
```

---

## 1. URL (`src/url.rs`) — EXISTS

```rust
/// A validated, parsed URL.
pub trait ParsedUrl {
    fn scheme(&self) -> &str;
    fn host(&self) -> &str;
    fn port(&self) -> Option<u16>;
    fn path(&self) -> &str;
    fn query(&self) -> Option<&str>;
    fn fragment(&self) -> Option<&str>;
    fn is_secure(&self) -> bool;
    fn as_str(&self) -> &str;
}

pub trait UrlParser {
    type Output: ParsedUrl;
    fn parse(input: &str) -> crate::error::Result<Self::Output>;
}
```

**Status**: `BrowserUrl` implements this informally. Formalize into the trait.

---

## 2. Network (`src/network.rs`) — EXISTS (partial)

```rust
pub struct RawResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub url: String,           // final URL after redirects
    pub mime_type: String,     // e.g. "text/html"
}

pub trait Fetcher {
    /// Perform a GET request. Follows redirects. Returns raw bytes.
    fn fetch(&self, url: &str) -> crate::error::Result<RawResponse>;

    /// Perform a POST request with a body.
    fn post(&self, url: &str, body: &[u8], content_type: &str)
        -> crate::error::Result<RawResponse>;
}
```

**Status**: `HttpClient` exists but returns `String`. Needs to return `RawResponse`.

---

## 3. HTML Parser (`src/html.rs`) — EXISTS (stub)

The current `HtmlDocument` uses a regex-based stub. Replace with a real parser
that produces a proper DOM tree.

```rust
/// A node in the DOM tree.
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

pub struct Node {
    pub kind: NodeKind,
    pub children: Vec<Node>,
}

pub struct Dom {
    pub root: Node,
}

impl Dom {
    pub fn title(&self) -> Option<&str> { ... }
    pub fn query_selector(&self, selector: &str) -> Vec<&Node> { ... }
    pub fn query_selector_all(&self, selector: &str) -> Vec<&Node> { ... }
}

pub trait HtmlParser {
    fn parse(input: &str) -> crate::error::Result<Dom>;
}
```

**Recommended starting point**: implement the [HTML5 tokenizer spec] states
(Data, TagOpen, TagName, BeforeAttrName, etc.) as an explicit state machine enum.

---

## 4. CSS Engine (`src/css/`) — NOT YET BUILT

```rust
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

pub struct Declaration {
    pub property: String,
    pub value: CssValue,
}

pub enum CssValue {
    Keyword(String),
    Length(f32, LengthUnit),
    Color(u8, u8, u8, u8),   // RGBA
    Percentage(f32),
    Number(f32),
}

pub enum LengthUnit { Px, Em, Rem, Vh, Vw }

pub trait CssParser {
    fn parse(input: &str) -> crate::error::Result<Stylesheet>;
}

pub struct StyledNode<'dom> {
    pub node: &'dom Node,
    pub styles: HashMap<String, CssValue>,
    pub children: Vec<StyledNode<'dom>>,
}

pub trait StyleEngine {
    /// Apply stylesheets to a DOM tree, producing a styled tree.
    fn style<'d>(
        &self,
        dom: &'d Dom,
        stylesheets: &[Stylesheet],
    ) -> StyledNode<'d>;
}
```

---

## 5. Layout Engine (`src/layout/`) — NOT YET BUILT

```rust
pub struct LayoutBox {
    pub rect: LayoutRect,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
}

pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub enum BoxType {
    Block,
    Inline,
    Anonymous,
}

pub trait LayoutEngine {
    /// Compute positions and sizes for every node.
    fn layout(
        &self,
        styled: &StyledNode<'_>,
        containing_width: f32,
    ) -> LayoutBox;
}
```

---

## 6. Paint / Display List (`src/paint/`) — NOT YET BUILT

```rust
pub enum DisplayCommand {
    FillRect { rect: LayoutRect, color: (u8, u8, u8, u8) },
    DrawText { x: f32, y: f32, text: String, size: f32, color: (u8, u8, u8, u8) },
    DrawImage { rect: LayoutRect, data: Arc<Vec<u8>> },
    ClipRect(LayoutRect),
    PopClip,
}

pub type DisplayList = Vec<DisplayCommand>;

pub trait Painter {
    /// Walk the layout tree and emit draw commands.
    fn paint(&self, layout: &LayoutBox) -> DisplayList;
}
```

---

## 7. Renderer (`src/renderer.rs`) — EXISTS (chrome only)

Extend to consume a `DisplayList` in addition to drawing chrome.

```rust
pub trait Compositor {
    /// Execute a display list into the pixel buffer.
    fn composite(&mut self, commands: &DisplayList);

    /// Draw browser chrome over the content.
    fn draw_chrome(&mut self, width: u32);

    /// Return the final pixel buffer (0x00RRGGBB per pixel).
    fn buffer(&self) -> &[u32];
}
```

---

## 8. JavaScript Engine (`src/js/`) — FUTURE / DEFER

This is the hardest component. Options in order of build time:
1. **Defer** — ship without JS first, cover 40% of the web
2. **Embed V8 via `rusty_v8`** — fast integration, large binary
3. **Embed QuickJS via `rquickjs`** — small, embeddable, ES2020 compliant
4. **Build from scratch** — months of work, not recommended until other layers work

**Recommended**: defer until HTML+CSS+layout render real pages, then use `rquickjs`.

---

## 9. Window / Chrome (`src/window.rs`) — EXISTS

No trait needed here — this is the top-level orchestrator. Its job is:
1. Own the winit event loop
2. Receive input events (keyboard, mouse) and route them to the right subsystem
3. Call each pipeline stage in order on navigation
4. Blit the final pixel buffer via softbuffer

```
navigate(url):
  response  = Fetcher::fetch(url)
  dom       = HtmlParser::parse(response.body)
  styled    = StyleEngine::style(dom, stylesheets)
  layout    = LayoutEngine::layout(styled, viewport_width)
  display   = Painter::paint(layout)
  renderer.composite(display)
  renderer.draw_chrome(window_width)
  buffer.present()
```

---

## Build Order (recommended for parallel agents)

| Priority | Component | Depends on | Parallelizable? |
|----------|-----------|------------|-----------------|
| 1 | HTML tokenizer + parser | nothing | yes |
| 1 | CSS parser | nothing | yes |
| 1 | Network `RawResponse` refactor | nothing | yes |
| 2 | Style engine | HTML parser + CSS parser | after P1 |
| 2 | Layout engine (block flow only) | Style engine | after P1 |
| 3 | Paint / display list | Layout engine | after P2 |
| 3 | Font rendering (fontdue) | Renderer | yes |
| 4 | Compositor integration | All above | after P3 |
| 5 | JS engine (rquickjs) | DOM | after P4 |
