use proc_macro::TokenStream;
use syn::LitStr;

#[proc_macro]
pub fn include_obbidl_file(tokens: TokenStream) -> TokenStream {
    let path: LitStr = syn::parse_macro_input!(tokens);
    let output = obbidl::build1(&path.value());
    output.parse().unwrap()
}
