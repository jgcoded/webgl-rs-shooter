use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;

use crate::{dom::post_message};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ui {
    pub current_player: Option<usize>,
    pub player_color: Option<String>,
    pub cannon_power: Option<u32>,
    pub game_over: Option<bool>,
}

pub fn post_ui_state(state: &Ui) -> Result<(), JsValue> {
    let serialized = JsValue::from_serde(state).unwrap();
    post_message(&serialized)
}
