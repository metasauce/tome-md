//! Markdown -> Typst conversion with a small GFM subset.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// A parsed document: front-matter values and the rendered element list.
pub struct Document {
    pub meta: std::collections::BTreeMap<String, String>,
    pub elements: Vec<Element>,
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
pub fn parse(markdown: &str) -> Document {
    let (meta, body) = split_frontmatter(markdown);
    Document {
        meta,
        elements: to_elements(body),
    }
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

fn split_frontmatter(input: &str) -> (std::collections::BTreeMap<String, String>, &str) {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut meta = std::collections::BTreeMap::new();
    let Some((first, mut rest)) = input.split_once('\n') else {
        return (meta, input);
    };
    if first.trim() != "---" {
        return (meta, input);
    }

    loop {
        let Some((line, after)) = rest.split_once('\n') else {
            if rest.trim() == "---" {
                return (meta, "");
            }
            return (std::collections::BTreeMap::new(), input);
        };

        if line.trim() == "---" {
            return (meta, after.trim_start_matches(['\r', '\n']));
        }

        if let Some((key, value)) = line.trim().split_once(':') {
            let key = key.trim();
            if !key.is_empty() {
                meta.insert(
                    key.to_owned(),
                    value.trim().trim_matches(['"', '\'']).to_owned(),
                );
            }
        }

        rest = after;
    }
}

fn to_elements(markdown: &str) -> Vec<Element> {
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

struct Table {
    alignments: Vec<Alignment>,
    header: Vec<Vec<String>>,
    rows: Vec<Vec<String>>,
    row: Vec<String>,
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

    fn event(&mut self, event: Event<'_>) {
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
            // Raw HTML and math are not rendered by this small pipeline.
            Event::Html(_) | Event::InlineHtml(_) => {}
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
            // Images are not fetched in the browser demo; show the alt text.
            Tag::Image { .. } => self.write("#emph["),
            Tag::BlockQuote(_) => self.write("\n#quote(block: true)[\n"),
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
                self.write("\n");
                if let Some(start) = start {
                    self.write(&format!("#set enum(start: {start})\n"));
                }
            }
            Tag::Item => {
                self.write("\n");
                for _ in 0..self.list_stack.len() {
                    self.write("  ");
                }
                let ordered = self.list_stack.last().is_some_and(|state| state.ordered);
                self.write(if ordered { "+ " } else { "- " });
            }
            Tag::Table(alignments) => {
                self.flush_content();
                self.table = Some(Table {
                    alignments,
                    header: Vec::new(),
                    rows: Vec::new(),
                    row: Vec::new(),
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
            | TagEnd::Subscript
            | TagEnd::Image => self.write("]"),
            TagEnd::Link if self.link_stack.pop().unwrap_or(false) => self.write("]"),
            TagEnd::Link => {}
            TagEnd::BlockQuote(_) => self.write("\n]\n\n"),
            TagEnd::CodeBlock => {
                if let Some(code) = self.code_block.take() {
                    let fence = code_fence(&code.text);
                    let body = code.text.strip_suffix('\n').unwrap_or(&code.text);
                    self.write(&format!("\n{fence}{}\n{body}\n{fence}\n\n", code.lang));
                }
            }
            TagEnd::List(_) => {
                if let Some(state) = self.list_stack.pop()
                    && state.ordered
                {
                    let restore = self
                        .list_stack
                        .last()
                        .and_then(|parent| parent.start)
                        .unwrap_or(1);
                    self.write(&format!("#set enum(start: {restore})\n"));
                }
                if self.list_stack.is_empty() {
                    self.write("\n");
                }
            }
            TagEnd::Item => self.write("\n"),
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
        self.elements.push(Element::Table {
            content: std::mem::take(&mut self.out),
            data,
        });
    }

    fn finish(mut self) -> Vec<Element> {
        self.flush_content();
        self.elements
    }
}

/// Escape text for Typst markup.
pub(crate) fn escape_typst(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if is_typst_special(ch) {
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

    #[test]
    fn frontmatter_and_basics() {
        let doc =
            parse("---\ntitle: Hello\ndate: 2026-01-02\n---\n\n# Hi\n\nSome *text* and `code`.\n");
        assert_eq!(doc.meta.get("title").map(String::as_str), Some("Hello"));
        assert_eq!(doc.meta.get("date").map(String::as_str), Some("2026-01-02"));
        assert!(doc.body().contains("= Hi"));
        assert!(doc.body().contains("#emph[text]"));
        assert!(doc.body().contains("`code`"));
    }

    #[test]
    fn table() {
        let doc = parse("| A | B |\n| :- | -: |\n| 1 | 2 |\n");
        assert!(doc.body().contains("#table("));
        assert!(doc.body().contains("table.header"));
        assert!(doc.body().contains("align: (left, right)"));
    }

    #[test]
    fn escapes_a_leading_slash() {
        let doc = parse("/ hello");
        assert!(doc.body().contains("\\/ hello"), "body: {}", doc.body());
    }

    #[test]
    fn code_block_uses_a_fence_longer_than_its_content() {
        let doc = parse("````\n```\ninside\n```\n````\n");
        assert!(
            doc.body().contains("````\n```\ninside\n```\n````"),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn preserves_an_ordered_list_start() {
        let doc = parse("3. third\n4. fourth\n");
        assert!(
            doc.body().contains("#set enum(start: 3)"),
            "body: {}",
            doc.body()
        );
    }

    #[test]
    fn empty_link_destination_is_rendered_as_text() {
        let doc = parse("[x]()");
        assert!(!doc.body().contains("#link(\"\")"), "body: {}", doc.body());
        assert!(doc.body().contains('x'), "body: {}", doc.body());
    }

    #[test]
    fn strips_a_bom_before_front_matter() {
        let doc = parse("\u{feff}---\ntitle: BOM\n---\n\nHello");
        assert_eq!(doc.meta.get("title").map(String::as_str), Some("BOM"));
        assert!(!doc.body().contains("title: BOM"), "body: {}", doc.body());
    }

    #[test]
    fn collects_tables_as_plain_text() {
        let doc = parse(
            "| Std. | Tätigkeit |\n| ---: | --- |\n| 4,0 | **Setup** [PR](https://example.com) |\n",
        );
        let tables = doc.tables().collect::<Vec<_>>();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].rows[0], vec!["Std.", "Tätigkeit"]);
        assert_eq!(tables[0].rows[1], vec!["4,0", "Setup PR"]);
    }

    #[test]
    fn collects_multiple_tables() {
        let doc = parse("| A |\n| --- |\n| 1 |\n\n| B |\n| --- |\n| 2 |\n");
        let tables = doc.tables().collect::<Vec<_>>();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].rows[1], vec!["1"]);
        assert_eq!(tables[1].rows[1], vec!["2"]);
    }

    #[test]
    fn elements_keep_tables_in_document_order() {
        let doc = parse("Hello\n\n| A |\n| --- |\n| 1 |\n\nWorld\n");
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
