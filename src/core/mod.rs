use crate::error::Error;
use markdown::mdast::{Node, ReferenceKind};

mod blocks;
mod tables;
mod wrap;

pub(super) const MAX_LINE_WIDTH: usize = 80;

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
        Node::Root(root) => blocks::render_root_blocks(&root.children),
        Node::Blockquote(blockquote) => blocks::render_blockquote(&blockquote.children),
        Node::FootnoteDefinition(footnote) => {
            format!(
                "[^{}]: {}",
                footnote.identifier,
                blocks::render_blocks(&footnote.children)
            )
        }
        Node::MdxJsxFlowElement(element) => blocks::render_blocks(&element.children),
        Node::List(list) => {
            blocks::render_list(&list.children, list.ordered, list.start, list.spread, "")
        }
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
        Node::Table(table) => tables::render_table(&table.children, &table.align),
        Node::ThematicBreak(_) => "---".to_string(),
        Node::TableRow(row) => tables::render_unpadded_table_row(&row.children),
        Node::TableCell(cell) => render_inlines(&cell.children),
        Node::ListItem(item) => blocks::render_blocks(&item.children),
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

fn render_inlines(children: &[Node]) -> String {
    children.iter().map(render_node).collect()
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

fn trim_trailing_newlines(value: &str) -> &str {
    value.trim_end_matches(['\n', '\r'])
}
