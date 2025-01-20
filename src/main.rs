use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::Sourcepos;
use cot::bytes::Bytes;
use cot::config::ProjectConfig;
use cot::middleware::LiveReloadMiddleware;
use cot::request::{Request, RequestExt};
use cot::response::{Response, ResponseExt};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::{reverse, reverse_redirect, static_files, Body, CotApp, CotProject, Error, StatusCode};
use rinja::Template;
use serde::Deserialize;

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    request: &'a Request,
}

async fn index(request: Request) -> cot::Result<Response> {
    let index_template = IndexTemplate { request: &request };
    let rendered = index_template.render()?;

    Ok(Response::new_html(StatusCode::OK, Body::fixed(rendered)))
}

#[derive(Debug, Template)]
#[template(path = "guide.html")]
struct GuideTemplate<'a> {
    link_categories: &'a [GuideLinkCategory],
    guide: &'a Guide,
    request: &'a Request,
}

#[derive(Debug)]
struct GuideHeadingAdapter {
    anchorizer: Mutex<comrak::Anchorizer>,
    sections: Mutex<Vec<Section>>,
}

impl HeadingAdapter for GuideHeadingAdapter {
    fn enter(
        &self,
        output: &mut dyn Write,
        heading: &HeadingMeta,
        _sourcepos: Option<Sourcepos>,
    ) -> std::io::Result<()> {
        if heading.level == 1 {
            return write!(output, "<h{}>", heading.level);
        }

        let anchor = {
            let mut anchorizer = self.anchorizer.lock().unwrap();
            anchorizer.anchorize(heading.content.clone())
        };

        {
            let section = Section {
                level: heading.level,
                title: heading.content.clone(),
                anchor: anchor.clone(),
                children: vec![],
            };
            let mut sections = self.sections.lock().unwrap();
            sections.push(section);
        }

        write!(
            output,
            "<h{} id=\"{}\"><a class=\"anchor-link\" href=\"#{}\" aria-label=\"Link to this section: {}\"></a>",
            heading.level, anchor, anchor, heading.content
        )
    }

    fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> std::io::Result<()> {
        write!(output, "</h{}>", heading.level)
    }
}

#[derive(Debug, Clone)]
struct Guide {
    link: &'static str,
    title: String,
    content_html: String,
    sections: Vec<Section>,
}

#[derive(Debug, Clone)]
struct GuideLinkCategory {
    title: &'static str,
    guides: Vec<GuideLink>,
}

#[derive(Debug, Clone)]
struct GuideLink {
    link: &'static str,
    title: String,
}

impl From<&Guide> for GuideLink {
    fn from(value: &Guide) -> Self {
        Self {
            link: value.link,
            title: value.title.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FrontMatter {
    title: String,
}

#[derive(Debug, Clone, Template)]
#[template(path = "_guide_toc_item.html")]
struct Section {
    level: u8,
    title: String,
    anchor: String,
    children: Vec<Self>,
}

macro_rules! md_guide {
    ($name:literal) => {
        parse_guide($name, include_str!(concat!("guide/", $name, ".md")))
    };
}

fn parse_guides() -> (Vec<GuideLinkCategory>, HashMap<&'static str, Guide>) {
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

const DEFAULT_GUIDE_PAGE: &'static str = "introduction";

async fn guide(request: Request) -> cot::Result<Response> {
    page_response(&request, DEFAULT_GUIDE_PAGE)
}

async fn guide_page(request: Request) -> cot::Result<Response> {
    let page = request.path_params().get("page").unwrap();
    if page == DEFAULT_GUIDE_PAGE {
        return Ok(reverse_redirect!(request, "guide"));
    }

    page_response(&request, page)
}

fn page_response(request: &Request, page: &str) -> cot::Result<Response> {
    let (link_categories, guide_map) = parse_guides();
    let guide = guide_map.get(page).unwrap();

    let guide_template = GuideTemplate {
        link_categories: &link_categories,
        guide,
        request: &request,
    };

    let rendered = guide_template.render()?;
    Ok(Response::new_html(StatusCode::OK, Body::fixed(rendered)))
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

    let heading_adapter = GuideHeadingAdapter {
        anchorizer: Mutex::new(comrak::Anchorizer::new()),
        sections: Mutex::new(vec![]),
    };

    let syntax_highlighter = comrak::plugins::syntect::SyntectAdapter::new(None);
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

    // todo(cot) (typed?) path params
    // todo(cot) slashes in URLs
    // todo guide
    // todo faq
    // todo licenses page
    // todo opengraph/twitter meta
    // todo(cot) config from env
    // todo docker image
    // todo next/previous guide links
    // todo proc macros
}

struct CotSiteApp;

impl CotApp for CotSiteApp {
    fn name(&self) -> &'static str {
        "cot-site"
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            Route::with_handler_and_name("/guide/latest/", guide, "guide"),
            Route::with_handler_and_name("/guide/latest/:page", guide_page, "guide_page"),
        ])
    }

    fn static_files(&self) -> Vec<(String, Bytes)> {
        static_files!(
            "css/main.css",
            "js/color-modes.js",
            "images/cot-dark.svg",
            "images/favicon.svg",
            "images/favicon-32.png",
            "images/favicon-180.png",
            "images/favicon-192.png",
            "images/favicon-512.png",
            "images/site.webmanifest"
        )
    }
}

#[cot::main]
async fn main() -> cot::Result<CotProject> {
    let todo_project = CotProject::builder()
        .config(ProjectConfig::builder().build())
        .with_cli(cot::cli::metadata!())
        .register_app_with_views(CotSiteApp, "")
        .middleware_with_context(StaticFilesMiddleware::from_app_context)
        .middleware(LiveReloadMiddleware::new())
        .build()
        .await?;

    Ok(todo_project)
}
