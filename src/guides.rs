use crate::{GuideLinkCategory};
use std::collections::HashMap;
use cot_site_common::guides::{Guide, GuideLink};
use cot_site_macros::md_guide;

pub fn parse_guides() -> (Vec<GuideLinkCategory>, HashMap<String, Guide>) {
    let categories = [(
        "Getting started",
        vec![
            md_guide!("introduction"),
            md_guide!("templates"),
            md_guide!("forms"),
            md_guide!("db-models"),
            md_guide!("admin-panel"),
            md_guide!("static-files"),
            md_guide!("authentication"),
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
        .map(|guide| (guide.link.clone(), guide))
        .collect();

    (categories_links, guide_map)
}

pub fn get_prev_next_link<'a>(
    guides: &'a [GuideLinkCategory],
    current_id: &str,
) -> (Option<&'a GuideLink>, Option<&'a GuideLink>) {
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
