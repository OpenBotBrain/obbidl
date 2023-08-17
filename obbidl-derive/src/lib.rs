use std::{env, fs, path::Path};

use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;

#[proc_macro]
pub fn include_obbidl_file(tokens: TokenStream) -> TokenStream {
    let cwd = env::current_dir().unwrap();
    let str_lit: LitStr = syn::parse_macro_input!(tokens);
    let str = str_lit.value();
    let path = cwd.join(&str);
    let path_str = path.to_str().unwrap();
    let source = fs::read_to_string(&path).unwrap();
    let output = obbidl::build1(&source);

    let out_dir = env::var("OUT_DIR").unwrap();
    let output_path = Path::new(&out_dir).join(&str).with_extension("rs");
    let output_path_str = output_path.to_str();
    fs::write(&output_path, output).unwrap();

    quote! {
        const SOURCE: &str = include_str!(#path_str);
        include!(#output_path_str);
    }
    .into()
}
