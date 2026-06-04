use markdown::mdast::{AlignKind, Node};
use unicode_width::UnicodeWidthStr;

use super::{render_inlines, render_node};

pub(super) fn render_table(children: &[Node], align: &[AlignKind], input: &str) -> String {
    let rows = children
        .iter()
        .filter_map(|child| match child {
            Node::TableRow(row) => Some(render_table_cells(&row.children, input)),
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

pub(super) fn render_unpadded_table_row(children: &[Node], input: &str) -> String {
    format!(
        "| {} |",
        children
            .iter()
            .map(|child| render_node(child, input))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

fn render_table_cells(children: &[Node], input: &str) -> Vec<String> {
    children
        .iter()
        .map(|child| match child {
            Node::TableCell(cell) => render_inlines(&cell.children, input),
            _ => render_node(child, input),
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
