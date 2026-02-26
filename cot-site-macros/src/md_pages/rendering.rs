use std::fmt::Write;

use comrak::html::{
    ChildRendering, Context, format_document_with_formatter, format_node_default, render_sourcepos,
};
use comrak::nodes::{AstNode, NodeLink, NodeValue};
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
        NodeValue::Link(ref ln) => render_link_custom(context, node, entering, ln),
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

// This regex matches the format used in
// rustdoc links: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html#namespaces-and-disambiguators
// The format is:
// type@cot::a::b::Name
// where type is optional and can be one of the options mentioned here: https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html#path-ambiguities.
// If type is not provided, we assume it's a module.
const REGEX: &str = r"^\s*(?:\[(?P<display>[^\]]+)\])?\s*(?:(?P<ty>[A-Za-z0-9_]+)@|@)?(?P<route>cot::[A-Za-z0-9_:]+)\s*$";

fn resolve_url(route: &str, user_data: &UserData) -> String {
    let re = Regex::new(REGEX).unwrap();

    if let Some(caps) = re.captures(route) {
        let version = user_data.version.to_string();
        let mut parts: Vec<String> = vec![
            "https://docs.rs/cot".to_string(),
            version,
            "cot".to_string(),
        ];

        let ty = caps.name("ty").map(|m| m.as_str());

        let route_str = caps
            .name("route")
            .expect(&format!("could not resolve route: {route}"))
            .as_str();

        let segs: Vec<&str> = route_str.split("::").collect();

        // we're interested in everything between "cot" and the last segment
        if segs.len() > 2 {
            parts.extend(segs[1..segs.len() - 1].iter().map(|s| s.to_string()));
        }

        let last_part = segs.last().expect("route split produced no segments");
        if let Some(ty) = ty {
            parts.push(format!("{}.{}.html", ty, last_part))
        } else {
            parts.push(last_part.to_string())
        }

        parts.join("/")
    } else {
        route.to_string()
    }
}

fn render_link_custom<'a>(
    context: &mut Context<UserData>,
    _node: &'a AstNode<'a>,
    entering: bool,
    nl: &NodeLink,
) -> Result<ChildRendering, std::fmt::Error> {
    let url = resolve_url(&nl.url, &context.user);
    let node = AstNode::from(NodeValue::Link(Box::new(NodeLink {
        url,
        title: nl.title.clone(),
    })));

    format_node_default(context, &node, entering)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_resolve {
        ($route:expr, $expected:expr) => {
            let user_data = UserData {
                version: Version::new(1, 2, 3),
            };
            let url = resolve_url($route, &user_data);
            assert_eq!(url, $expected);
        };
    }

    #[test]
    fn test_resolve_url() {
        test_resolve!(
            "struct@cot::a::b::Name",
            "https://docs.rs/cot/1.2.3/cot/a/b/struct.Name.html"
        );
        test_resolve!("cot::a::b", "https://docs.rs/cot/1.2.3/cot/a/b");
        test_resolve!("@cot::a::b::Name", "https://docs.rs/cot/1.2.3/cot/a/b/Name");
        test_resolve!("attr@invalid::a::b::name", "attr@invalid::a::b::name");
        test_resolve!("http://example.com", "http://example.com");
        test_resolve!("cot::name", "https://docs.rs/cot/1.2.3/cot/name");
        test_resolve!(
            "struct@cot::Name",
            "https://docs.rs/cot/1.2.3/cot/struct.Name.html"
        );
        test_resolve!("cot", "cot");
        test_resolve!("cot::", "cot::");
    }
}
