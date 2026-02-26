use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub location: Location,
    #[serde(default)]
    pub defaults: Defaults,
    pub color_curve: Option<Vec<ColorKeyframe>>,
    #[serde(default)]
    pub backends: Backends,
    pub zone: Vec<ZoneConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    #[serde(default)]
    pub sunrise_offset: i16,
    #[serde(default)]
    pub sunset_offset: i16,
    #[serde(default)]
    pub night_brightness: f64,
    #[serde(default = "default_interval")]
    pub interval: u16,
}

fn default_interval() -> u16 {
    10
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            sunrise_offset: 0,
            sunset_offset: 0,
            night_brightness: 0.0,
            interval: default_interval(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColorKeyframe {
    pub elevation: f64,
    pub temp: u16,
    pub brightness: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct Backends {
    #[serde(default)]
    pub gpio: HashMap<String, GpioConnection>,
    #[serde(default)]
    pub deconz: HashMap<String, DeconzConnection>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GpioConnection {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeconzConnection {
    pub host: String,
    #[serde(default = "default_deconz_port")]
    pub port: u16,
    pub api_key: String,
}

fn default_deconz_port() -> u16 {
    80
}

#[derive(Debug, Deserialize)]
pub struct ZoneConfig {
    pub name: String,
    pub sunrise_offset: Option<i16>,
    pub sunset_offset: Option<i16>,
    pub color_curve: Option<Vec<ColorKeyframe>>,
    pub light: Vec<LightConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LightConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub light_type: String,
    pub backend: String,
    // Type-specific fields (flat, validated later)
    pub temp: Option<u16>,
    pub cold_temp: Option<u16>,
    pub warm_temp: Option<u16>,
    pub white_temp: Option<u16>,
    // GPIO-specific
    pub pin: Option<u8>,
    pub cold_pin: Option<u8>,
    pub warm_pin: Option<u8>,
    pub pwm_frequency: Option<u32>,
    // deCONZ-specific
    pub light_id: Option<u16>,
    pub group_id: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Mono { temp: u16 },
    Dual { cold_temp: u16, warm_temp: u16 },
    Rgb,
    Wrgb { white_temp: u16 },
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}
