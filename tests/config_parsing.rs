use heol::config::Config;

#[test]
fn parse_minimal_config() {
    let toml_str = include_str!("fixtures/minimal.toml");
    let config = Config::from_toml(toml_str).unwrap();

    assert_eq!(config.location.latitude, 48.957145);
    assert_eq!(config.location.longitude, 2.880308);
    assert!(config.location.elevation.is_none());
    assert_eq!(config.defaults.interval, 10);
    assert_eq!(config.zone.len(), 1);
    assert_eq!(config.zone[0].light.len(), 1);
    assert_eq!(config.zone[0].light[0].name, "test light");
}

#[test]
fn parse_full_config() {
    let toml_str = include_str!("fixtures/full.toml");
    let config = Config::from_toml(toml_str).unwrap();

    assert_eq!(config.location.elevation, Some(39.0));
    assert_eq!(config.zone.len(), 2);
    assert_eq!(config.backends.gpio.len(), 2);
    assert_eq!(config.backends.deconz.len(), 1);
    assert_eq!(config.color_curve.as_ref().unwrap().len(), 3);

    // Zone overrides
    assert_eq!(config.zone[1].sunrise_offset, Some(-30));

    // Light types
    let dual = &config.zone[0].light[0];
    assert_eq!(dual.cold_temp, Some(6500));
    assert_eq!(dual.warm_temp, Some(2700));

    let deconz_light = &config.zone[1].light[0];
    assert_eq!(deconz_light.light_id, Some(8));
}

#[test]
fn parse_defaults_have_sane_values() {
    let toml_str = include_str!("fixtures/minimal.toml");
    let config = Config::from_toml(toml_str).unwrap();

    assert_eq!(config.defaults.sunrise_offset, 0);
    assert_eq!(config.defaults.sunset_offset, 0);
    assert_eq!(config.defaults.night_brightness, 0.0);
}
