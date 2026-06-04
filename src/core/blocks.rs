use markdown::mdast::Node;

use super::wrap::{render_prefixed_block, render_wrapped_paragraph};
use super::{render_node, MAX_LINE_WIDTH};

pub(super) fn render_blocks(children: &[Node], input: &str) -> String {
    children
        .iter()
        .map(|child| render_node(child, input))
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(super) fn render_root_blocks(children: &[Node], input: &str) -> String {
    let mut output = String::new();
    let mut previous: Option<&Node> = None;

    for (index, child) in children.iter().enumerate() {
        let block = match child {
            Node::Paragraph(paragraph) => {
                render_wrapped_paragraph(&paragraph.children, "", "", input)
            }
            Node::ThematicBreak(_) if thematic_break_should_expand(children, index) => {
                "-".repeat(MAX_LINE_WIDTH)
            }
            _ => render_node(child, input),
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

pub(super) fn render_blockquote(children: &[Node], input: &str) -> String {
    children
        .iter()
        .map(|child| match child {
            Node::Paragraph(paragraph) => {
                render_wrapped_paragraph(&paragraph.children, "> ", "> ", input)
            }
            _ => render_prefixed_block(&render_node(child, input), "> ", "> "),
        })
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n>\n")
}

pub(super) fn render_list(
    children: &[Node],
    ordered: bool,
    start: Option<u32>,
    spread: bool,
    base_prefix: &str,
    input: &str,
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
            ordered_list_marker(child, start + index as u32, input)
        } else {
            "-".to_string()
        };
        output.push_str(&render_list_item(child, &marker, base_prefix, input));
    }

    output
}

fn render_list_item(node: &Node, marker: &str, base_prefix: &str, input: &str) -> String {
    let (children, checked, spread) = match node {
        Node::ListItem(item) => (&item.children, item.checked, item.spread),
        _ => return format!("{base_prefix}{marker} {}", render_node(node, input)),
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
            if should_insert_blank_line_between_list_item_blocks(
                &children[index - 1],
                child,
                spread,
            ) {
                output.push('\n');
            }
        }

        let rendered = match child {
            Node::Paragraph(paragraph) if index == 0 => render_wrapped_paragraph(
                &paragraph.children,
                &first_prefix,
                &first_continuation,
                input,
            ),
            Node::Paragraph(paragraph) => render_wrapped_paragraph(
                &paragraph.children,
                following_prefix,
                following_prefix,
                input,
            ),
            Node::List(list) => render_list(
                &list.children,
                list.ordered,
                list.start,
                list.spread,
                following_prefix,
                input,
            ),
            _ if index == 0 => {
                render_prefixed_block(&render_node(child, input), &first_prefix, following_prefix)
            }
            _ => render_prefixed_block(
                &render_node(child, input),
                following_prefix,
                following_prefix,
            ),
        };

        output.push_str(&rendered);
    }

    if output.is_empty() {
        return marker.to_string();
    }

    output
}

fn ordered_list_marker(node: &Node, fallback: u32, input: &str) -> String {
    let fallback = format!("{fallback}.");
    let item = match node {
        Node::ListItem(item) => item,
        _ => return fallback,
    };
    let position = match &item.position {
        Some(position) => position,
        None => return fallback,
    };
    let line = match input.lines().nth(position.start.line.saturating_sub(1)) {
        Some(line) => line,
        None => return fallback,
    };
    let trimmed = line.trim_start();
    let digits = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    match trimmed.chars().nth(digits.chars().count()) {
        Some(delimiter @ ('.' | ')')) if !digits.is_empty() => format!("{digits}{delimiter}"),
        _ => fallback,
    }
}

fn thematic_break_should_expand(children: &[Node], index: usize) -> bool {
    let previous = index.checked_sub(1).and_then(|idx| children.get(idx));
    matches!(previous, Some(Node::List(_)))
}

fn should_insert_blank_line_between_blocks(previous: &Node, current: &Node) -> bool {
    is_code_like_block(previous) || is_code_like_block(current)
}

fn should_insert_blank_line_between_list_item_blocks(
    previous: &Node,
    current: &Node,
    spread: bool,
) -> bool {
    should_insert_blank_line_between_blocks(previous, current)
        || (spread
            && match (previous, current) {
                (Node::Paragraph(_), Node::Paragraph(_)) => true,
                (Node::Paragraph(_), Node::List(list)) => !list.ordered,
                (Node::List(list), Node::Paragraph(_)) => !list.ordered,
                _ => false,
            })
}

fn is_code_like_block(node: &Node) -> bool {
    matches!(node, Node::Code(_) | Node::Math(_))
}

fn should_compact_root_blocks(previous: &Node, current: &Node) -> bool {
    matches!(previous, Node::Definition(_)) && matches!(current, Node::Definition(_))
}
