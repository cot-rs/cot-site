#![cfg_attr(cot_use_nightly, feature(proc_macro_tracked_path))]

use proc_macro::TokenStream;

use crate::md_pages::{ExternalMdPageInput, MdPageInput};

mod md_pages;

#[proc_macro]
pub fn md_page(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let MdPageInput { prefix, link } = syn::parse2(input).unwrap();

    let md_page = md_pages::parse_md_page(&format!("docs/{prefix}"), &link, &prefix);
    md_pages::quote_md_page(&md_page).into()
}

#[proc_macro]
pub fn external_md_page(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let ExternalMdPageInput { link } = syn::parse2(input).unwrap();

    let md_page = md_pages::parse_md_page("..", &link, "master");
    md_pages::quote_md_page(&md_page).into()
}
