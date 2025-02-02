use proc_macro::TokenStream;

use crate::guides::MdGuide;

mod guides;

#[proc_macro]
pub fn md_guide(input: TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let MdGuide { link } = syn::parse2(input).unwrap();

    let guide = guides::parse_guide(&link);
    guides::quote_guide(&guide).into()
}
