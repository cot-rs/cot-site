use std::fmt::Write;

use comrak::html::{
    ChildRendering, Context, format_document_with_formatter, format_node_default, render_sourcepos,
};
use comrak::nodes::{AstNode, NodeCode, NodeValue};
use comrak::options::Plugins;
use comrak::{Arena, Options, parse_document};
use cot_site_common::Version;
use regex::Regex;

#[derive(Debug, Clone)]
struct UserData {
    version: Version,
}

pub(super) fn markdown_to_html(
    md: &str,
    options: &Options,
    plugins: &Plugins,
    version: Version,
) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut s = String::new();
    let user_data = UserData { version };
    format_document_with_formatter(
        root,
        options,
        &mut s,
        plugins,
        format_node_custom,
        user_data,
    )
    .unwrap();

    s
}

fn format_node_custom<'a>(
    context: &mut Context<UserData>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    match node.data.borrow().value {
        NodeValue::Table(_) => render_table_custom(context, node, entering),
        NodeValue::Code(NodeCode {
            num_backticks: 1, ..
        }) => render_code_custom(context, node, entering),
        _ => format_node_default(context, node, entering),
    }
}

fn render_table_custom<'a>(
    context: &mut Context<UserData>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    if entering {
        context.cr()?;
        // add the Bootstrap "table" class
        context.write_str("<table class=\"table\"")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        if !node
            .last_child()
            .expect("table node has no children")
            .same_node(node.first_child().expect("table node has no children"))
        {
            context.cr()?;
            context.write_str("</tbody>")?;
        }
        context.cr()?;
        context.write_str("</table>")?;
    }

    Ok(ChildRendering::HTML)
}

// regex that captures: optional [display], required <cot::...>, optional |type
// examples matched:
//   [Display]<cot::a::b::Name>|struct
//   <cot::a::Name>|fn
//   [Display]<cot::a::b::Name>
const REGEX: &str =
    r"^\s*(?:\[(?P<display>[^\]]+)\])?\s*<(?P<route>cot::[^>]+)>(?:\|(?P<ty>[A-Za-z0-9_]+))?\s*$";

fn render_code_custom<'a>(
    context: &mut Context<UserData>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    if entering {
        let node_data = node.data.borrow();
        let code = match &node_data.value {
            NodeValue::Code(code) => code,
            _ => return format_node_default(context, node, entering),
        };

        let code_str = &code.literal;
        let re = Regex::new(REGEX).unwrap();

        if let Some(caps) = re.captures(code_str) {
            let route = caps.name("route").unwrap().as_str();
            let display = {
                let d = caps.name("display").map_or(route, |m| m.as_str());
                context.escape(d)?;
                d
            };
            let ty = caps.name("ty").map(|m| m.as_str());
            let last_part = route.split("::").last().unwrap();
            let last_part = if let Some(ty) = ty {
                format!("{ty}.{last_part}")
            } else {
                last_part.to_string()
            };

            let version = &context.user;
            let link = format!(
                "https://docs.rs/cot/{}/cot/{}/{}.html",
                version.version.to_string(),
                route
                    .split("::")
                    .skip(1)
                    .take(route.split("::").count() - 2)
                    .collect::<Vec<&str>>()
                    .join("/"),
                last_part
            );

            context.write_str(&format!(
                "<a href=\"{link}\" target=\"_blank\" rel=\"noopener noreferrer\">{display}</a>"
            ))?;
        } else {
            return format_node_default(context, node, entering);
        }
    }

    Ok(ChildRendering::HTML)
}
