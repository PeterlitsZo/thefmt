use markdown::mdast::Node;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{render_inlines, render_node, MAX_LINE_WIDTH};

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

pub(super) fn render_wrapped_paragraph(
    children: &[Node],
    first_prefix: &str,
    continuation_prefix: &str,
) -> String {
    if children.iter().any(|child| matches!(child, Node::Break(_))) {
        return render_prefixed_block(&render_inlines(children), first_prefix, continuation_prefix);
    }

    let fragments = render_inline_fragments(children);
    wrap_fragments(&fragments, first_prefix, continuation_prefix)
}

pub(super) fn render_prefixed_block(
    block: &str,
    first_prefix: &str,
    continuation_prefix: &str,
) -> String {
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

fn wrap_fragments(
    fragments: &[InlineFragment],
    first_prefix: &str,
    continuation_prefix: &str,
) -> String {
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
