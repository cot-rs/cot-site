use std::collections::HashMap;

use cot_site_common::md_pages::{MdPage, MdPageLink};
use cot_site_macros::md_page;

use crate::GuideLinkCategory;

pub fn parse_guides(version: &str) -> (Vec<GuideLinkCategory>, HashMap<String, MdPage>) {
    let categories = get_categories_for_version(version);

    let categories_links = categories
        .iter()
        .map(|(title, guides)| GuideLinkCategory {
            title,
            guides: guides.iter().map(MdPageLink::from).collect(),
        })
        .collect();
    let guide_map = categories
        .into_iter()
        .flat_map(|(_title, guides)| guides)
        .map(|guide| (guide.link.clone(), guide))
        .collect();

    (categories_links, guide_map)
}

pub fn get_prev_next_link<'a>(
    guides: &'a [GuideLinkCategory],
    current_id: &str,
) -> (Option<&'a MdPageLink>, Option<&'a MdPageLink>) {
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

fn get_categories_for_version(version: &str) -> Vec<(&'static str, Vec<MdPage>)> {
    match version {
        "v0.1" => vec![(
            "Getting started",
            vec![
                md_page!("v0.1", "introduction"),
                md_page!("v0.1", "templates"),
                md_page!("v0.1", "forms"),
                md_page!("v0.1", "db-models"),
                md_page!("v0.1", "admin-panel"),
                md_page!("v0.1", "static-files"),
                md_page!("v0.1", "error-pages"),
                md_page!("v0.1", "testing"),
            ],
        )],
        "v0.2" => vec![(
            "Getting started",
            vec![
                md_page!("v0.2", "introduction"),
                md_page!("v0.2", "templates"),
                md_page!("v0.2", "forms"),
                md_page!("v0.2", "db-models"),
                md_page!("v0.2", "admin-panel"),
                md_page!("v0.2", "static-files"),
                md_page!("v0.2", "error-pages"),
                md_page!("v0.2", "testing"),
            ],
        )],
        "v0.3" => vec![(
            "Getting started",
            vec![
                md_page!("v0.3", "introduction"),
                md_page!("v0.3", "templates"),
                md_page!("v0.3", "forms"),
                md_page!("v0.3", "db-models"),
                md_page!("v0.3", "admin-panel"),
                md_page!("v0.3", "static-files"),
                md_page!("v0.3", "error-pages"),
                md_page!("v0.3", "openapi"),
                md_page!("v0.3", "testing"),
            ],
        )],
        "v0.4" => vec![
            (
                "Getting started",
                vec![
                    md_page!("v0.4", "introduction"),
                    md_page!("v0.4", "templates"),
                    md_page!("v0.4", "forms"),
                    md_page!("v0.4", "db-models"),
                    md_page!("v0.4", "admin-panel"),
                    md_page!("v0.4", "static-files"),
                    md_page!("v0.4", "error-pages"),
                    md_page!("v0.4", "openapi"),
                    md_page!("v0.4", "testing"),
                ],
            ),
            ("Upgrading", vec![md_page!("v0.4", "upgrade-guide")]),
        ],
        "master" => vec![
            (
                "Getting started",
                vec![
                    md_page!("master", "introduction"),
                    md_page!("master", "templates"),
                    md_page!("master", "forms"),
                    md_page!("master", "db-models"),
                    md_page!("master", "admin-panel"),
                    md_page!("master", "static-files"),
                    md_page!("master", "error-pages"),
                    md_page!("master", "openapi"),
                    md_page!("master", "testing"),
                ],
            ),
            ("Upgrading", vec![md_page!("master", "upgrade-guide")]),
            ("About", vec![md_page!("master", "framework-comparison")]),
        ],
        _ => unreachable!(),
    }
}
