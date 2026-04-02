mod guides;
mod search;
mod template_util;

use std::sync::Arc;

use askama::filters::{HtmlSafe, Safe};
use async_trait::async_trait;
use cot::error::NotFound;
use cot::error::handler::RequestError;
use cot::html::Html;
use cot::http::header;
use cot::project::App;
use cot::request::extractors::{FromRequestHead, Path, StaticFiles};
use cot::request::{RequestExt, RequestHead};
use cot::response::{IntoResponse, Response};
use cot::router::{Route, Router, Urls};
use cot::static_files::StaticFile;
use cot::{ProjectContext, Template, reverse_redirect, static_files};
pub use cot_site_common;
use cot_site_common::md_pages::{MdPage, MdPageLink, Section};
use cot_site_common::{ALL_VERSIONS, LATEST_VERSION};
pub use cot_site_macros::external_md_page as md_page;
use cot_site_macros::md_page as internal_md_page;

use crate::guides::{ParsedPages, get_categories, get_prev_next_link};
use crate::search::{SEARCH_INDEX, SEARCH_INDEX_TIMEOUT, SearchIndex, build_search_index};

#[derive(Debug, Clone, FromRequestHead)]
pub struct BaseContext {
    urls: Urls,
    static_files: StaticFiles,
    route_name: RouteName,
}

#[derive(Debug, Clone)]
struct RouteName(String);

impl FromRequestHead for RouteName {
    async fn from_request_head(head: &RequestHead) -> cot::Result<Self> {
        let route_name = head.route_name().unwrap_or_default().to_owned();
        Ok(Self(route_name))
    }
}

impl PartialEq<str> for RouteName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    base_context: &'a BaseContext,
}

async fn index(base_context: BaseContext) -> cot::Result<Html> {
    let index_template = IndexTemplate {
        base_context: &base_context,
    };
    let rendered = index_template.render()?;

    Ok(Html::new(rendered))
}

#[derive(Debug, Template)]
#[template(path = "guide.html")]
struct GuideTemplate<'a> {
    link_categories: &'a [GuideLinkCategory],
    guide: &'a MdPage,
    versions: &'static [&'static str],
    version: &'a str,
    display_version: &'a str,
    canonical_link: &'a str,
    base_context: &'a BaseContext,
    search_index: SearchIndex,
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

async fn guide(base_context: BaseContext) -> cot::Result<Response> {
    reverse_redirect!(base_context.urls, "guide_version", version = "latest")
}

async fn guide_version(
    base_context: BaseContext,
    search_index: SearchIndex,
    Path(version): Path<String>,
    pages: Arc<ParsedPages>,
) -> cot::Result<Html> {
    page_response(
        base_context,
        search_index,
        &version,
        DEFAULT_GUIDE_PAGE,
        pages,
    )
}

async fn guide_page(
    base_context: BaseContext,
    search_index: SearchIndex,
    Path((version, page)): Path<(String, String)>,
    pages: Arc<ParsedPages>,
) -> cot::Result<Response> {
    if page == DEFAULT_GUIDE_PAGE {
        return Ok(reverse_redirect!(
            base_context.urls,
            "guide_version",
            version = version
        )?);
    }

    page_response(base_context, search_index, &version, &page, pages).into_response()
}

fn page_response(
    base_context: BaseContext,
    search_index: SearchIndex,
    version: &str,
    page: &str,
    pages: Arc<ParsedPages>,
) -> cot::Result<Html> {
    let file_version = if version == "latest" {
        LATEST_VERSION
    } else {
        version
    };
    let pages = pages
        .version_map
        .get(file_version)
        .ok_or_else(NotFound::new)?;
    let guide = pages.guide_map.get(page).ok_or_else(NotFound::new)?;
    let (prev, next) = get_prev_next_link(&pages.categories_links, page);
    let canonical_link = canonical_link(&base_context.urls, file_version, page)
        .expect("Failed to create canonical link");

    let guide_template = GuideTemplate {
        link_categories: &pages.categories_links,
        guide,
        versions: ALL_VERSIONS,
        version,
        display_version: file_version,
        canonical_link: &canonical_link,
        base_context: &base_context,
        search_index,
        prev,
        next,
    };

    let rendered = guide_template.render()?;
    Ok(Html::new(rendered))
}

fn canonical_link(urls: &Urls, version: &str, page: &str) -> cot::Result<String> {
    const BASE_URL: &str = "https://cot.rs";

    let path = if page == DEFAULT_GUIDE_PAGE {
        cot::reverse!(urls, "guide_version", version = version)?
    } else {
        cot::reverse!(urls, "guide_page", version = version, page = page)?
    };

    Ok(format!("{BASE_URL}{path}"))
}

#[derive(Debug, Template)]
#[template(path = "md_page.html")]
struct MdPageTemplate<'a> {
    page: &'a MdPage,
    base_context: &'a BaseContext,
}

async fn faq(base_context: BaseContext) -> cot::Result<Html> {
    let template = MdPageTemplate {
        page: &internal_md_page!("", "faq"),
        base_context: &base_context,
    };

    Ok(Html::new(template.render()?))
}

async fn licenses(base_context: BaseContext) -> cot::Result<Html> {
    let template = MdPageTemplate {
        page: &internal_md_page!("", "licenses"),
        base_context: &base_context,
    };

    Ok(Html::new(template.render()?))
}

// TODO: remove when Cot supports wildcard routes
async fn serve_pagefind_2(
    index: SearchIndex,
    Path((dir, file)): Path<(String, String)>,
) -> cot::Result<impl IntoResponse> {
    serve_pagefind(index, Path(format!("{dir}/{file}"))).await
}

async fn serve_pagefind(
    index: SearchIndex,
    Path(path): Path<String>,
) -> cot::Result<impl IntoResponse> {
    let content = index.get_file(&path).ok_or_else(NotFound::new)?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    Ok(content
        .to_vec()
        .with_content_type(mime.to_string())
        .with_header(
            header::CACHE_CONTROL,
            header::HeaderValue::from_str(&format!("max-age={}", SEARCH_INDEX_TIMEOUT.as_secs()))
                .expect("failed to create cache control header"),
        ))
}

#[derive(Debug)]
pub struct CotSiteApp {
    pages: Arc<ParsedPages>,
}

impl CotSiteApp {
    /// Creates a new instance of [`CotSiteApp`].
    ///
    /// The `master_pages` parameter should contain a list of sections, where
    /// each section is a tuple containing the name of the section and list
    /// of pages inside it.
    pub fn new(master_pages: Vec<(&'static str, Vec<MdPage>)>) -> Self {
        let pages = get_categories(master_pages);

        Self {
            pages: Arc::new(pages),
        }
    }
}

#[async_trait]
impl App for CotSiteApp {
    fn name(&self) -> &'static str {
        "cot-site"
    }

    fn router(&self) -> Router {
        let pages_guide_version = self.pages.clone();
        let pages_guide_page = self.pages.clone();

        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            Route::with_handler_and_name("/faq/", faq, "faq"),
            Route::with_handler_and_name("/licenses/", licenses, "licenses"),
            Route::with_handler_and_name("/guide/", guide, "guide"),
            Route::with_handler_and_name("/_pagefind/{file}", serve_pagefind, "serve_pagefind"),
            Route::with_handler("/_pagefind/{dir}/{file}", serve_pagefind_2),
            Route::with_handler_and_name(
                "/guide/{version}/",
                async move |base_context: BaseContext,
                            search_index: SearchIndex,
                            path: Path<String>| {
                    guide_version(
                        base_context,
                        search_index,
                        path,
                        Arc::clone(&pages_guide_version),
                    )
                    .await
                },
                "guide_version",
            ),
            Route::with_handler_and_name(
                "/guide/{version}/{page}/",
                async move |base_context: BaseContext,
                            search_index: SearchIndex,
                            path: Path<(String, String)>| {
                    guide_page(
                        base_context,
                        search_index,
                        path,
                        Arc::clone(&pages_guide_page),
                    )
                    .await
                },
                "guide_page",
            ),
        ])
    }

    fn static_files(&self) -> Vec<StaticFile> {
        static_files!(
            "favicon.ico",
            "static/css/main.css",
            "static/js/color-modes.js",
            "static/js/search.js",
            "static/images/cot-dark.svg",
            "static/images/favicon.svg",
            "static/images/favicon-32.png",
            "static/images/favicon-180.png",
            "static/images/favicon-192.png",
            "static/images/favicon-512.png",
            "static/images/search.svg",
            "static/images/site.webmanifest",
        )
    }

    async fn init(&self, context: &mut ProjectContext) -> cot::Result<()> {
        let urls = Urls::from(context);
        let search_index = build_search_index(urls, Arc::clone(&self.pages)).await;

        SEARCH_INDEX
            .set(search_index)
            .expect("search index should be set once");
        Ok(())
    }
}

pub async fn cot_site_handle_error(
    base_context: BaseContext,
    error: RequestError,
) -> cot::Result<impl IntoResponse> {
    #[derive(Debug, Template)]
    #[template(path = "error.html")]
    struct ErrorTemplate<'a> {
        base_context: &'a BaseContext,
        error: RequestError,
    }

    let status_code = error.status_code();

    let error_template = ErrorTemplate {
        base_context: &base_context,
        error,
    };
    let rendered = error_template.render()?;

    Ok(Html::new(rendered).with_status(status_code))
}
