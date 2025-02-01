use cot_site_common::guides::{FrontMatter, Guide, GuideHeadingAdapter, Section};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::Path;
use std::sync::Mutex;
use syn::parse::{Parse, ParseStream};
use syn::LitStr;

pub(super) struct MdGuide {
    pub(super) link: String,
}

impl Parse for MdGuide {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let link = input.parse::<LitStr>()?.value();
        Ok(Self { link })
    }
}

fn read_guide(link: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = Path::new(&manifest_dir)
        .join("src")
        .join("guide")
        .join(link)
        .with_extension("md");
    std::fs::read_to_string(path).expect("failed to read file")
}

pub(super) fn quote_guide(guide: &Guide) -> TokenStream {
    let link = &guide.link;
    let title = &guide.title;
    let content_html = &guide.content_html;
    let sections = guide.sections.iter().map(quote_section);

    let guide = quote! {
        cot_site_common::guides::Guide {
            link: String::from(#link),
            title: String::from(#title),
            content_html: String::from(#content_html),
            sections: vec![#(#sections),*],
        }
    };
    guide.into()
}

fn quote_section(section: &Section) -> TokenStream {
    let level = section.level;
    let title = &section.title;
    let anchor = &section.anchor;
    let children = section.children.iter().map(quote_section);

    let section = quote! {
        cot_site_common::guides::Section {
            level: #level,
            title: String::from(#title),
            anchor: String::from(#anchor),
            children: vec![#(#children),*],
        }
    };
    section.into()
}

pub(super) fn parse_guide(link: &str) -> Guide {
    let guide_content = read_guide(link);

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
                "../../syntax-highlighting/defs.bin"
            ))
            .expect("failed to load syntax set"),
        )
        .build();
    let render_plugins = comrak::RenderPlugins::builder()
        .codefence_syntax_highlighter(&syntax_highlighter)
        .heading_adapter(&heading_adapter)
        .build();
    let plugins = comrak::Plugins::builder().render(render_plugins).build();

    let guide_content = comrak::markdown_to_html_with_plugins(&guide_content, &options, &plugins);
    let mut sections = heading_adapter.sections.lock().unwrap().clone();
    let root_section = fix_section_children(&mut sections);

    let guide = Guide {
        link: link.to_string(),
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
