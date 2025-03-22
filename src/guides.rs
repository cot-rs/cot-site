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

fn get_categories_for_version(version: &str) -> [(&'static str, Vec<MdPage>); 2] {
    [
        (
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
        ),
        ("About", vec![md_page!("v0.1", "framework-comparison")]),
    ]
}
