/// Maximum raw value for RME Babyface volume controls
pub const MAX_RAW: f64 = 65536.0;

/// Exponent for the power curve (2.0 = quadratic)
/// Higher values = more granular control at low volumes
const EXPONENT: f64 = 2.0;

/// Convert percentage (0-100) to raw value (0-65536) using power curve
/// This makes low volumes more granular and high volumes faster
pub fn percent_to_raw(percent: f64) -> i32 {
    let percent = percent.clamp(0.0, 100.0);
    let normalized = percent / 100.0;
    let curved = normalized.powf(EXPONENT);
    (curved * MAX_RAW) as i32
}

/// Convert raw value (0-65536) to percentage (0-100) using inverse power curve
pub fn raw_to_percent(raw: i32) -> f64 {
    let raw = (raw as f64).clamp(0.0, MAX_RAW);
    let normalized = raw / MAX_RAW;
    let curved = normalized.powf(1.0 / EXPONENT);
    curved * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_to_raw_boundaries() {
        assert_eq!(percent_to_raw(0.0), 0);
        assert_eq!(percent_to_raw(100.0), 65536);
    }

    #[test]
    fn test_percent_to_raw_midpoint() {
        // With exponent 2.0, 50% should give 25% of raw range
        assert_eq!(percent_to_raw(50.0), 16384);
    }

    #[test]
    fn test_raw_to_percent_boundaries() {
        assert!((raw_to_percent(0) - 0.0).abs() < 0.01);
        assert!((raw_to_percent(65536) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_roundtrip() {
        for percent in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let raw = percent_to_raw(percent);
            let back = raw_to_percent(raw);
            assert!((back - percent).abs() < 0.1, "Roundtrip failed for {}", percent);
        }
    }

    #[test]
    fn test_clamp_above_100() {
        assert_eq!(percent_to_raw(150.0), 65536);
    }

    #[test]
    fn test_clamp_below_0() {
        assert_eq!(percent_to_raw(-10.0), 0);
    }
}
