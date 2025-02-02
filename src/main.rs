mod guides;

use cot::bytes::Bytes;
use cot::config::ProjectConfig;
use cot::middleware::LiveReloadMiddleware;
use cot::request::{Request, RequestExt};
use cot::response::{Response, ResponseExt};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::{reverse_redirect, static_files, Body, CotApp, CotProject, StatusCode};
use cot_site_common::guides::{Guide, GuideLink, Section};
use rinja::filters::{HtmlSafe, Safe};
use rinja::Template;

use crate::guides::{get_prev_next_link, parse_guides};

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
    prev: Option<&'a GuideLink>,
    next: Option<&'a GuideLink>,
}

#[derive(Debug, Clone)]
struct GuideLinkCategory {
    title: &'static str,
    guides: Vec<GuideLink>,
}

fn render_section(section: &Section) -> Safe<String> {
    #[derive(Debug, Clone, Template)]
    #[template(path = "_guide_toc_item.html")]
    struct RenderableSection<'a> {
        section: &'a Section,
    }

    impl HtmlSafe for RenderableSection<'_> {}

    let rendered = RenderableSection { section }.render().unwrap();
    Safe(rendered)
}

const DEFAULT_GUIDE_PAGE: &'static str = "introduction";

async fn guide(request: Request) -> cot::Result<Response> {
    page_response(&request, DEFAULT_GUIDE_PAGE)
}

async fn guide_page(request: Request) -> cot::Result<Response> {
    let page = request.path_params().parse()?;

    if page == DEFAULT_GUIDE_PAGE {
        return Ok(reverse_redirect!(request, "guide")?);
    }

    page_response(&request, page)
}

fn page_response(request: &Request, page: &str) -> cot::Result<Response> {
    let (link_categories, guide_map) = parse_guides();
    let guide = guide_map.get(page).unwrap();
    let (prev, next) = get_prev_next_link(&link_categories, page);

    let guide_template = GuideTemplate {
        link_categories: &link_categories,
        guide,
        request: &request,
        prev,
        next,
    };

    let rendered = guide_template.render()?;
    Ok(Response::new_html(StatusCode::OK, Body::fixed(rendered)))

    // todo(cot) move code out of lib.rs
    // todo(cot) slashes in URLs
    // todo(cot) query!() working with path::function()
    // todo(cot) cot::test
    // todo(cot) admin panel
    // todo(cot) disable live reloading in production
    // todo(cot) 404 support
    // todo(cot) cli: add migration to file
    // todo(cot) config from env
    // todo(cot) proper html escaping
    // todo(cot) generating openapi spec
    // todo(cot) ergonomic input/output in views
    // todo guide
    // todo faq
    // todo licenses page
    // todo opengraph/twitter meta
    // todo 404 page
    // todo webhook to deploy
    // todo README.md
    // todo throw 404
    // todo simple blog
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
            Route::with_handler_and_name("/guide/latest/{page}", guide_page, "guide_page"),
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
    let builder = CotProject::builder()
        .config(ProjectConfig::builder().build())
        .with_cli(cot::cli::metadata!())
        .register_app_with_views(CotSiteApp, "")
        .middleware_with_context(StaticFilesMiddleware::from_app_context);
    #[cfg(debug_assertions)]
    let builder = builder.middleware(LiveReloadMiddleware::new());

    Ok(builder.build().await?)
}
