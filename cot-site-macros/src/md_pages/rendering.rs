use std::fmt;
use std::fmt::Write;

use comrak::html::{
    ChildRendering, Context, format_document_with_formatter, format_node_default, render_sourcepos,
};
use comrak::nodes::{AstNode, NodeLink, NodeValue};
use comrak::options::Plugins;
use comrak::{Arena, Options, parse_document};
use cot_site_common::Version;

const COT_RUSTDOC_BASE_URL: &str = "https://docs.rs/cot";
const COT_RUSTDOC_CRATE_OVERVIEW_URL: &str = "https://docs.rs/crate/cot";

#[derive(Debug, Clone)]
struct PageContext {
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
    let page_context = PageContext { version };
    format_document_with_formatter(
        root,
        options,
        &mut s,
        plugins,
        format_node_custom,
        page_context,
    )
    .unwrap();

    s
}

fn format_node_custom<'a>(
    context: &mut Context<PageContext>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, std::fmt::Error> {
    match node.data.borrow().value {
        NodeValue::Table(_) => render_table_custom(context, node, entering),
        NodeValue::Link(ref ln) => render_link_custom(context, node, entering, ln),
        NodeValue::CodeBlock(_) => render_code_block(context, node, entering),
        _ => format_node_default(context, node, entering),
    }
}

fn render_code_block<'a, T>(
    context: &mut Context<T>,
    node: &'a AstNode<'a>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<div class=\"code-block\">")?;
        context.write_str("<button type=\"button\" class=\"code-block-copy-btn\" data-copy-code aria-label=\"copy\" title=\"copy\">Copy</button>")?;
    }

    format_node_default(context, node, entering)?;

    if !entering {
        context.write_str("</div>")?;
    }

    Ok(ChildRendering::HTML)
}

fn render_table_custom<'a>(
    context: &mut Context<PageContext>,
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

/// Resolve routes of the format used in rustdoc links:
/// https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html#namespaces-and-disambiguators
/// into valid rustdoc URLs.
///
/// The format is:
/// `type@cot::a::b::Name`
///
/// where type is optional and can be one of the options mentioned here: https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html#path-ambiguities
/// If the type is not provided, we assume it's a module.
///
/// We try to follow the same structure as rustdoc links as much as we can.
/// However, there some special cases to be aware of:
///
/// ## Methods
/// Methods in rustdoc follow the format `method@cot::a::b`. However, in our
/// case, we want to be able to tell what type the method belongs to and whether
/// it is a required method or not (rustdoc links provide different URLs for
/// required and provided methods). To achieve this, we require the type the
/// method belongs to be specified as the type in the route. We also require the
/// method name to be specified as an internal navigation link, e.g. `struct@
/// cot::a::b::Name#method` for a provided method and
/// `trait@cot::a::b::Name#tymethod` for a required method. Note the `tymethod`
/// suffix for required methods and the `method` suffix for provided methods.
///
/// Examples
/// ```
/// # reference to `foo` method of `Name` struct in `cot::a::b` module
/// struct@cot::a::b::Name#method.foo -> https://docs.rs/cot/0.5.0/cot/a/b/struct.Name.html#method.foo
/// # reference to `foo` required method of `Name` trait in `cot::a::b` module
/// trait@cot::a::b::Name#tymethod -> https://docs.rs/cot/0.5.0/cot/a/b/trait.Name.html#tymethod
/// ```
///
/// ## Features
/// Reference to features should follow the format `feature@feature_name`, where
/// `feature_name` is the name of the feature.
///
/// # Other Internal Navigation Links
/// Other types such as struct fields follow the same format as methods.
///
/// Examples:
///
/// ```
/// # reference to a `url` field in the `DatabaseConfig` struct in the `cot::config` module.
/// struct@cot::config::DatabaseConfig#structfield.url -> https://docs.rs/cot/0.5.0/cot/config/struct.DatabaseConfig.html/structfield.url
/// ```
fn resolve_url(route: &str, page_context: &PageContext) -> String {
    let version = format!(
        "{}.{}",
        page_context.version.major(),
        page_context.version.minor()
    );
    let mut parts: Vec<String> = vec![COT_RUSTDOC_BASE_URL.to_string(), version];

    let (ty, route_str) = route
        .split_once('@')
        .map(|(ty, route_str)| {
            let ty = if ty.trim().is_empty() {
                None
            } else {
                Some(ty.trim())
            };
            (ty, route_str)
        })
        .unwrap_or((None, route));

    if !route_str.starts_with("cot::") && ty != Some("feature") {
        return route.to_string();
    }

    let segs: Vec<&str> = route_str.split("::").collect();
    if segs.len() > 1 {
        parts.extend(
            // we are only interested in everything but the last segement.
            segs.iter().take(segs.len() - 1).map(|s| s.to_string()),
        );
    }

    // the last segment can contain an internal navigation link, e.g.
    // `struct@cot::a::b::Name#method`.
    let (last_part, internal_nav_link) = segs
        .last()
        .expect("route split produced no segments")
        .split_once('#')
        .map(|(last_part, internal_nav_link)| (last_part, Some(internal_nav_link)))
        .unwrap_or((segs.last().unwrap(), None));

    if let Some(ty) = ty {
        match ty {
            "feature" => {
                // features use the crate overview page instead of the regular doc.rs page.
                parts[0] = COT_RUSTDOC_CRATE_OVERVIEW_URL.to_string();
                // rustdoc uses `features`. we use `feature` for consistency with other types.
                parts.push(format!("features#{}", last_part))
            }
            other => {
                let mut part_str = format!("{}.{}.html", other, last_part);
                // add the internal navigation link if it exists.
                if let Some(internal_nav_link) = internal_nav_link {
                    part_str.push_str(&format!("#{}", internal_nav_link));
                }
                parts.push(part_str);
            }
        }
    } else {
        parts.push(last_part.to_string())
    }

    parts.join("/")
}

fn render_link_custom<'a>(
    context: &mut Context<PageContext>,
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
            let user_data = PageContext {
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
            "https://docs.rs/cot/1.2/cot/a/b/struct.Name.html"
        );
        test_resolve!("cot::a::b", "https://docs.rs/cot/1.2/cot/a/b");
        test_resolve!("@cot::a::b::Name", "https://docs.rs/cot/1.2/cot/a/b/Name");
        test_resolve!("attr@invalid::a::b::name", "attr@invalid::a::b::name");
        test_resolve!("http://example.com", "http://example.com");
        test_resolve!("cot::name", "https://docs.rs/cot/1.2/cot/name");
        test_resolve!(
            "struct@cot::Name",
            "https://docs.rs/cot/1.2/cot/struct.Name.html"
        );
        test_resolve!("cot", "cot");
    }
}
