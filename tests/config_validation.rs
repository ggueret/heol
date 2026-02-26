use heol::config::Config;

#[test]
fn rejects_unknown_backend_profile() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "gpio.nonexistent"
  temp = 4500
  pin = 17
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("not found"), "got: {err}");
}

#[test]
fn rejects_dual_gpio_missing_pins() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[backends.gpio.local]
host = "localhost"
port = 8888
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "dual"
  backend = "gpio.local"
  cold_temp = 6500
  warm_temp = 2700
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("cold_pin"), "got: {err}");
}

#[test]
fn rejects_mono_gpio_missing_pin() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[backends.gpio.local]
host = "localhost"
port = 8888
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "gpio.local"
  temp = 4500
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("pin"), "got: {err}");
}

#[test]
fn rejects_deconz_missing_light_and_group_id() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[backends.deconz.home]
host = "192.168.1.10"
port = 80
api_key = "test"
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "deconz.home"
  temp = 4500
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(
        err.to_string().contains("light_id or group_id"),
        "got: {err}"
    );
}

#[test]
fn rejects_deconz_both_light_and_group_id() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[backends.deconz.home]
host = "192.168.1.10"
port = 80
api_key = "test"
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "deconz.home"
  temp = 4500
  light_id = 1
  group_id = 2
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("mutually exclusive"), "got: {err}");
}

#[test]
fn rejects_unsorted_color_curve() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[[color_curve]]
elevation = 30.0
temp = 6500
brightness = 1.0
[[color_curve]]
elevation = 0.0
temp = 3200
brightness = 0.35
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "gpio.local"
  temp = 4500
  pin = 17
[backends.gpio.local]
host = "localhost"
port = 8888
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("sorted"), "got: {err}");
}

#[test]
fn rejects_brightness_out_of_range() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
night_brightness = 1.5
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "gpio.local"
  temp = 4500
  pin = 17
[backends.gpio.local]
host = "localhost"
port = 8888
"#;
    let config = Config::from_toml(toml_str).unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("brightness"), "got: {err}");
}

#[test]
fn accepts_valid_full_config() {
    let toml_str = include_str!("fixtures/full.toml");
    let config = Config::from_toml(toml_str).unwrap();
    config.validate().unwrap();
}
