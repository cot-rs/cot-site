use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use cot::request::RequestHead;
use cot::request::extractors::FromRequestHead;
use cot::router::Urls;
use cot_site_common::ALL_VERSIONS;
use pagefind::api::PagefindIndex;
use pagefind::options::PagefindServiceConfig;
use tracing::info;

use crate::guides::parse_guides;

static SEARCH_INDEX: tokio::sync::OnceCell<SearchIndex> = tokio::sync::OnceCell::const_new();

pub const SEARCH_INDEX_TIMEOUT: Duration = Duration::from_secs(365 * 24 * 60 * 60); // 1 year

#[derive(Debug, Clone)]
pub struct SearchIndex {
    files: Arc<HashMap<String, Vec<u8>>>,
}

impl SearchIndex {
    pub async fn generate(urls: Urls) -> cot::Result<Self> {
        let options = PagefindServiceConfig::builder()
            .keep_index_url(true)
            .force_language("en".to_string())
            .build();

        let mut indexer = PagefindIndex::new(Some(options))
            .map_err(|e| cot::Error::internal(format!("Failed to initialize Pagefind: {}", e)))?;

        for &version in ALL_VERSIONS {
            let (_, guide_map) = parse_guides(version);
            for (page_id, page) in guide_map {
                let url = cot::reverse!(urls, "guide_page", version = version, page = page_id)
                    .expect("Failed to reverse URL for guide page");
                let html = format!(
                    r#"<html><body><article data-pagefind-body data-pagefind-filter="version:{}"><h1>{}</h1>{}</article></body></html>"#,
                    version, page.title, page.content_html
                );
                indexer
                    .add_html_file(None, Some(url), html)
                    .await
                    .map_err(|e| {
                        cot::Error::internal(format!("Failed to add HTML to index: {}", e))
                    })?;
            }
        }

        let files = indexer
            .get_files()
            .await
            .map_err(|e| cot::Error::internal(format!("Failed to get Pagefind files: {}", e)))?;

        let mut files_map = HashMap::new();
        for file in files {
            files_map.insert(file.filename.to_string_lossy().to_string(), file.contents);
        }

        Ok(Self {
            files: Arc::new(files_map),
        })
    }

    pub fn get_file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(|v| v.as_slice())
    }

    pub fn get_pagefind_url(&self, urls: &Urls) -> String {
        let url = cot::reverse!(urls, "serve_pagefind", file = "pagefind.js")
            .expect("Failed to reverse URL for pagefind.js");
        let file = self
            .get_file("pagefind.js")
            .expect("pagefind.js should be in the search index");
        let file_hash = hex::encode(&blake3::hash(file).as_slice()[0..6]);
        format!("{}?v={}", url, file_hash)
    }
}

impl FromRequestHead for SearchIndex {
    async fn from_request_head(head: &RequestHead) -> cot::Result<Self> {
        let urls = Urls::from_request_head(head)
            .await
            .expect("Failed to extract URLs from request head");
        let index = SEARCH_INDEX
            .get_or_init(async || build_search_index(urls).await)
            .await;
        Ok(index.clone())
    }
}

async fn build_search_index(urls: Urls) -> SearchIndex {
    tokio::task::spawn_blocking(move || {
        // SearchIndex::generate is not Send due to the PagefindIndex it uses
        // internally, so we run it in a separate tokio runtime
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            info!("Generating search index...");
            match SearchIndex::generate(urls).await {
                Ok(index) => {
                    info!("Search index generated successfully");
                    index
                }
                Err(e) => {
                    panic!("Failed to generate search index: {}", e);
                }
            }
        })
    })
    .await
    .expect("Failed to spawn blocking task for search index")
}
