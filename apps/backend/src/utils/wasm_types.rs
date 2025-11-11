use worker::wasm_bindgen::JsValue;

pub fn js_number(value: i64) -> JsValue {
    JsValue::from_f64(value as f64)
}

pub fn js_optional_number(value: Option<i64>) -> JsValue {
    value
        .map(|v| JsValue::from_f64(v as f64))
        .unwrap_or(JsValue::NULL)
}

pub fn js_optional_string(value: Option<&String>) -> JsValue {
    value.map(|v| JsValue::from_str(v)).unwrap_or(JsValue::NULL)
}
