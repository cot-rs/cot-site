mod guides;

use cot::bytes::Bytes;
use cot::cli::CliMetadata;
use cot::config::ProjectConfig;
use cot::project::{App, Project, RootHandlerBuilder, WithApps, WithConfig};
use cot::request::{Request, RequestExt};
use cot::response::{Response, ResponseExt};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::{
    reverse_redirect, static_files, AppBuilder, Body, BoxedHandler, ProjectContext, StatusCode,
};
use cot_site_common::md_pages::{MdPage, MdPageLink, Section};
use cot_site_macros::md_page;
use rinja::filters::{HtmlSafe, Safe};
use rinja::Template;

use crate::guides::{get_prev_next_link, parse_guides};

pub(crate) const LATEST_VERSION: &'static str = "v0.2";
pub(crate) const ALL_VERSIONS: &'static [&'static str] = &["latest", "v0.2", "v0.1"];

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
    guide: &'a MdPage,
    versions: &'static [&'static str],
    version: &'a str,
    request: &'a Request,
    prev: Option<&'a MdPageLink>,
    next: Option<&'a MdPageLink>,
}

#[derive(Debug, Clone)]
struct GuideLinkCategory {
    title: &'static str,
    guides: Vec<MdPageLink>,
}

fn render_section(section: &Section) -> Safe<String> {
    #[derive(Debug, Clone, Template)]
    #[template(path = "_md_page_toc_item.html")]
    struct RenderableSection<'a> {
        section: &'a Section,
    }

    impl HtmlSafe for RenderableSection<'_> {}

    let rendered = RenderableSection { section }.render().unwrap();
    Safe(rendered)
}

const DEFAULT_GUIDE_PAGE: &str = "introduction";

async fn guide(request: Request) -> cot::Result<Response> {
    reverse_redirect!(request, "guide_version", version = "latest")
}

async fn guide_version(request: Request) -> cot::Result<Response> {
    let version = request.path_params().parse()?;
    page_response(&request, version, DEFAULT_GUIDE_PAGE)
}

async fn guide_page(request: Request) -> cot::Result<Response> {
    let (version, page) = request.path_params().parse()?;

    if page == DEFAULT_GUIDE_PAGE {
        return Ok(reverse_redirect!(
            request,
            "guide_version",
            version = version
        )?);
    }

    page_response(&request, version, page)
}

fn page_response(request: &Request, version: &str, page: &str) -> cot::Result<Response> {
    let file_version = if version == "latest" {
        LATEST_VERSION
    } else {
        version
    };
    if !ALL_VERSIONS.contains(&file_version) {
        return Err(cot::Error::not_found());
    }
    let (link_categories, guide_map) = parse_guides(file_version);
    let guide = guide_map.get(page).ok_or_else(cot::Error::not_found)?;
    let (prev, next) = get_prev_next_link(&link_categories, page);

    let guide_template = GuideTemplate {
        link_categories: &link_categories,
        guide,
        versions: ALL_VERSIONS,
        version,
        request,
        prev,
        next,
    };

    let rendered = guide_template.render()?;
    Ok(Response::new_html(StatusCode::OK, Body::fixed(rendered)))
}

#[derive(Debug, Template)]
#[template(path = "md_page.html")]
struct MdPageTemplate<'a> {
    page: &'a MdPage,
    request: &'a Request,
}

async fn faq(request: Request) -> cot::Result<Response> {
    let template = MdPageTemplate {
        page: &md_page!("", "faq"),
        request: &request,
    };

    Ok(Response::new_html(
        StatusCode::OK,
        Body::fixed(template.render()?),
    ))
}

async fn licenses(request: Request) -> cot::Result<Response> {
    let template = MdPageTemplate {
        page: &md_page!("", "licenses"),
        request: &request,
    };

    Ok(Response::new_html(
        StatusCode::OK,
        Body::fixed(template.render()?),
    ))
}

struct CotSiteApp;

impl App for CotSiteApp {
    fn name(&self) -> &'static str {
        "cot-site"
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            Route::with_handler_and_name("/faq/", faq, "faq"),
            Route::with_handler_and_name("/licenses/", licenses, "licenses"),
            Route::with_handler_and_name("/guide/", guide, "guide"),
            Route::with_handler_and_name("/guide/{version}/", guide_version, "guide_version"),
            Route::with_handler_and_name("/guide/{version}/{page}/", guide_page, "guide_page"),
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
        Ok(ProjectConfig::default())
    }

    fn register_apps(&self, modules: &mut AppBuilder, _app_context: &ProjectContext<WithConfig>) {
        modules.register_with_views(CotSiteApp, "");
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &ProjectContext<WithApps>,
    ) -> BoxedHandler {
        let handler = handler.middleware(StaticFilesMiddleware::from_context(context));
        #[cfg(debug_assertions)]
        let handler = handler.middleware(cot::middleware::LiveReloadMiddleware::new());
        handler.build()
    }
}

#[cot::main]
fn main() -> impl Project {
    CotSiteProject
}
