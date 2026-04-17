//! Mathematical utilities for astronomical calculations
//!
//! This module provides degree-based trigonometric functions and other
//! mathematical utilities needed for prayer time calculations.

/// Cosine of angle in degrees
pub fn dcos(degrees: f64) -> f64 {
    degrees.to_radians().cos()
}

/// Sine of angle in degrees
pub fn dsin(degrees: f64) -> f64 {
    degrees.to_radians().sin()
}

/// Tangent of angle in degrees
pub fn dtan(degrees: f64) -> f64 {
    degrees.to_radians().tan()
}

/// Arc sine in degrees
/// Returns NaN if x is outside [-1, 1]
pub fn darcsin(x: f64) -> f64 {
    if x.abs() > 1.0 {
        f64::NAN
    } else {
        x.asin().to_degrees()
    }
}

/// Arc cosine in degrees
/// Returns NaN if x is outside [-1, 1]
pub fn darccos(x: f64) -> f64 {
    if x.abs() > 1.0 {
        f64::NAN
    } else {
        x.acos().to_degrees()
    }
}

/// Arc cotangent in degrees
/// Returns NaN if x is zero (division by zero)
pub fn darccot(x: f64) -> f64 {
    if x == 0.0 {
        f64::NAN
    } else {
        (1.0 / x).atan().to_degrees()
    }
}

/// Two-argument arc tangent in degrees
pub fn darctan2(y: f64, x: f64) -> f64 {
    y.atan2(x).to_degrees()
}

// pub fn deg_to_rad(degrees: f64) -> f64 {
//     degrees * std::f64::consts::PI / 180.0
// }
// pub fn rad_to_deg(radians: f64) -> f64 {
//     radians * 180.0 / std::f64::consts::PI
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_trig() {
        assert!((dcos(0.0) - 1.0).abs() < 1e-10);
        assert!((dsin(90.0) - 1.0).abs() < 1e-10);
        assert!((dtan(45.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_trig_bounds() {
        assert!(darcsin(2.0).is_nan());
        assert!(darcsin(-2.0).is_nan());
        assert!(darccos(2.0).is_nan());
        assert!(darccos(-2.0).is_nan());
    }

    #[test]
    fn test_darccot_zero() {
        assert!(darccot(0.0).is_nan());
    }
}
