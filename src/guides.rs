use std::collections::HashMap;

use cot_site_common::md_pages::{MdPage, MdPageLink};
use cot_site_macros::md_page;

use crate::GuideLinkCategory;

pub fn parse_guides() -> (Vec<GuideLinkCategory>, HashMap<String, MdPage>) {
    let categories = [(
        "Getting started",
        vec![
            md_page!("introduction"),
            md_page!("templates"),
            md_page!("forms"),
            md_page!("db-models"),
            md_page!("admin-panel"),
            md_page!("static-files"),
            md_page!("error-pages"),
            md_page!("testing"),
        ],
    )];

    let categories_links = categories
        .iter()
        .map(|(title, guides)| GuideLinkCategory {
            title,
            guides: guides.iter().map(MdPageLink::from).collect(),
        })
        .collect();
    let guide_map = categories
        .into_iter()
        .map(|(_title, guides)| guides)
        .flatten()
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
