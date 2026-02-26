use crate::config::ZoneConfig;
use crate::curve::{ColorCurve, TargetState};

pub fn resolve_zone_target(
    zone: &ZoneConfig,
    global_curve: &ColorCurve,
    elevation: f64,
) -> TargetState {
    let curve = zone
        .color_curve
        .as_ref()
        .map(|kf| ColorCurve::new(kf.clone()))
        .unwrap_or_else(|| global_curve.clone());

    curve.resolve(elevation)
}
