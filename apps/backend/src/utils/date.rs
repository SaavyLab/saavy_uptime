pub fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

pub fn now_s() -> i64 {
    now_ms() / 1000
}
