use std::collections::HashMap;

use cot_site_common::md_pages::{MdPage, MdPageLink};
use cot_site_macros::md_page;

use crate::{GuideCategoryItem, GuideItem, GuideLinkCategory};

pub fn parse_guides(categories: Vec<(&'static str, Vec<GuideItem>)>) -> ParsedPagesForVersion {
    let categories_links = categories
        .iter()
        .map(|(title, items)| GuideLinkCategory {
            title,
            guides: items
                .iter()
                .map(|item| match item {
                    GuideItem::Page(page) => GuideCategoryItem::Page(MdPageLink::from(page)),
                    GuideItem::SubCategory { title, pages } => GuideCategoryItem::SubCategory {
                        title,
                        pages: pages.iter().map(MdPageLink::from).collect(),
                    },
                })
                .collect(),
        })
        .collect();

    let guide_map = categories
        .into_iter()
        .flat_map(|(_title, items)| items)
        .flat_map(|item| match item {
            GuideItem::Page(page) => vec![page],
            GuideItem::SubCategory { pages, .. } => pages,
        })
        .map(|page| (page.link.clone(), page))
        .collect();

    ParsedPagesForVersion {
        categories_links,
        guide_map,
    }
}

#[derive(Debug)]
pub(crate) struct ParsedPagesForVersion {
    pub(crate) categories_links: Vec<GuideLinkCategory>,
    pub(crate) guide_map: HashMap<String, MdPage>,
}

#[derive(Debug)]
pub(crate) struct ParsedPages {
    pub(crate) version_map: HashMap<&'static str, ParsedPagesForVersion>,
}

pub fn get_prev_next_link<'a>(
    guides: &'a [GuideLinkCategory],
    current_id: &str,
) -> (Option<&'a MdPageLink>, Option<&'a MdPageLink>) {
    let all_links: Vec<&MdPageLink> = guides
        .iter()
        .flat_map(|category| category.guides.iter())
        .flat_map(|item| match item {
            GuideCategoryItem::Page(link) => vec![link],
            GuideCategoryItem::SubCategory { pages, .. } => pages.iter().collect(),
        })
        .collect();

    let mut prev = None;
    let mut has_found = false;

    for link in all_links {
        if has_found {
            return (prev, Some(link));
        } else if link.link == current_id {
            has_found = true;
        } else {
            prev = Some(link);
        }
    }

    (prev, None)
}

pub fn get_categories(master_version: Vec<(&'static str, Vec<GuideItem>)>) -> ParsedPages {
    let version_map = HashMap::from([
        (
            "v0.1",
            vec![(
                "Getting started",
                vec![
                    GuideItem::Page(md_page!("v0.1", "introduction")),
                    GuideItem::Page(md_page!("v0.1", "templates")),
                    GuideItem::Page(md_page!("v0.1", "forms")),
                    GuideItem::Page(md_page!("v0.1", "db-models")),
                    GuideItem::Page(md_page!("v0.1", "admin-panel")),
                    GuideItem::Page(md_page!("v0.1", "static-files")),
                    GuideItem::Page(md_page!("v0.1", "error-pages")),
                    GuideItem::Page(md_page!("v0.1", "testing")),
                ],
            )],
        ),
        (
            "v0.2",
            vec![(
                "Getting started",
                vec![
                    GuideItem::Page(md_page!("v0.2", "introduction")),
                    GuideItem::Page(md_page!("v0.2", "templates")),
                    GuideItem::Page(md_page!("v0.2", "forms")),
                    GuideItem::Page(md_page!("v0.2", "db-models")),
                    GuideItem::Page(md_page!("v0.2", "admin-panel")),
                    GuideItem::Page(md_page!("v0.2", "static-files")),
                    GuideItem::Page(md_page!("v0.2", "error-pages")),
                    GuideItem::Page(md_page!("v0.2", "testing")),
                ],
            )],
        ),
        (
            "v0.3",
            vec![(
                "Getting started",
                vec![
                    GuideItem::Page(md_page!("v0.3", "introduction")),
                    GuideItem::Page(md_page!("v0.3", "templates")),
                    GuideItem::Page(md_page!("v0.3", "forms")),
                    GuideItem::Page(md_page!("v0.3", "db-models")),
                    GuideItem::Page(md_page!("v0.3", "admin-panel")),
                    GuideItem::Page(md_page!("v0.3", "static-files")),
                    GuideItem::Page(md_page!("v0.3", "error-pages")),
                    GuideItem::Page(md_page!("v0.3", "openapi")),
                    GuideItem::Page(md_page!("v0.3", "testing")),
                ],
            )],
        ),
        (
            "v0.4",
            vec![
                (
                    "Getting started",
                    vec![
                        GuideItem::Page(md_page!("v0.4", "introduction")),
                        GuideItem::Page(md_page!("v0.4", "templates")),
                        GuideItem::Page(md_page!("v0.4", "forms")),
                        GuideItem::Page(md_page!("v0.4", "db-models")),
                        GuideItem::Page(md_page!("v0.4", "admin-panel")),
                        GuideItem::Page(md_page!("v0.4", "static-files")),
                        GuideItem::Page(md_page!("v0.4", "error-pages")),
                        GuideItem::Page(md_page!("v0.4", "openapi")),
                        GuideItem::Page(md_page!("v0.4", "testing")),
                    ],
                ),
                (
                    "Upgrading",
                    vec![GuideItem::Page(md_page!("v0.4", "upgrade-guide"))],
                ),
            ],
        ),
        (
            "v0.5",
            vec![
                (
                    "Getting started",
                    vec![
                        GuideItem::Page(md_page!("v0.5", "introduction")),
                        GuideItem::Page(md_page!("v0.5", "templates")),
                        GuideItem::Page(md_page!("v0.5", "forms")),
                        GuideItem::Page(md_page!("v0.5", "db-models")),
                        GuideItem::Page(md_page!("v0.5", "admin-panel")),
                        GuideItem::Page(md_page!("v0.5", "static-files")),
                        GuideItem::Page(md_page!("v0.5", "sending-emails")),
                        GuideItem::Page(md_page!("v0.5", "caching")),
                        GuideItem::Page(md_page!("v0.5", "error-pages")),
                        GuideItem::Page(md_page!("v0.5", "openapi")),
                        GuideItem::Page(md_page!("v0.5", "testing")),
                    ],
                ),
                (
                    "Upgrading",
                    vec![GuideItem::Page(md_page!("v0.5", "upgrade-guide"))],
                ),
                (
                    "About",
                    vec![GuideItem::Page(md_page!("v0.5", "framework-comparison"))],
                ),
            ],
        ),
        ("master", master_version),
    ]);

    let version_map = version_map
        .into_iter()
        .map(|(version, pages)| (version, parse_guides(pages)))
        .collect();
    ParsedPages { version_map }
}
