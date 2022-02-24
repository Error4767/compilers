use wasm_bindgen::prelude::wasm_bindgen;

mod xml_compile;
use xml_compile::xml_compile;

#[wasm_bindgen]
pub fn compile_xml(xml_raw: &str)-> String {
    xml_compile(xml_raw)
}
