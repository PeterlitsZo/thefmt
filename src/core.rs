use crate::error::Error;
use markdown::mdast::{AlignKind, Node, ReferenceKind};
use unicode_width::UnicodeWidthStr;

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
        Node::Root(root) => render_blocks(&root.children),
        Node::Blockquote(blockquote) => render_blockquote(&blockquote.children),
        Node::FootnoteDefinition(footnote) => {
            format!(
                "[^{}]: {}",
                footnote.identifier,
                render_blocks(&footnote.children)
            )
        }
        Node::MdxJsxFlowElement(element) => render_blocks(&element.children),
        Node::List(list) => render_list(&list.children, list.ordered, list.start, list.spread),
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

fn render_inlines(children: &[Node]) -> String {
    children.iter().map(render_node).collect()
}

fn render_blockquote(children: &[Node]) -> String {
    render_blocks(children)
        .lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_string()
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_list(children: &[Node], ordered: bool, start: Option<u32>, spread: bool) -> String {
    let start = start.unwrap_or(1);
    let separator = if spread { "\n\n" } else { "\n" };

    children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let marker = if ordered {
                format!("{}.", start + index as u32)
            } else {
                "-".to_string()
            };
            render_list_item(child, &marker)
        })
        .collect::<Vec<_>>()
        .join(separator)
}

fn render_list_item(node: &Node, marker: &str) -> String {
    let (children, checked) = match node {
        Node::ListItem(item) => (&item.children, item.checked),
        _ => return format!("{marker} {}", render_node(node)),
    };

    let mut content = render_blocks(children);
    if let Some(checked) = checked {
        let checkbox = if checked { "[x] " } else { "[ ] " };
        content.insert_str(0, checkbox);
    }

    if content.is_empty() {
        return marker.to_string();
    }

    let indent = " ".repeat(marker.len() + 1);
    let mut lines = content.lines();
    let mut output = format!("{marker} {}", lines.next().unwrap_or_default());

    for line in lines {
        output.push('\n');
        output.push_str(&indent);
        output.push_str(line);
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
