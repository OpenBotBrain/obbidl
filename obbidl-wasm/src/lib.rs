use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

use obbidl_lib::{parser::parse, ast::File};

#[wasm_bindgen]
pub fn generate_fsm(protocol: &str) -> String {


    let status = match parse::<File>(protocol) {
        Ok(_) => "Success!",
        Err(_) => "Error!",
    };
    status.into()
}
