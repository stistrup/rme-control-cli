pub const MAX_RAW: f64 = 65536.0;
const EXPONENT: f64 = 2.0;

pub fn percent_to_raw(percent: f64) -> i32 {
    let percent = percent.clamp(0.0, 100.0);
    let normalized = percent / 100.0;
    let curved = normalized.powf(EXPONENT);
    (curved * MAX_RAW) as i32
}

pub fn raw_to_percent(raw: i32) -> f64 {
    let raw = (raw as f64).clamp(0.0, MAX_RAW);
    let normalized = raw / MAX_RAW;
    let curved = normalized.powf(1.0 / EXPONENT);
    curved * 100.0
}