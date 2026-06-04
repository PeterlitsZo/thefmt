use markdown::mdast::Node;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{render_inlines, render_node, MAX_LINE_WIDTH};

const PREFERRED_SOFT_BREAK_SLACK: usize = 2;

#[derive(Clone)]
enum InlineFragment {
    BreakableText(Vec<BreakableUnit>),
    Atomic(String),
}

#[derive(Clone)]
enum LinePart {
    Breakable(Vec<BreakableUnit>),
    Atomic(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BreakableUnit {
    Visible(char),
    SoftBreak { preferred: bool },
}

#[derive(Clone, Copy)]
struct BreakPoint {
    part_index: usize,
    unit_index: usize,
    width: usize,
}

#[derive(Default)]
struct LineState {
    parts: Vec<LinePart>,
    width: usize,
    last_preferred_break: Option<BreakPoint>,
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
        .enumerate()
        .filter_map(|(index, child)| match child {
            Node::Text(text) => {
                let normalized = normalize_breakable_text(
                    &text.value,
                    previous_visible_char(children, index),
                    next_visible_char(children, index),
                );
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

fn normalize_breakable_text(
    text: &str,
    previous_context: Option<char>,
    next_context: Option<char>,
) -> Vec<BreakableUnit> {
    let mut normalized = Vec::new();
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            let whitespace_start = index;
            let previous = normalized
                .iter()
                .rev()
                .find_map(|unit| match unit {
                    BreakableUnit::Visible(ch) => Some(*ch),
                    BreakableUnit::SoftBreak { .. } => None,
                })
                .or(previous_context);
            let next = chars[index + 1..]
                .iter()
                .copied()
                .find(|candidate| !candidate.is_whitespace())
                .or(next_context);

            if previous.is_some() && next.is_some() {
                if should_preserve_soft_break_space(previous, next) {
                    if !matches!(normalized.last(), Some(BreakableUnit::Visible(' '))) {
                        normalized.push(BreakableUnit::Visible(' '));
                    }
                } else if !matches!(normalized.last(), Some(BreakableUnit::SoftBreak { .. })) {
                    let contains_newline = chars[whitespace_start..]
                        .iter()
                        .take_while(|candidate| candidate.is_whitespace())
                        .any(|candidate| matches!(candidate, '\n' | '\r'));
                    let preferred = contains_newline
                        && matches!(
                            (previous, next),
                            (Some(previous), Some(next))
                                if is_likely_cjk_clause_boundary(previous, next)
                        );
                    normalized.push(BreakableUnit::SoftBreak { preferred });
                }
            }

            while index + 1 < chars.len() && chars[index + 1].is_whitespace() {
                index += 1;
            }
        } else {
            normalized.push(BreakableUnit::Visible(ch));
        }

        index += 1;
    }

    normalized
}

fn previous_visible_char(children: &[Node], index: usize) -> Option<char> {
    children[..index].iter().rev().find_map(last_visible_char)
}

fn next_visible_char(children: &[Node], index: usize) -> Option<char> {
    children[index + 1..].iter().find_map(first_visible_char)
}

fn first_visible_char(node: &Node) -> Option<char> {
    render_node(node).chars().find(|ch| !ch.is_whitespace())
}

fn last_visible_char(node: &Node) -> Option<char> {
    render_node(node)
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
}

fn should_preserve_soft_break_space(previous: Option<char>, next: Option<char>) -> bool {
    match (previous, next) {
        (Some(previous), Some(next)) => !(is_cjk_word_char(previous) && is_cjk_word_char(next)),
        _ => false,
    }
}

fn is_cjk_word_char(ch: char) -> bool {
    UnicodeWidthChar::width(ch).unwrap_or(0) > 1 && ch.is_alphanumeric()
}

// Prefer source line breaks that already sit on common Chinese phrase boundaries.
fn is_likely_cjk_clause_boundary(previous: char, next: char) -> bool {
    matches!(previous, '的' | '地' | '得') || matches!(next, '来' | '去' | '并' | '再' | '将')
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
                for unit in text {
                    append_breakable_unit(
                        &mut lines,
                        &mut line,
                        &mut current_prefix,
                        continuation_prefix,
                        *unit,
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

fn append_breakable_unit<'a>(
    lines: &mut Vec<String>,
    line: &mut LineState,
    current_prefix: &mut &'a str,
    continuation_prefix: &'a str,
    unit: BreakableUnit,
) {
    if line.width == 0
        && matches!(
            unit,
            BreakableUnit::Visible(' ') | BreakableUnit::SoftBreak { .. }
        )
    {
        return;
    }

    let char_width = breakable_unit_width(unit);
    let prefix_width = UnicodeWidthStr::width(*current_prefix);

    if prefix_width + line.width + char_width <= MAX_LINE_WIDTH || line.width == 0 {
        push_breakable_unit(line, unit, char_width);
        return;
    }

    if let Some(breakpoint) = choose_breakpoint(line) {
        let remainder = split_line_at_breakpoint(line, breakpoint);
        flush_line(lines, line, current_prefix);
        *current_prefix = continuation_prefix;
        *line = remainder;
        append_breakable_unit(lines, line, current_prefix, continuation_prefix, unit);
        return;
    }

    flush_line(lines, line, current_prefix);
    *current_prefix = continuation_prefix;
    append_breakable_unit(lines, line, current_prefix, continuation_prefix, unit);
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

    line.parts.push(LinePart::Atomic(text.to_string()));
    line.width += text_width;
}

fn push_breakable_unit(line: &mut LineState, unit: BreakableUnit, char_width: usize) {
    if let Some(last) = line.parts.last_mut() {
        if let LinePart::Breakable(units) = last {
            units.push(unit);
        } else {
            line.parts.push(LinePart::Breakable(vec![unit]));
        }
    } else {
        line.parts.push(LinePart::Breakable(vec![unit]));
    }

    line.width += char_width;
    let part_index = line.parts.len() - 1;
    let unit_index = breakable_part_len(&line.parts[part_index]);
    if let BreakableUnit::SoftBreak { preferred: true } = unit {
        line.last_preferred_break = Some(BreakPoint {
            part_index,
            unit_index,
            width: line.width,
        });
    }
    if can_break_after_unit(unit) {
        line.last_char_break = Some(BreakPoint {
            part_index,
            unit_index,
            width: line.width,
        });
    }

    if unit == BreakableUnit::Visible(' ') {
        line.last_space_break = Some(BreakPoint {
            part_index,
            unit_index,
            width: line.width,
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

        match part {
            LinePart::Breakable(units) if index == breakpoint.part_index => {
                let head = units[..breakpoint.unit_index].to_vec();
                if !head.is_empty() {
                    left_parts.push(LinePart::Breakable(head));
                }
                let tail = units[breakpoint.unit_index..].to_vec();
                if !tail.is_empty() {
                    remainder_parts.push(LinePart::Breakable(tail));
                }
            }
            _ => remainder_parts.push(part.clone()),
        }
    }

    *line = build_line_state(left_parts);
    build_line_state(remainder_parts)
}

fn build_line_state(parts: Vec<LinePart>) -> LineState {
    let mut line = LineState::default();

    for part in parts {
        if line_part_is_empty(&part) {
            continue;
        }

        let part_index = line.parts.len();
        match &part {
            LinePart::Breakable(units) => {
                for (unit_index, unit) in units.iter().copied().enumerate() {
                    line.width += breakable_unit_width(unit);
                    if let BreakableUnit::SoftBreak { preferred: true } = unit {
                        line.last_preferred_break = Some(BreakPoint {
                            part_index,
                            unit_index: unit_index + 1,
                            width: line.width,
                        });
                    }
                    if can_break_after_unit(unit) {
                        line.last_char_break = Some(BreakPoint {
                            part_index,
                            unit_index: unit_index + 1,
                            width: line.width,
                        });
                    }
                    if unit == BreakableUnit::Visible(' ') {
                        line.last_space_break = Some(BreakPoint {
                            part_index,
                            unit_index: unit_index + 1,
                            width: line.width,
                        });
                    }
                }
            }
            LinePart::Atomic(text) => {
                line.width += UnicodeWidthStr::width(text.as_str());
            }
        }

        line.parts.push(part);
    }

    line
}

fn line_part_is_empty(part: &LinePart) -> bool {
    match part {
        LinePart::Breakable(units) => units.is_empty(),
        LinePart::Atomic(text) => text.is_empty(),
    }
}

fn breakable_part_len(part: &LinePart) -> usize {
    match part {
        LinePart::Breakable(units) => units.len(),
        LinePart::Atomic(_) => 0,
    }
}

fn render_line_parts(parts: &[LinePart]) -> String {
    let mut content = String::new();
    for part in parts {
        match part {
            LinePart::Breakable(units) => {
                for unit in units {
                    if let BreakableUnit::Visible(ch) = unit {
                        content.push(*ch);
                    }
                }
            }
            LinePart::Atomic(text) => content.push_str(text),
        }
    }
    while content.ends_with(' ') {
        content.pop();
    }
    content
}

fn choose_breakpoint(line: &LineState) -> Option<BreakPoint> {
    if let Some(preferred) = line.last_preferred_break {
        if line.width.saturating_sub(preferred.width) <= PREFERRED_SOFT_BREAK_SLACK {
            return Some(preferred);
        }
    }

    match (line.last_space_break, line.last_char_break) {
        (Some(space), Some(character)) if breakpoint_precedes(space, character) => Some(character),
        (Some(space), _) => Some(space),
        (None, Some(character)) => Some(character),
        (None, None) => None,
    }
}

fn breakpoint_precedes(left: BreakPoint, right: BreakPoint) -> bool {
    (left.part_index, left.unit_index) < (right.part_index, right.unit_index)
}

fn can_break_after_unit(unit: BreakableUnit) -> bool {
    matches!(unit, BreakableUnit::SoftBreak { .. })
        || matches!(unit, BreakableUnit::Visible(ch) if !ch.is_whitespace() && UnicodeWidthChar::width(ch).unwrap_or(0) > 1)
}

fn breakable_unit_width(unit: BreakableUnit) -> usize {
    match unit {
        BreakableUnit::Visible(ch) => UnicodeWidthChar::width(ch).unwrap_or(0),
        BreakableUnit::SoftBreak { .. } => 0,
    }
}
fn flush_line(lines: &mut Vec<String>, line: &mut LineState, prefix: &str) {
    let content = render_line_parts(&line.parts);
    if content.is_empty() {
        return;
    }

    lines.push(format!("{prefix}{content}"));
    *line = LineState::default();
}
