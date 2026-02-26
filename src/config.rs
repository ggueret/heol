use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("{0}")]
    Validation(String),
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, ConfigError> {
        Ok(toml::from_str(s)?)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Brightness range
        if !(0.0..=1.0).contains(&self.defaults.night_brightness) {
            return Err(ConfigError::Validation(
                "defaults.night_brightness must be between 0.0 and 1.0".into(),
            ));
        }

        // Color curve sorted
        if let Some(curve) = &self.color_curve {
            validate_curve_sorted(curve, "global color_curve")?;
            validate_curve_brightness(curve, "global color_curve")?;
        }

        // Zones
        for zone in &self.zone {
            if let Some(curve) = &zone.color_curve {
                validate_curve_sorted(curve, &format!("zone '{}' color_curve", zone.name))?;
                validate_curve_brightness(curve, &format!("zone '{}' color_curve", zone.name))?;
            }

            for light in &zone.light {
                self.validate_light(light)?;
            }
        }

        Ok(())
    }

    fn validate_light(&self, light: &LightConfig) -> Result<(), ConfigError> {
        let (backend_type, profile_name) = light.backend.split_once('.').ok_or_else(|| {
            ConfigError::Validation(format!(
                "light '{}': backend '{}' must be in 'type.name' format",
                light.name, light.backend
            ))
        })?;

        // Check backend profile exists
        match backend_type {
            "gpio" => {
                if !self.backends.gpio.contains_key(profile_name) {
                    return Err(ConfigError::Validation(format!(
                        "light '{}': backend '{}' not found in [backends.gpio]",
                        light.name, light.backend
                    )));
                }
                self.validate_gpio_light(light)?;
            }
            "deconz" => {
                if !self.backends.deconz.contains_key(profile_name) {
                    return Err(ConfigError::Validation(format!(
                        "light '{}': backend '{}' not found in [backends.deconz]",
                        light.name, light.backend
                    )));
                }
                self.validate_deconz_light(light)?;
            }
            _ => {
                return Err(ConfigError::Validation(format!(
                    "light '{}': unknown backend type '{backend_type}'",
                    light.name
                )));
            }
        }

        Ok(())
    }

    fn validate_gpio_light(&self, light: &LightConfig) -> Result<(), ConfigError> {
        match light.light_type.as_str() {
            "mono" => {
                if light.pin.is_none() {
                    return Err(ConfigError::Validation(format!(
                        "light '{}': mono gpio requires pin",
                        light.name
                    )));
                }
            }
            "dual" => {
                if light.cold_pin.is_none() || light.warm_pin.is_none() {
                    return Err(ConfigError::Validation(format!(
                        "light '{}': dual gpio requires cold_pin and warm_pin",
                        light.name
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_deconz_light(&self, light: &LightConfig) -> Result<(), ConfigError> {
        match (light.light_id, light.group_id) {
            (None, None) => Err(ConfigError::Validation(format!(
                "light '{}': deconz requires light_id or group_id",
                light.name
            ))),
            (Some(_), Some(_)) => Err(ConfigError::Validation(format!(
                "light '{}': light_id and group_id are mutually exclusive",
                light.name
            ))),
            _ => Ok(()),
        }
    }
}

fn validate_curve_sorted(curve: &[ColorKeyframe], context: &str) -> Result<(), ConfigError> {
    for window in curve.windows(2) {
        if window[1].elevation <= window[0].elevation {
            return Err(ConfigError::Validation(format!(
                "{context}: keyframes must be sorted by ascending elevation"
            )));
        }
    }
    Ok(())
}

fn validate_curve_brightness(curve: &[ColorKeyframe], context: &str) -> Result<(), ConfigError> {
    for kf in curve {
        if !(0.0..=1.0).contains(&kf.brightness) {
            return Err(ConfigError::Validation(format!(
                "{context}: brightness must be between 0.0 and 1.0 (got {} at elevation {})",
                kf.brightness, kf.elevation
            )));
        }
    }
    Ok(())
}
