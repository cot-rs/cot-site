use std::path::Path;
use std::sync::Mutex;

use cot_site_common::md_pages::{FrontMatter, MdPage, MdPageHeadingAdapter, Section};
use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;
use syn::parse::{Parse, ParseStream};

pub(super) struct MdPageInput {
    pub(super) prefix: String,
    pub(super) link: String,
}

impl Parse for MdPageInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let prefix = input.parse::<LitStr>()?.value();
        input.parse::<syn::Token![,]>()?;
        let link = input.parse::<LitStr>()?.value();
        Ok(Self { prefix, link })
    }
}

fn read_md_page(link: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = Path::new(&manifest_dir)
        .join("src")
        .join("md-pages")
        .join(link)
        .with_extension("md");

    #[cfg(feature = "nightly")]
    {
        let path_str = path.to_str().expect("path is not valid UTF-8");
        proc_macro::tracked_path::path(path_str);
    }

    std::fs::read_to_string(path).expect("failed to read file")
}

pub(super) fn quote_md_page(md_page: &MdPage) -> TokenStream {
    let link = &md_page.link;
    let title = &md_page.title;
    let content_html = &md_page.content_html;
    let sections = md_page.sections.iter().map(quote_section);

    let md_page = quote! {
        cot_site_common::md_pages::MdPage {
            link: String::from(#link),
            title: String::from(#title),
            content_html: String::from(#content_html),
            sections: vec![#(#sections),*],
        }
    };
    md_page
}

fn quote_section(section: &Section) -> TokenStream {
    let level = section.level;
    let title = &section.title;
    let anchor = &section.anchor;
    let children = section.children.iter().map(quote_section);

    let section = quote! {
        cot_site_common::md_pages::Section {
            level: #level,
            title: String::from(#title),
            anchor: String::from(#anchor),
            children: vec![#(#children),*],
        }
    };
    section
}

pub(super) fn parse_md_page(prefix: &str, link: &str) -> MdPage {
    let file_link = if prefix.is_empty() {
        link
    } else {
        &format!("{prefix}/{link}")
    };
    let md_page_content = read_md_page(file_link);

    let front_matter = md_page_content
        .split("---")
        .nth(1)
        .expect("front matter not found");
    let front_matter: FrontMatter =
        serde_yml::from_str(front_matter).expect("invalid front matter");

    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.parse.smart = true;
    options.render.r#unsafe = true;

    let heading_adapter = MdPageHeadingAdapter {
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
    let render_plugins = comrak::options::RenderPlugins::builder()
        .codefence_syntax_highlighter(&syntax_highlighter)
        .heading_adapter(&heading_adapter)
        .build();
    let plugins = comrak::options::Plugins::builder()
        .render(render_plugins)
        .build();

    let md_page_content =
        comrak::markdown_to_html_with_plugins(&md_page_content, &options, &plugins);
    let sections = heading_adapter.sections.lock().unwrap().clone();
    let root_section = fix_section_children(&sections);

    MdPage {
        link: link.to_string(),
        title: front_matter.title,
        content_html: md_page_content,
        sections: root_section.children,
    }
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
