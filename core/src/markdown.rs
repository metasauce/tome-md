//! Markdown -> Typst conversion with a small GFM subset.

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, Options, Parser, Tag, TagEnd,
};
use serde_yaml_ng::{Mapping, Value};

use crate::error::Error;

/// A parsed document: front-matter values and the rendered element list.
#[derive(Debug)]
pub struct Document {
    pub meta: Meta,
    pub elements: Vec<Element>,
    has_alerts: bool,
}

/// One parsed front-matter value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaValue {
    /// A scalar, converted to a string (dates, numbers and booleans included).
    Scalar(String),
    /// A nested mapping, in source order.
    Mapping(std::collections::BTreeMap<String, MetaValue>),
    /// A YAML sequence.
    Sequence(Vec<MetaValue>),
    /// An explicit YAML null.
    Null,
}

/// Top-level front-matter keys and their values.
pub type Meta = std::collections::BTreeMap<String, MetaValue>;

impl MetaValue {
    /// The scalar text, if this is a scalar.
    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::Mapping(_) | Self::Sequence(_) | Self::Null => None,
        }
    }

    /// The nested mapping, if this is a mapping.
    pub fn as_mapping(&self) -> Option<&std::collections::BTreeMap<String, Self>> {
        match self {
            Self::Mapping(value) => Some(value),
            Self::Scalar(_) | Self::Sequence(_) | Self::Null => None,
        }
    }

    /// Render this value as a Typst expression: content for scalars,
    /// dictionaries for mappings and arrays for sequences.
    pub(crate) fn to_typst(&self) -> String {
        match self {
            Self::Scalar(value) => format!("[{}]", escape_typst(value)),
            Self::Null => "none".to_owned(),
            Self::Sequence(values) => {
                let values = values
                    .iter()
                    .map(Self::to_typst)
                    .collect::<Vec<_>>()
                    .join(", ");
                if values.is_empty() {
                    "()".to_owned()
                } else {
                    format!("({values},)")
                }
            }
            Self::Mapping(values) => {
                if values.is_empty() {
                    return "(:)".to_owned();
                }
                let fields = values
                    .iter()
                    .map(|(key, value)| {
                        let key = if is_typst_ident(key) {
                            key.clone()
                        } else {
                            format!("\"{}\"", escape_string(key))
                        };
                        format!("{key}: {}", value.to_typst())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({fields})")
            }
        }
    }
}

/// One element of the rendered Markdown, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// A run of non-table Typst content.
    Content(String),
    /// A rendered table and its plain-text data.
    Table {
        content: String,
        data: MarkdownTable,
    },
}

/// One Markdown table with plain-text cells. `rows[0]` is the header row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownTable {
    pub rows: Vec<Vec<String>>,
}

impl Document {
    /// Whether the document contains GitHub alert blockquotes.
    pub fn has_alerts(&self) -> bool {
        self.has_alerts
    }

    /// Concatenated Typst source of all elements.
    pub fn body(&self) -> String {
        let mut out = String::new();
        for element in &self.elements {
            match element {
                Element::Content(content) => out.push_str(content),
                Element::Table { content, .. } => out.push_str(content),
            }
        }
        out
    }

    /// Plain-text tables in document order.
    pub fn tables(&self) -> impl Iterator<Item = &MarkdownTable> {
        self.elements.iter().filter_map(|element| match element {
            Element::Table { data, .. } => Some(data),
            Element::Content(_) => None,
        })
    }
}

/// Parse Markdown (optional `key: value` front matter) and emit elements.
pub fn parse(markdown: &str) -> Result<Document, Error> {
    let (meta, body) = split_frontmatter(markdown)?;
    let (elements, has_alerts) = to_elements(body);
    Ok(Document {
        meta,
        elements,
        has_alerts,
    })
}

/// Extract the Markdown tables as plain-text data. The first row is the header.
pub fn extract_tables(markdown: &str) -> Vec<MarkdownTable> {
    let mut tables = Vec::new();
    let mut table: Option<MarkdownTable> = None;
    let mut row: Vec<String> = Vec::new();
    let mut cell: Option<String> = None;

    for event in Parser::new_ext(markdown, Options::all()) {
        match event {
            Event::Start(Tag::Table(_)) => {
                table = Some(MarkdownTable { rows: Vec::new() });
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => row.clear(),
            Event::Start(Tag::TableCell) => cell = Some(String::new()),
            Event::Text(text) => push_cell(&mut cell, &text),
            Event::Code(code) => push_cell(&mut cell, &code.replace('\n', " ")),
            Event::SoftBreak | Event::HardBreak => push_cell(&mut cell, " "),
            Event::TaskListMarker(done) => {
                push_cell(&mut cell, if done { "☑ " } else { "☐ " });
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(text) = cell.take() {
                    row.push(normalize_cell(&text));
                }
            }
            Event::End(TagEnd::TableHead | TagEnd::TableRow) => {
                if let Some(table) = &mut table {
                    table.rows.push(std::mem::take(&mut row));
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = table.take() {
                    tables.push(table);
                }
            }
            _ => {}
        }
    }

    tables
}

fn push_cell(cell: &mut Option<String>, text: &str) {
    if let Some(cell) = cell {
        cell.push_str(text);
    }
}

fn split_frontmatter(input: &str) -> Result<(Meta, &str), Error> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let meta = Meta::new();
    let Some((first, mut rest)) = input.split_once('\n') else {
        return Ok((meta, input));
    };
    if first.trim() != "---" {
        return Ok((meta, input));
    }

    let mut yaml = String::new();
    loop {
        let Some((line, after)) = rest.split_once('\n') else {
            if rest.trim() == "---" {
                return Ok((parse_frontmatter(&yaml)?, ""));
            }
            return Ok((Meta::new(), input));
        };

        let line = line.trim_end_matches('\r');
        if line.trim_end() == "---" {
            return Ok((
                parse_frontmatter(&yaml)?,
                after.trim_start_matches(['\r', '\n']),
            ));
        }

        if !yaml.is_empty() {
            yaml.push('\n');
        }
        yaml.push_str(line);
        rest = after;
    }
}

fn parse_frontmatter(yaml: &str) -> Result<Meta, Error> {
    if yaml.trim().is_empty() {
        return Ok(Meta::new());
    }

    let value: Value = serde_yaml_ng::from_str(yaml).map_err(|error| Error::FrontMatter {
        message: error.to_string(),
    })?;
    let Value::Mapping(mapping) = value else {
        return Err(Error::FrontMatter {
            message: "the front matter must be a YAML mapping".to_owned(),
        });
    };

    mapping_to_meta(mapping)
}

fn mapping_to_meta(mapping: Mapping) -> Result<Meta, Error> {
    let mut meta = Meta::new();
    for (key, value) in mapping {
        let Value::String(key) = key else {
            return Err(Error::FrontMatter {
                message: "front matter keys must be strings".to_owned(),
            });
        };
        meta.insert(key, yaml_to_meta(value)?);
    }
    Ok(meta)
}

fn yaml_to_meta(value: Value) -> Result<MetaValue, Error> {
    match value {
        Value::Null => Ok(MetaValue::Null),
        Value::Bool(value) => Ok(MetaValue::Scalar(value.to_string())),
        Value::Number(value) => Ok(MetaValue::Scalar(value.to_string())),
        Value::String(value) => Ok(MetaValue::Scalar(value)),
        Value::Sequence(values) => values
            .into_iter()
            .map(yaml_to_meta)
            .collect::<Result<Vec<_>, _>>()
            .map(MetaValue::Sequence),
        Value::Mapping(mapping) => {
            let mut values = std::collections::BTreeMap::new();
            for (key, value) in mapping {
                let Value::String(key) = key else {
                    return Err(Error::FrontMatter {
                        message: "nested front matter keys must be strings".to_owned(),
                    });
                };
                values.insert(key, yaml_to_meta(value)?);
            }
            Ok(MetaValue::Mapping(values))
        }
        Value::Tagged(value) => yaml_to_meta(value.value),
    }
}

fn is_typst_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let first = chars.next();
    let rest_valid = chars.all(|ch| ch == '_' || ch.is_alphanumeric());
    let is_keyword = matches!(
        value,
        "none"
            | "auto"
            | "true"
            | "false"
            | "let"
            | "set"
            | "show"
            | "if"
            | "else"
            | "for"
            | "in"
            | "while"
            | "break"
            | "continue"
            | "return"
            | "import"
            | "include"
            | "as"
            | "context"
            | "and"
            | "or"
            | "not"
    );
    first.is_some_and(|first| first == '_' || first.is_alphabetic()) && rest_valid && !is_keyword
}

fn to_elements(markdown: &str) -> (Vec<Element>, bool) {
    let mut emitter = Emitter::new(extract_tables(markdown));
    for event in Parser::new_ext(markdown, Options::all()) {
        emitter.event(event);
    }
    emitter.finish()
}

struct Emitter {
    out: String,
    elements: Vec<Element>,
    tables: std::vec::IntoIter<MarkdownTable>,
    list_stack: Vec<ListState>,
    code_block: Option<CodeBlock>,
    link_stack: Vec<bool>,
    image: Option<ImageState>,
    has_alerts: bool,
    blockquote_depth: usize,
    table: Option<Table>,
    cell: Option<String>,
}

#[derive(Clone, Copy)]
struct ListState {
    ordered: bool,
    start: Option<u64>,
}

struct CodeBlock {
    lang: String,
    text: String,
}

struct ImageState {
    alt: String,
}

struct Table {
    alignments: Vec<Alignment>,
    header: Vec<Vec<String>>,
    rows: Vec<Vec<String>>,
    row: Vec<String>,
    inline: bool,
}

impl Emitter {
    fn new(tables: Vec<MarkdownTable>) -> Self {
        Self {
            out: String::new(),
            elements: Vec::new(),
            tables: tables.into_iter(),
            list_stack: Vec::new(),
            code_block: None,
            link_stack: Vec::new(),
            image: None,
            has_alerts: false,
            blockquote_depth: 0,
            table: None,
            cell: None,
        }
    }

    fn flush_content(&mut self) {
        if !self.out.is_empty() {
            self.elements
                .push(Element::Content(std::mem::take(&mut self.out)));
        }
    }

    fn target(&mut self) -> &mut String {
        self.cell.as_mut().unwrap_or(&mut self.out)
    }

    fn write(&mut self, text: &str) {
        self.target().push_str(text);
    }

    /// Write text with Typst markup escaped, without a temporary allocation.
    fn write_escaped(&mut self, text: &str) {
        let target = self.cell.as_mut().unwrap_or(&mut self.out);
        for ch in text.chars() {
            if is_typst_special(ch) {
                target.push('\\');
            }
            target.push(if ch == '\n' { ' ' } else { ch });
        }
    }

    fn newline(&mut self) {
        if self.table.is_some() {
            self.write(" ");
        } else if self.list_stack.is_empty() {
            self.write("\n\n");
        } else {
            self.write("\n");
        }
    }

    /// Ensure the next write starts on a new line, but never add a second one.
    /// A blank line inside a list is a paragraph break to Typst: items then get
    /// `par` spacing instead of list spacing, and nested items drift right.
    fn line_break(&mut self) {
        if !self.at_line_start() {
            self.write("\n");
        }
    }

    fn at_line_start(&self) -> bool {
        self.cell.as_deref().unwrap_or(&self.out).ends_with('\n')
    }

    /// Two spaces per nesting level, with top-level items at the margin so they
    /// line up with the surrounding text.
    fn list_indent(&self) -> String {
        "  ".repeat(self.list_stack.len().saturating_sub(1))
    }

    fn event(&mut self, event: Event<'_>) {
        if let Some(image) = &mut self.image {
            match event {
                Event::Text(text) => image.alt.push_str(&text),
                Event::Code(code) => image.alt.push_str(&code),
                Event::SoftBreak | Event::HardBreak => image.alt.push(' '),
                Event::End(TagEnd::Image) => {
                    let image = self.image.take().expect("image state");
                    if !image.alt.trim().is_empty() {
                        self.write(", alt: \"");
                        self.write(&escape_string(image.alt.trim()));
                        self.write("\"");
                    }
                    self.write(")");
                }
                _ => {}
            }
            return;
        }

        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => {
                if let Some(code) = &mut self.code_block {
                    code.text.push_str(&text);
                } else {
                    self.write_escaped(&text);
                }
            }
            Event::Code(code) => {
                let code = code.replace('\n', " ");
                self.write(&raw_inline(&code));
            }
            Event::SoftBreak => {
                if let Some(code) = &mut self.code_block {
                    code.text.push('\n');
                } else {
                    self.write(" ");
                }
            }
            Event::HardBreak => {
                if let Some(code) = &mut self.code_block {
                    code.text.push('\n');
                } else {
                    self.write("\\\n");
                }
            }
            Event::Rule => {
                self.write("\n#line(length: 100%, stroke: 0.75pt + luma(185))\n\n");
            }
            Event::TaskListMarker(done) => self.write(if done { "☑ " } else { "☐ " }),
            // Inline superscript/subscript tags are supported; other raw
            // HTML and math are not rendered by this small pipeline.
            Event::InlineHtml(html) => match html.as_ref() {
                "<sup>" => self.write("#super["),
                "</sup>" => self.write("]"),
                "<sub>" => self.write("#sub["),
                "</sub>" => self.write("]"),
                _ => {}
            },
            Event::Html(_) => {}
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.write("\n");
                self.write(&"=".repeat(level as usize));
                self.write(" ");
            }
            Tag::Paragraph => {}
            Tag::Emphasis => self.write("#emph["),
            Tag::Strong => self.write("#strong["),
            Tag::Strikethrough => self.write("#strike["),
            Tag::Superscript => self.write("#super["),
            Tag::Subscript => self.write("#sub["),
            Tag::Link { dest_url, .. } => {
                if dest_url.is_empty() {
                    self.link_stack.push(false);
                } else {
                    self.write("#link(\"");
                    self.write(&escape_string(&dest_url));
                    self.write("\")[");
                    self.link_stack.push(true);
                }
            }
            Tag::Image { dest_url, .. } => {
                self.write("#image(\"");
                self.write(&escape_string(&dest_url));
                self.write("\"");
                self.image = Some(ImageState { alt: String::new() });
            }
            Tag::BlockQuote(Some(kind)) => {
                self.has_alerts = true;
                self.blockquote_depth += 1;
                self.write("\n#_tome_md_github_alert(\"");
                self.write(alert_kind_name(kind));
                self.write("\")[\n");
            }
            Tag::BlockQuote(None) => {
                self.blockquote_depth += 1;
                self.write("\n#quote(block: true)[\n");
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code_block = Some(CodeBlock {
                    lang: lang.trim().to_owned(),
                    text: String::new(),
                });
            }
            Tag::List(start) => {
                let state = ListState {
                    ordered: start.is_some(),
                    start,
                };
                self.list_stack.push(state);
                self.line_break();
                if let Some(start) = start.filter(|start| *start != 1) {
                    let indent = self.list_indent();
                    self.write(&format!("{indent}#set enum(start: {start})\n"));
                }
            }
            Tag::Item => {
                self.line_break();
                let indent = self.list_indent();
                self.write(&indent);
                let ordered = self.list_stack.last().is_some_and(|state| state.ordered);
                self.write(if ordered { "+ " } else { "- " });
            }
            Tag::Table(alignments) => {
                let inline = self.blockquote_depth > 0;
                if !inline {
                    self.flush_content();
                }
                self.table = Some(Table {
                    alignments,
                    header: Vec::new(),
                    rows: Vec::new(),
                    row: Vec::new(),
                    inline,
                });
            }
            Tag::TableHead => {
                if let Some(table) = &mut self.table {
                    table.row.clear();
                }
            }
            Tag::TableRow => {
                if let Some(table) = &mut self.table {
                    table.row.clear();
                }
            }
            Tag::TableCell if self.table.is_some() => {
                self.cell = Some(String::new());
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => self.write("\n\n"),
            TagEnd::Paragraph => self.newline(),
            TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript => self.write("]"),
            TagEnd::Link if self.link_stack.pop().unwrap_or(false) => self.write("]"),
            TagEnd::Link => {}
            TagEnd::BlockQuote(_) => {
                self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
                self.write("\n]\n\n");
            }
            TagEnd::CodeBlock => {
                if let Some(code) = self.code_block.take() {
                    let fence = code_fence(&code.text);
                    let body = code.text.strip_suffix('\n').unwrap_or(&code.text);
                    self.write(&format!("\n{fence}{}\n{body}\n{fence}\n\n", code.lang));
                }
            }
            TagEnd::List(_) => {
                if let Some(state) = self.list_stack.pop()
                    && state.start.is_some_and(|start| start != 1)
                {
                    let restore = self
                        .list_stack
                        .last()
                        .and_then(|parent| parent.start)
                        .unwrap_or(1);
                    let indent = self.list_indent();
                    self.write(&format!("{indent}#set enum(start: {restore})\n"));
                }
                if self.list_stack.is_empty() {
                    self.line_break();
                }
            }
            TagEnd::Item => self.line_break(),
            TagEnd::TableHead => {
                if let Some(table) = &mut self.table {
                    let row = std::mem::take(&mut table.row);
                    table.header.push(row);
                }
            }
            TagEnd::TableRow => {
                if let Some(table) = &mut self.table {
                    let row = std::mem::take(&mut table.row);
                    table.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                if let (Some(cell), Some(table)) = (self.cell.take(), &mut self.table) {
                    table.row.push(cell);
                }
            }
            TagEnd::Table => self.finish_table(),
            _ => {}
        }
    }

    fn finish_table(&mut self) {
        let Some(table) = self.table.take() else {
            return;
        };

        let columns = table
            .header
            .iter()
            .chain(table.rows.iter())
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(table.alignments.len())
            .max(1);

        self.write("\n#table(\n");
        self.write(&format!("  columns: {columns},\n"));

        let alignments = (0..columns)
            .map(|i| match table.alignments.get(i) {
                None | Some(Alignment::None) | Some(Alignment::Left) => "left",
                Some(Alignment::Center) => "center",
                Some(Alignment::Right) => "right",
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.write(&format!("  align: ({alignments}),\n"));

        if !table.header.is_empty() {
            self.write("  table.header(");
            for row in &table.header {
                for cell in row {
                    self.write(&format!("[{}], ", cell.trim()));
                }
            }
            self.write("),\n");
        }

        for row in &table.rows {
            self.write("  ");
            for cell in row {
                self.write(&format!("[{}], ", cell.trim()));
            }
            self.write("\n");
        }

        self.write(")\n\n");

        let data = self
            .tables
            .next()
            .unwrap_or(MarkdownTable { rows: Vec::new() });
        if table.inline {
            return;
        }
        self.elements.push(Element::Table {
            content: std::mem::take(&mut self.out),
            data,
        });
    }

    fn finish(mut self) -> (Vec<Element>, bool) {
        self.flush_content();
        (self.elements, self.has_alerts)
    }
}

fn alert_kind_name(kind: BlockQuoteKind) -> &'static str {
    match kind {
        BlockQuoteKind::Note => "note",
        BlockQuoteKind::Tip => "tip",
        BlockQuoteKind::Important => "important",
        BlockQuoteKind::Warning => "warning",
        BlockQuoteKind::Caution => "caution",
    }
}

/// Escape text for Typst markup.
pub(crate) fn escape_typst(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        // Front-matter values are wrapped in `[...]`, where a leading `4.`
        // would otherwise become an enumeration. Escaping `.` keeps values
        // like dates as plain text.
        if is_typst_special(ch) || ch == '.' {
            out.push('\\');
        }
        out.push(if ch == '\n' { ' ' } else { ch });
    }
    out
}

/// Collapse runs of whitespace in a plain-text table cell.
fn normalize_cell(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_typst_special(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '#' | '*' | '_' | '`' | '$' | '@' | '<' | '[' | ']' | '~' | '-' | '+' | '=' | '/'
    )
}

/// A raw code fence must be longer than any backtick run in its content.
fn code_fence(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat(longest.max(2) + 1)
}

/// Escape a string literal passed to a Typst function such as `link("...")`.
fn escape_string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Inline raw text, using enough backticks to survive backticks in the content.
fn raw_inline(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest + 1);
    format!("{fence}{text}{fence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(input: &str) -> Document {
        parse(input).expect("parse markdown")
    }

    #[test]
    fn nested_frontmatter() {
        let doc = parse_ok(
            "---\nsender:\n  name: Anja Ahlsen\n  address:\n    street: Beispielallee 28\nrecipient:\n  name: Finanzamt\n---\n\nHello",
        );
        let sender = doc
            .meta
            .get("sender")
            .and_then(MetaValue::as_mapping)
            .expect("sender mapping");
        assert_eq!(
            sender.get("name").and_then(MetaValue::as_scalar),
            Some("Anja Ahlsen")
        );
        let address = sender
            .get("address")
            .and_then(MetaValue::as_mapping)
            .expect("nested address mapping");
        assert_eq!(
            address.get("street").and_then(MetaValue::as_scalar),
            Some("Beispielallee 28")
        );
    }

    #[test]
    fn invalid_frontmatter_is_an_error() {
        let error = parse("---\nsender: [unterminated\n---\n").expect_err("invalid YAML");
        assert!(
            error.to_string().contains("invalid front matter"),
            "{error}"
        );
    }

    #[test]
    fn nested_values_render_as_typst() {
        let value = MetaValue::Mapping(std::collections::BTreeMap::from([
            (
                "tags".to_owned(),
                MetaValue::Sequence(vec![
                    MetaValue::Scalar("a".to_owned()),
                    MetaValue::Scalar("b".to_owned()),
                ]),
            ),
            (
                "empty".to_owned(),
                MetaValue::Mapping(std::collections::BTreeMap::new()),
            ),
            (
                "child".to_owned(),
                MetaValue::Mapping(std::collections::BTreeMap::from([(
                    "name".to_owned(),
                    MetaValue::Scalar("Jane".to_owned()),
                )])),
            ),
        ]));
        assert_eq!(
            value.to_typst(),
            "(child: (name: [Jane]), empty: (:), tags: ([a], [b],))"
        );
    }

    #[test]
    fn frontmatter_sequences_and_nulls() {
        let doc = parse_ok("---\ntags: [a, b]\nempty:\n---\n");
        assert_eq!(
            doc.meta.get("tags"),
            Some(&MetaValue::Sequence(vec![
                MetaValue::Scalar("a".to_owned()),
                MetaValue::Scalar("b".to_owned()),
            ]))
        );
        assert_eq!(doc.meta.get("empty"), Some(&MetaValue::Null));
    }

    #[test]
    fn frontmatter_and_basics() {
        let doc = parse_ok(
            "---\ntitle: Hello\ndate: 2026-01-02\n---\n\n# Hi\n\nSome *text* and `code`.\n",
        );
        assert_eq!(
            doc.meta.get("title").and_then(MetaValue::as_scalar),
            Some("Hello")
        );
        assert_eq!(
            doc.meta.get("date").and_then(MetaValue::as_scalar),
            Some("2026-01-02")
        );
        assert!(doc.body().contains("= Hi"));
        assert!(doc.body().contains("#emph[text]"));
        assert!(doc.body().contains("`code`"));
    }

    #[test]
    fn table() {
        let doc = parse_ok("| A | B |\n| :- | -: |\n| 1 | 2 |\n");
        assert!(doc.body().contains("#table("));
        assert!(doc.body().contains("table.header"));
        assert!(doc.body().contains("align: (left, right)"));
    }

    #[test]
    fn escapes_a_leading_slash() {
        let doc = parse_ok("/ hello");
        assert!(doc.body().contains("\\/ hello"), "body: {}", doc.body());
    }

    #[test]
    fn code_block_uses_a_fence_longer_than_its_content() {
        let doc = parse_ok("````\n```\ninside\n```\n````\n");
        assert!(
            doc.body().contains("````\n```\ninside\n```\n````"),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn nested_lists_keep_their_indentation() {
        let doc = parse_ok("- parent\n  - child\n    1. one\n    2. two\n- sibling");
        let body = doc.body();
        assert!(body.contains("\n- parent"), "{body}");
        assert!(body.contains("\n  - child"), "{body}");
        assert!(body.contains("\n    + one"), "{body}");
        assert!(!body.contains("#set enum(start: 1)"), "{body}");
    }

    #[test]
    fn list_items_are_not_separated_by_blank_lines() {
        let doc = parse_ok("- a\n  - b\n- c\n\n1. one\n2. two\n");
        let body = doc.body();
        for marker in ["- ", "  - ", "+ ", "  + "] {
            assert!(
                !body.contains(&format!("\n\n{marker}")),
                "empty line before an item ({marker}): {body}"
            );
        }
    }

    #[test]
    fn preserves_an_ordered_list_start() {
        let doc = parse_ok("3. third\n4. fourth\n");
        assert!(
            doc.body().contains("#set enum(start: 3)"),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn github_alerts_use_the_alert_helper() {
        for (marker, kind) in [
            ("NOTE", "note"),
            ("TIP", "tip"),
            ("IMPORTANT", "important"),
            ("WARNING", "warning"),
            ("CAUTION", "caution"),
        ] {
            let doc = parse_ok(&format!("> [!{marker}]\n> Body"));
            assert!(doc.has_alerts(), "{marker}");
            assert!(
                doc.body()
                    .contains(&format!("#_tome_md_github_alert(\"{kind}\")[")),
                "{}",
                doc.body()
            );
        }
    }

    #[test]
    fn lowercase_github_alerts_are_recognized() {
        let doc = parse_ok("> [!note]\n> Body");
        assert!(doc.has_alerts());
        assert!(doc.body().contains("#_tome_md_github_alert(\"note\")["));
    }

    #[test]
    fn ordinary_blockquotes_are_unchanged() {
        let doc = parse_ok("> A normal quote");
        assert!(!doc.has_alerts());
        assert!(doc.body().contains("#quote(block: true)["));
    }

    #[test]
    fn malformed_alerts_are_normal_quotes() {
        let doc = parse_ok("> [!BOGUS]\n> Body");
        assert!(!doc.has_alerts());
        assert!(doc.body().contains("#quote(block: true)["));
    }

    #[test]
    fn alerts_can_contain_lists_and_tables() {
        let doc = parse_ok(
            "> [!WARNING]\n> Intro\n>\n> - one\n> - two\n>\n> | A | B |\n> | --- | --- |\n> | 1 | 2 |",
        );
        assert!(doc.has_alerts());
        assert!(doc.body().contains("#_tome_md_github_alert(\"warning\")["));
        assert!(doc.body().contains("- one"));
        assert!(doc.body().contains("#table("));
        assert!(doc.body().contains("]"), "{}", doc.body());
    }

    #[test]
    fn inline_sup_and_sub_tags() {
        let doc = parse_ok("H<sub>2</sub>O and 15<sup>th</sup>");
        assert!(doc.body().contains("#sub[2]"), "body: {}", doc.body());
        assert!(doc.body().contains("#super[th]"), "body: {}", doc.body());
    }

    #[test]
    fn images_keep_their_alt_text() {
        let doc = parse_ok("![Pumpkin soup](assets/pumpkin-soup.svg)");
        assert!(
            doc.body()
                .contains(r#"#image("assets/pumpkin-soup.svg", alt: "Pumpkin soup")"#),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn images_without_alt_text() {
        let doc = parse_ok("![](assets/photo.png)");
        assert!(
            doc.body().contains(r#"#image("assets/photo.png")"#),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn empty_link_destination_is_rendered_as_text() {
        let doc = parse_ok("[x]()");
        assert!(!doc.body().contains("#link(\"\")"), "body: {}", doc.body());
        assert!(doc.body().contains('x'), "body: {}", doc.body());
    }

    #[test]
    fn strips_a_bom_before_front_matter() {
        let doc = parse_ok("\u{feff}---\ntitle: BOM\n---\n\nHello");
        assert_eq!(
            doc.meta.get("title").and_then(MetaValue::as_scalar),
            Some("BOM")
        );
        assert!(!doc.body().contains("title: BOM"), "body: {}", doc.body());
    }

    #[test]
    fn collects_tables_as_plain_text() {
        let doc = parse_ok(
            "| Std. | Tätigkeit |\n| ---: | --- |\n| 4,0 | **Setup** [PR](https://example.com) |\n",
        );
        let tables = doc.tables().collect::<Vec<_>>();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].rows[0], vec!["Std.", "Tätigkeit"]);
        assert_eq!(tables[0].rows[1], vec!["4,0", "Setup PR"]);
    }

    #[test]
    fn collects_multiple_tables() {
        let doc = parse_ok("| A |\n| --- |\n| 1 |\n\n| B |\n| --- |\n| 2 |\n");
        let tables = doc.tables().collect::<Vec<_>>();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].rows[1], vec!["1"]);
        assert_eq!(tables[1].rows[1], vec!["2"]);
    }

    #[test]
    fn elements_keep_tables_in_document_order() {
        let doc = parse_ok("Hello\n\n| A |\n| --- |\n| 1 |\n\nWorld\n");
        let kinds = doc
            .elements
            .iter()
            .map(|element| match element {
                Element::Content(_) => "content",
                Element::Table { .. } => "table",
            })
            .collect::<Vec<_>>();
        assert_eq!(kinds, ["content", "table", "content"]);
    }
}
