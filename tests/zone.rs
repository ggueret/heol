use heol::config::{ColorKeyframe, ZoneConfig};
use heol::curve::ColorCurve;
use heol::zone::resolve_zone_target;

#[test]
fn zone_uses_own_curve_when_present() {
    let zone = ZoneConfig {
        name: "test".to_string(),
        sunrise_offset: None,
        sunset_offset: None,
        color_curve: Some(vec![
            ColorKeyframe { elevation: 0.0, temp: 3000, brightness: 0.5 },
            ColorKeyframe { elevation: 30.0, temp: 5000, brightness: 1.0 },
        ]),
        light: vec![],
    };
    let global = ColorCurve::builtin();
    let target = resolve_zone_target(&zone, &global, 15.0);
    // Should use zone curve: midpoint between 3000/5000K
    assert!((target.color_temp_k - 4000.0).abs() < 50.0);
}

#[test]
fn zone_falls_back_to_global_curve() {
    let zone = ZoneConfig {
        name: "test".to_string(),
        sunrise_offset: None,
        sunset_offset: None,
        color_curve: None,
        light: vec![],
    };
    let global = ColorCurve::builtin();
    let target = resolve_zone_target(&zone, &global, 30.0);
    assert_eq!(target.color_temp_k, 6500.0);
    assert_eq!(target.brightness, 1.0);
}
