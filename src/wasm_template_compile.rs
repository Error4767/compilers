use wasm_bindgen::prelude::wasm_bindgen;

mod template_compile;
use template_compile::template_compile;

#[wasm_bindgen]
pub fn compile_template(xml_raw: &str)-> String {
    template_compile(xml_raw)
}
