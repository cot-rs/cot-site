#![cfg_attr(feature = "nightly", feature(track_path))]

use proc_macro::TokenStream;

use crate::md_pages::MdPageInput;

mod md_pages;

#[proc_macro]
pub fn md_page(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let MdPageInput { prefix, link } = syn::parse2(input).unwrap();

    let md_page = md_pages::parse_md_page(&prefix, &link);
    md_pages::quote_md_page(&md_page).into()
}
