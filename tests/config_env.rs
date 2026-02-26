use heol::config::Config;

#[test]
fn env_overrides_deconz_api_key() {
    let toml_str = r#"
[location]
latitude = 48.0
longitude = 2.0
[defaults]
interval = 10
[backends.deconz.home]
host = "192.168.1.10"
port = 80
api_key = "original"
[[zone]]
name = "z"
  [[zone.light]]
  name = "l"
  type = "mono"
  backend = "deconz.home"
  temp = 4500
  light_id = 1
"#;
    unsafe { std::env::set_var("HEOL_DECONZ_HOME_API_KEY", "secret123") };
    let config = Config::from_toml(toml_str).unwrap().with_env_overrides();
    assert_eq!(config.backends.deconz["home"].api_key, "secret123");
    unsafe { std::env::remove_var("HEOL_DECONZ_HOME_API_KEY") };
}

#[test]
fn env_overrides_gpio_host() {
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
  pin = 17
"#;
    unsafe { std::env::set_var("HEOL_GPIO_LOCAL_HOST", "10.0.0.1") };
    let config = Config::from_toml(toml_str).unwrap().with_env_overrides();
    assert_eq!(config.backends.gpio["local"].host, "10.0.0.1");
    unsafe { std::env::remove_var("HEOL_GPIO_LOCAL_HOST") };
}
