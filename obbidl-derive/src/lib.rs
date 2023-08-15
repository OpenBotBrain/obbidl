use proc_macro::TokenStream;
use syn::LitStr;

#[proc_macro]
pub fn include_obbidl_file(tokens: TokenStream) -> TokenStream {
    let path: LitStr = syn::parse_macro_input!(tokens);
    let path = path.value();
    let output = obbidl::build1(&path);
    format!(
        "const _ : &str = include_str!(\"../{}\");\n{}",
        path, output
    )
    .parse()
    .unwrap()
}
