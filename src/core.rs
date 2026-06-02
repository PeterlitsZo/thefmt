use crate::error::Error;
use markdown::mdast::{AlignKind, Node, ReferenceKind};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MAX_LINE_WIDTH: usize = 80;

pub fn format_markdown(input: &str) -> Result<String, Error> {
    let mdast = markdown::to_mdast(input, &markdown::ParseOptions::gfm()).map_err(Error::Parse)?;
    let mut output = render_node(&mdast);

    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
}

fn render_node(node: &Node) -> String {
    match node {
        Node::Root(root) => render_root_blocks(&root.children),
        Node::Blockquote(blockquote) => render_blockquote(&blockquote.children),
        Node::FootnoteDefinition(footnote) => {
            format!(
                "[^{}]: {}",
                footnote.identifier,
                render_blocks(&footnote.children)
            )
        }
        Node::MdxJsxFlowElement(element) => render_blocks(&element.children),
        Node::List(list) => render_list(&list.children, list.ordered, list.start, list.spread, ""),
        Node::MdxjsEsm(esm) => esm.value.clone(),
        Node::Toml(toml) => format!("+++\n{}\n+++", trim_trailing_newlines(&toml.value)),
        Node::Yaml(yaml) => format!("---\n{}\n---", trim_trailing_newlines(&yaml.value)),
        Node::Break(_) => "\\\n".to_string(),
        Node::InlineCode(code) => format!("`{}`", code.value),
        Node::InlineMath(math) => format!("${}$", math.value),
        Node::Delete(delete) => format!("~~{}~~", render_inlines(&delete.children)),
        Node::Emphasis(emphasis) => format!("*{}*", render_inlines(&emphasis.children)),
        Node::MdxTextExpression(expression) => format!("{{{}}}", expression.value),
        Node::FootnoteReference(reference) => format!("[^{}]", reference.identifier),
        Node::Html(html) => html.value.clone(),
        Node::Image(image) => {
            render_resource(&format!("![{}]", image.alt), &image.url, &image.title)
        }
        Node::ImageReference(image) => render_reference(
            &format!("![{}]", image.alt),
            &image.identifier,
            image.reference_kind,
        ),
        Node::MdxJsxTextElement(element) => render_inlines(&element.children),
        Node::Link(link) => render_resource(
            &format!("[{}]", render_inlines(&link.children)),
            &link.url,
            &link.title,
        ),
        Node::LinkReference(link) => render_reference(
            &format!("[{}]", render_inlines(&link.children)),
            &link.identifier,
            link.reference_kind,
        ),
        Node::Strong(strong) => format!("**{}**", render_inlines(&strong.children)),
        Node::Text(text) => text.value.clone(),
        Node::Code(code) => {
            render_code_block(code.lang.as_deref(), code.meta.as_deref(), &code.value)
        }
        Node::Math(math) => format!("$$\n{}\n$$", trim_trailing_newlines(&math.value)),
        Node::MdxFlowExpression(expression) => format!("{{{}}}", expression.value),
        Node::Heading(heading) => {
            format!(
                "{} {}",
                "#".repeat(heading.depth.into()),
                render_inlines(&heading.children)
            )
        }
        Node::Table(table) => render_table(&table.children, &table.align),
        Node::ThematicBreak(_) => "---".to_string(),
        Node::TableRow(row) => render_unpadded_table_row(&row.children),
        Node::TableCell(cell) => render_inlines(&cell.children),
        Node::ListItem(item) => render_blocks(&item.children),
        Node::Definition(definition) => {
            let mut output = format!("[{}]: {}", definition.identifier, definition.url);
            if let Some(title) = &definition.title {
                output.push_str(&format!(" \"{}\"", title));
            }
            output
        }
        Node::Paragraph(paragraph) => render_inlines(&paragraph.children),
    }
}

fn render_blocks(children: &[Node]) -> String {
    children
        .iter()
        .map(render_node)
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_root_blocks(children: &[Node]) -> String {
    let mut output = String::new();
    let mut previous: Option<&Node> = None;

    for (index, child) in children.iter().enumerate() {
        let block = match child {
            Node::Paragraph(paragraph) => render_wrapped_paragraph(&paragraph.children, "", ""),
            Node::ThematicBreak(_) if thematic_break_should_expand(children, index) => {
                "-".repeat(MAX_LINE_WIDTH)
            }
            _ => render_node(child),
        };

        if block.is_empty() {
            continue;
        }

        if let Some(previous) = previous {
            output.push_str(if should_compact_root_blocks(previous, child) {
                "\n"
            } else {
                "\n\n"
            });
        }

        output.push_str(&block);
        previous = Some(child);
    }

    output
}

fn render_inlines(children: &[Node]) -> String {
    children.iter().map(render_node).collect()
}

fn render_blockquote(children: &[Node]) -> String {
    children
        .iter()
        .map(|child| match child {
            Node::Paragraph(paragraph) => render_wrapped_paragraph(&paragraph.children, "> ", "> "),
            _ => render_prefixed_block(&render_node(child), "> ", "> "),
        })
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n>\n")
}

fn render_list(
    children: &[Node],
    ordered: bool,
    start: Option<u32>,
    spread: bool,
    base_prefix: &str,
) -> String {
    let start = start.unwrap_or(1);
    let mut output = String::new();

    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            output.push('\n');
            if spread {
                output.push('\n');
            }
        }

        let marker = if ordered {
            let number = if spread { start } else { start + index as u32 };
            format!("{number}.")
        } else {
            "-".to_string()
        };
        output.push_str(&render_list_item(child, &marker, base_prefix));
    }

    output
}

fn render_list_item(node: &Node, marker: &str, base_prefix: &str) -> String {
    let (children, checked) = match node {
        Node::ListItem(item) => (&item.children, item.checked),
        _ => return format!("{base_prefix}{marker} {}", render_node(node)),
    };
    let indent = format!("{base_prefix}{}", " ".repeat(marker.len() + 1));
    let checkbox = checked.map(|checked| if checked { "[x] " } else { "[ ] " });
    let first_prefix = format!("{base_prefix}{marker} {}", checkbox.unwrap_or(""));
    let first_continuation = " ".repeat(first_prefix.len());
    let following_prefix = indent.as_str();
    let mut output = String::new();

    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            output.push('\n');
            if should_insert_blank_line_between_blocks(&children[index - 1], child) {
                output.push('\n');
            }
        }

        let rendered = match child {
            Node::Paragraph(paragraph) if index == 0 => {
                render_wrapped_paragraph(&paragraph.children, &first_prefix, &first_continuation)
            }
            Node::Paragraph(paragraph) => {
                render_wrapped_paragraph(&paragraph.children, following_prefix, following_prefix)
            }
            Node::List(list) if index == 0 => render_list(
                &list.children,
                list.ordered,
                list.start,
                list.spread,
                following_prefix,
            ),
            Node::List(list) => render_list(
                &list.children,
                list.ordered,
                list.start,
                list.spread,
                following_prefix,
            ),
            _ if index == 0 => render_prefixed_block(&render_node(child), &first_prefix, following_prefix),
            _ => render_prefixed_block(&render_node(child), following_prefix, following_prefix),
        };

        output.push_str(&rendered);
    }

    if output.is_empty() {
        return marker.to_string();
    }

    output
}

fn render_code_block(lang: Option<&str>, meta: Option<&str>, value: &str) -> String {
    let mut opening = String::from("```");
    if let Some(lang) = lang {
        opening.push_str(lang);
    }
    if let Some(meta) = meta {
        opening.push(' ');
        opening.push_str(meta);
    }

    format!("{opening}\n{}\n```", trim_trailing_newlines(value))
}

fn render_resource(label: &str, url: &str, title: &Option<String>) -> String {
    let mut output = format!("{label}({url}");
    if let Some(title) = title {
        output.push_str(&format!(" \"{title}\""));
    }
    output.push(')');
    output
}

fn render_reference(label: &str, identifier: &str, reference_kind: ReferenceKind) -> String {
    match reference_kind {
        ReferenceKind::Shortcut => label.to_string(),
        ReferenceKind::Collapsed => format!("{label}[]"),
        ReferenceKind::Full => format!("{label}[{identifier}]"),
    }
}

fn render_table(children: &[Node], align: &[AlignKind]) -> String {
    let rows = children
        .iter()
        .filter_map(|child| match child {
            Node::TableRow(row) => Some(render_table_cells(&row.children)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return String::new();
    }

    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths = (0..column_count)
        .map(|index| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(|cell| cell_display_width(cell))
                .max()
                .unwrap_or(0)
                .max(3)
        })
        .collect::<Vec<_>>();

    let separator = (0..column_count)
        .map(|index| {
            render_separator_cell(widths[index], align.get(index).unwrap_or(&AlignKind::None))
        })
        .collect::<Vec<_>>()
        .join(" | ");

    let mut output = render_table_row(&rows[0], &widths);
    output.push('\n');
    output.push_str("| ");
    output.push_str(&separator);
    output.push_str(" |");

    for row in rows.iter().skip(1) {
        output.push('\n');
        output.push_str(&render_table_row(row, &widths));
    }

    output
}

fn render_table_cells(children: &[Node]) -> Vec<String> {
    children
        .iter()
        .map(|child| match child {
            Node::TableCell(cell) => render_inlines(&cell.children),
            _ => render_node(child),
        })
        .collect()
}

fn render_table_row(cells: &[String], widths: &[usize]) -> String {
    let padded = widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let cell = cells.get(index).map(String::as_str).unwrap_or("");
            pad_cell(cell, *width)
        })
        .collect::<Vec<_>>()
        .join(" | ");

    format!("| {padded} |")
}

fn render_unpadded_table_row(children: &[Node]) -> String {
    format!(
        "| {} |",
        children
            .iter()
            .map(render_node)
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

fn render_separator_cell(width: usize, align: &AlignKind) -> String {
    match align {
        AlignKind::Left => format!(":{}", "-".repeat(width.saturating_sub(1).max(2))),
        AlignKind::Right => format!("{}:", "-".repeat(width.saturating_sub(1).max(2))),
        AlignKind::Center => format!(":{}:", "-".repeat(width.saturating_sub(2).max(1))),
        AlignKind::None => "-".repeat(width.max(3)),
    }
}

fn pad_cell(cell: &str, width: usize) -> String {
    let padding = width.saturating_sub(cell_display_width(cell));
    format!("{cell}{}", " ".repeat(padding))
}

fn cell_display_width(cell: &str) -> usize {
    UnicodeWidthStr::width(cell)
}

fn trim_trailing_newlines(value: &str) -> &str {
    value.trim_end_matches(['\n', '\r'])
}

fn thematic_break_should_expand(children: &[Node], index: usize) -> bool {
    let previous = index.checked_sub(1).and_then(|idx| children.get(idx));
    let next = children.get(index + 1);
    matches!(previous, Some(Node::List(_))) && matches!(next, Some(Node::List(_)))
}

fn should_insert_blank_line_between_blocks(previous: &Node, current: &Node) -> bool {
    is_code_like_block(previous) || is_code_like_block(current)
}

fn is_code_like_block(node: &Node) -> bool {
    matches!(node, Node::Code(_) | Node::Math(_))
}

#[derive(Clone)]
enum InlineFragment {
    BreakableText(String),
    Atomic(String),
}

#[derive(Clone)]
struct LinePart {
    text: String,
    breakable: bool,
}

#[derive(Clone, Copy)]
struct BreakPoint {
    part_index: usize,
    byte_index: usize,
}

#[derive(Default)]
struct LineState {
    parts: Vec<LinePart>,
    width: usize,
    last_space_break: Option<BreakPoint>,
    last_char_break: Option<BreakPoint>,
}

fn render_wrapped_paragraph(children: &[Node], first_prefix: &str, continuation_prefix: &str) -> String {
    if children.iter().any(|child| matches!(child, Node::Break(_))) {
        return render_prefixed_block(&render_inlines(children), first_prefix, continuation_prefix);
    }

    let fragments = render_inline_fragments(children);
    wrap_fragments(&fragments, first_prefix, continuation_prefix)
}

fn render_inline_fragments(children: &[Node]) -> Vec<InlineFragment> {
    children
        .iter()
        .filter_map(|child| match child {
            Node::Text(text) => {
                let normalized = normalize_breakable_text(&text.value);
                if normalized.is_empty() {
                    None
                } else {
                    Some(InlineFragment::BreakableText(normalized))
                }
            }
            _ => {
                let rendered = render_node(child);
                if rendered.is_empty() {
                    None
                } else {
                    Some(InlineFragment::Atomic(rendered))
                }
            }
        })
        .collect()
}

fn normalize_breakable_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_space = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(ch);
            last_was_space = false;
        }
    }

    normalized
}

fn wrap_fragments(fragments: &[InlineFragment], first_prefix: &str, continuation_prefix: &str) -> String {
    let mut lines = Vec::new();
    let mut line = LineState::default();
    let mut current_prefix = first_prefix;

    for fragment in fragments {
        match fragment {
            InlineFragment::BreakableText(text) => {
                for ch in text.chars() {
                    append_breakable_char(
                        &mut lines,
                        &mut line,
                        &mut current_prefix,
                        continuation_prefix,
                        ch,
                    );
                }
            }
            InlineFragment::Atomic(text) => append_atomic_fragment(
                &mut lines,
                &mut line,
                &mut current_prefix,
                continuation_prefix,
                text,
            ),
        }
    }

    flush_line(&mut lines, &mut line, current_prefix);

    lines.join("\n")
}

fn append_breakable_char<'a>(
    lines: &mut Vec<String>,
    line: &mut LineState,
    current_prefix: &mut &'a str,
    continuation_prefix: &'a str,
    ch: char,
) {
    if line.width == 0 && ch == ' ' {
        return;
    }

    let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
    let prefix_width = UnicodeWidthStr::width(*current_prefix);

    if prefix_width + line.width + char_width <= MAX_LINE_WIDTH || line.width == 0 {
        push_breakable_char(line, ch, char_width);
        return;
    }

    if let Some(breakpoint) = choose_breakpoint(line) {
        let remainder = split_line_at_breakpoint(line, breakpoint);
        flush_line(lines, line, current_prefix);
        *current_prefix = continuation_prefix;
        *line = remainder;
        append_breakable_char(lines, line, current_prefix, continuation_prefix, ch);
        return;
    }

    flush_line(lines, line, current_prefix);
    *current_prefix = continuation_prefix;
    append_breakable_char(lines, line, current_prefix, continuation_prefix, ch);
}

fn append_atomic_fragment<'a>(
    lines: &mut Vec<String>,
    line: &mut LineState,
    current_prefix: &mut &'a str,
    continuation_prefix: &'a str,
    text: &str,
) {
    let text_width = UnicodeWidthStr::width(text);
    let prefix_width = UnicodeWidthStr::width(*current_prefix);

    if line.width > 0 && prefix_width + line.width + text_width > MAX_LINE_WIDTH {
        flush_line(lines, line, current_prefix);
        *current_prefix = continuation_prefix;
    }

    line.parts.push(LinePart {
        text: text.to_string(),
        breakable: false,
    });
    line.width += text_width;
}

fn push_breakable_char(line: &mut LineState, ch: char, char_width: usize) {
    if let Some(last) = line.parts.last_mut() {
        if last.breakable {
            last.text.push(ch);
        } else {
            line.parts.push(LinePart {
                text: ch.to_string(),
                breakable: true,
            });
        }
    } else {
        line.parts.push(LinePart {
            text: ch.to_string(),
            breakable: true,
        });
    }

    line.width += char_width;
    let part_index = line.parts.len() - 1;
    let byte_index = line.parts[part_index].text.len();
    if can_break_after_char(ch) {
        line.last_char_break = Some(BreakPoint {
            part_index,
            byte_index,
        });
    }

    if ch == ' ' {
        line.last_space_break = Some(BreakPoint {
            part_index,
            byte_index,
        });
    }
}

fn split_line_at_breakpoint(line: &mut LineState, breakpoint: BreakPoint) -> LineState {
    let mut left_parts = Vec::new();
    let mut remainder_parts = Vec::new();

    for (index, part) in line.parts.iter().enumerate() {
        if index < breakpoint.part_index {
            left_parts.push(part.clone());
            continue;
        }

        if index == breakpoint.part_index {
            let head = &part.text[..breakpoint.byte_index];
            if !head.is_empty() {
                left_parts.push(LinePart {
                    text: head.to_string(),
                    breakable: part.breakable,
                });
            }
            let tail = &part.text[breakpoint.byte_index..];
            if !tail.is_empty() {
                remainder_parts.push(LinePart {
                    text: tail.to_string(),
                    breakable: part.breakable,
                });
            }
        } else {
            remainder_parts.push(part.clone());
        }
    }

    *line = build_line_state(left_parts);
    build_line_state(remainder_parts)
}

fn build_line_state(parts: Vec<LinePart>) -> LineState {
    let mut line = LineState::default();

    for part in parts {
        if part.text.is_empty() {
            continue;
        }

        let part_index = line.parts.len();
        if part.breakable {
            let mut byte_index = 0;
            for ch in part.text.chars() {
                byte_index += ch.len_utf8();
                line.width += UnicodeWidthChar::width(ch).unwrap_or(0);
                if can_break_after_char(ch) {
                    line.last_char_break = Some(BreakPoint {
                        part_index,
                        byte_index,
                    });
                }
                if ch == ' ' {
                    line.last_space_break = Some(BreakPoint {
                        part_index,
                        byte_index,
                    });
                }
            }
        } else {
            line.width += UnicodeWidthStr::width(part.text.as_str());
        }

        line.parts.push(part);
    }

    line
}

fn flush_line(lines: &mut Vec<String>, line: &mut LineState, prefix: &str) {
    let content = render_line_parts(&line.parts);
    if content.is_empty() {
        return;
    }

    lines.push(format!("{prefix}{content}"));
    *line = LineState::default();
}

fn render_line_parts(parts: &[LinePart]) -> String {
    let mut content = parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<String>();
    while content.ends_with(' ') {
        content.pop();
    }
    content
}

fn choose_breakpoint(line: &LineState) -> Option<BreakPoint> {
    match (line.last_space_break, line.last_char_break) {
        (Some(space), Some(character)) if breakpoint_precedes(space, character) => Some(character),
        (Some(space), _) => Some(space),
        (None, Some(character)) => Some(character),
        (None, None) => None,
    }
}

fn breakpoint_precedes(left: BreakPoint, right: BreakPoint) -> bool {
    (left.part_index, left.byte_index) < (right.part_index, right.byte_index)
}

fn can_break_after_char(ch: char) -> bool {
    !ch.is_whitespace() && UnicodeWidthChar::width(ch).unwrap_or(0) > 1
}

fn render_prefixed_block(block: &str, first_prefix: &str, continuation_prefix: &str) -> String {
    let mut lines = block.lines();
    let mut output = match lines.next() {
        Some(first) if !first.is_empty() => format!("{first_prefix}{first}"),
        Some(_) => first_prefix.to_string(),
        None => return String::new(),
    };

    for line in lines {
        output.push('\n');
        if line.is_empty() {
            continue;
        }
        output.push_str(continuation_prefix);
        output.push_str(line);
    }

    output
}

fn should_compact_root_blocks(previous: &Node, current: &Node) -> bool {
    matches!(previous, Node::Definition(_)) && matches!(current, Node::Definition(_))
}
