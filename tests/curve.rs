use heol::config::ColorKeyframe;
use heol::curve::ColorCurve;

fn make_curve() -> ColorCurve {
    ColorCurve::new(vec![
        ColorKeyframe { elevation: -6.0, temp: 2000, brightness: 0.02 },
        ColorKeyframe { elevation: 0.0, temp: 3200, brightness: 0.35 },
        ColorKeyframe { elevation: 30.0, temp: 6500, brightness: 1.0 },
    ])
}

#[test]
fn interpolates_between_keyframes() {
    let curve = make_curve();
    let state = curve.resolve(15.0); // midpoint between 0° and 30°
    assert!((state.brightness - 0.675).abs() < 0.01);
    assert!((state.color_temp_k - 4850.0).abs() < 50.0);
}

#[test]
fn clamps_below_minimum() {
    let curve = make_curve();
    let state = curve.resolve(-20.0);
    assert_eq!(state.brightness, 0.02);
    assert_eq!(state.color_temp_k, 2000.0);
}

#[test]
fn clamps_above_maximum() {
    let curve = make_curve();
    let state = curve.resolve(60.0);
    assert_eq!(state.brightness, 1.0);
    assert_eq!(state.color_temp_k, 6500.0);
}

#[test]
fn exact_keyframe_match() {
    let curve = make_curve();
    let state = curve.resolve(0.0);
    assert_eq!(state.brightness, 0.35);
    assert_eq!(state.color_temp_k, 3200.0);
}

#[test]
fn builtin_curve_exists() {
    let curve = ColorCurve::builtin();
    let state = curve.resolve(30.0);
    assert_eq!(state.brightness, 1.0);
    assert_eq!(state.color_temp_k, 6500.0);
}
