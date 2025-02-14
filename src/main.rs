mod guides;

use cot::bytes::Bytes;
use cot::cli::CliMetadata;
use cot::config::{LiveReloadMiddlewareConfig, MiddlewareConfig, ProjectConfig};
use cot::middleware::LiveReloadMiddleware;
use cot::project::{App, Project, RootHandlerBuilder, WithApps, WithConfig};
use cot::request::{Request, RequestExt};
use cot::response::{Response, ResponseExt};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::{
    reverse_redirect, static_files, AppBuilder, Body, BoxedHandler, ProjectContext, StatusCode,
};
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

    // todo(cot) use app name in table name
    // todo guide
    // todo faq
    // todo licenses page
    // todo simple blog
}

struct CotSiteApp;

impl App for CotSiteApp {
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

struct CotSiteProject;

impl Project for CotSiteProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn config(&self, _config_name: &str) -> cot::Result<ProjectConfig> {
        // we don't need to load any config
        Ok(ProjectConfig::builder()
            .middlewares(
                MiddlewareConfig::builder()
                    .live_reload(
                        LiveReloadMiddlewareConfig::builder()
                            .enabled(cfg!(debug_assertions))
                            .build(),
                    )
                    .build(),
            )
            .build())
    }

    fn register_apps(&self, modules: &mut AppBuilder, _app_context: &ProjectContext<WithConfig>) {
        modules.register_with_views(CotSiteApp, "");
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &ProjectContext<WithApps>,
    ) -> BoxedHandler {
        handler
            .middleware(StaticFilesMiddleware::from_app_context(context))
            .middleware(LiveReloadMiddleware::new())
            .build()
    }
}

#[cot::main]
fn main() -> impl Project {
    CotSiteProject
}
