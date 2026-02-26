use chrono::{TimeZone, Utc};
use heol::solar::SolarEngine;

#[test]
fn paris_noon_summer_high_elevation() {
    let engine = SolarEngine::new(48.8566, 2.3522, None);
    // June 21st 2025 at solar noon in Paris (~12:00 UTC)
    let dt = Utc.with_ymd_and_hms(2025, 6, 21, 12, 0, 0).unwrap();
    let state = engine.position(dt);
    // Sun should be high (>55°) at solar noon in Paris in June
    assert!(state.elevation > 55.0, "elevation was {}", state.elevation);
    assert!(state.elevation < 70.0, "elevation was {}", state.elevation);
}

#[test]
fn paris_midnight_winter_negative_elevation() {
    let engine = SolarEngine::new(48.8566, 2.3522, None);
    // Dec 21st at midnight UTC
    let dt = Utc.with_ymd_and_hms(2025, 12, 21, 0, 0, 0).unwrap();
    let state = engine.position(dt);
    // Sun should be well below horizon
    assert!(state.elevation < -10.0, "elevation was {}", state.elevation);
}

#[test]
fn equator_equinox_noon_near_90() {
    let engine = SolarEngine::new(0.0, 0.0, None);
    // March equinox, ~noon at longitude 0
    let dt = Utc.with_ymd_and_hms(2025, 3, 20, 12, 0, 0).unwrap();
    let state = engine.position(dt);
    // Sun should be very high (near zenith)
    assert!(state.elevation > 80.0, "elevation was {}", state.elevation);
}

#[test]
fn azimuth_in_valid_range() {
    let engine = SolarEngine::new(48.8566, 2.3522, None);
    let dt = Utc.with_ymd_and_hms(2025, 6, 21, 12, 0, 0).unwrap();
    let state = engine.position(dt);
    assert!(
        state.azimuth >= 0.0 && state.azimuth < 360.0,
        "azimuth was {}",
        state.azimuth
    );
}
