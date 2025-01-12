mod utils;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, league-sim!");
}

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
}
