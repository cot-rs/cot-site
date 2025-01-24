use crate::{FrontMatter, Guide, GuideHeadingAdapter, GuideLink, GuideLinkCategory, Section};
use std::collections::HashMap;
use std::sync::Mutex;

macro_rules! md_guide {
    ($name:literal) => {
        parse_guide($name, include_str!(concat!("guide/", $name, ".md")))
    };
}

pub fn parse_guides() -> (Vec<GuideLinkCategory>, HashMap<&'static str, Guide>) {
    let categories = [(
        "Getting started",
        vec![
            md_guide!("introduction"),
            md_guide!("templates"),
            md_guide!("forms"),
            md_guide!("static-files"),
            md_guide!("authentication"),
            md_guide!("db-models"),
            md_guide!("error-pages"),
            md_guide!("testing"),
        ],
    )];

    let categories_links = categories
        .iter()
        .map(|(title, guides)| GuideLinkCategory {
            title,
            guides: guides.iter().map(GuideLink::from).collect(),
        })
        .collect();
    let guide_map = categories
        .into_iter()
        .map(|(_title, guides)| guides)
        .flatten()
        .map(|guide| (guide.link, guide))
        .collect();

    (categories_links, guide_map)
}

pub fn get_prev_next_link<'a>(
    guides: &'a [GuideLinkCategory],
    current_id: &str,
) -> (Option<&'a GuideLink>, Option<&'a GuideLink>) {
    let mut prev = None;
    let mut has_found = false;

    for category in guides {
        for guide in &category.guides {
            if has_found {
                return (prev, Some(guide));
            } else if guide.link == current_id {
                has_found = true;
            } else {
                prev = Some(guide);
            }
        }
    }

    (prev, None)
}

fn parse_guide(link: &'static str, guide_content: &str) -> Guide {
    let front_matter = guide_content
        .split("---")
        .nth(1)
        .expect("front matter not found");
    let front_matter: FrontMatter =
        serde_yml::from_str(front_matter).expect("invalid front matter");

    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.parse.smart = true;
    options.render.unsafe_ = true;

    let heading_adapter = GuideHeadingAdapter {
        anchorizer: Mutex::new(comrak::Anchorizer::new()),
        sections: Mutex::new(vec![]),
    };

    let syntax_highlighter = comrak::plugins::syntect::SyntectAdapterBuilder::new()
        .css()
        .syntax_set(
            syntect::dumps::from_uncompressed_data(include_bytes!(
                "../syntax-highlighting/defs.bin"
            ))
            .expect("failed to load syntax set"),
        )
        .build();
    let render_plugins = comrak::RenderPlugins::builder()
        .codefence_syntax_highlighter(&syntax_highlighter)
        .heading_adapter(&heading_adapter)
        .build();
    let plugins = comrak::Plugins::builder().render(render_plugins).build();

    let guide_content = comrak::markdown_to_html_with_plugins(guide_content, &options, &plugins);
    let mut sections = heading_adapter.sections.lock().unwrap().clone();
    let root_section = fix_section_children(&mut sections);

    let guide = Guide {
        link,
        title: front_matter.title,
        content_html: guide_content,
        sections: root_section.children,
    };
    guide
}

fn fix_section_children(sections: &Vec<Section>) -> Section {
    let root_section = Section {
        level: 0,
        title: String::new(),
        anchor: String::new(),
        children: vec![],
    };
    let mut stack = vec![root_section];

    for section in sections {
        while stack[stack.len() - 1].level >= section.level {
            let last = stack
                .pop()
                .expect("just accessed stack[stack.len() - 1] so stack can't be empty");
            stack
                .last_mut()
                .expect("root section should always be in the stack")
                .children
                .push(last);
        }
        stack.push(section.clone());
    }

    while stack[stack.len() - 1].level > 0 {
        let last = stack
            .pop()
            .expect("just accessed stack[stack.len() - 1] so stack can't be empty");
        stack
            .last_mut()
            .expect("root section should always be in the stack")
            .children
            .push(last);
    }
    stack
        .into_iter()
        .next()
        .expect("root section should always be in the stack")
}
