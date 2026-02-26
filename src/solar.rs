use chrono::{DateTime, Utc};
use solar_positioning::{spa, RefractionCorrection};

pub struct SolarEngine {
    latitude: f64,
    longitude: f64,
    elevation: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SolarState {
    pub elevation: f64,
    pub azimuth: f64,
}

impl SolarEngine {
    pub fn new(latitude: f64, longitude: f64, elevation: Option<f64>) -> Self {
        Self {
            latitude,
            longitude,
            elevation: elevation.unwrap_or(0.0),
        }
    }

    pub fn position(&self, dt: DateTime<Utc>) -> SolarState {
        // Delta T ~69s is a reasonable estimate for 2025-2026
        let delta_t = 69.0;
        let pos = spa::solar_position(
            dt,
            self.latitude,
            self.longitude,
            self.elevation,
            delta_t,
            Some(RefractionCorrection::standard()),
        )
        .expect("solar position calculation should not fail for valid coordinates");

        SolarState {
            elevation: pos.elevation_angle(),
            azimuth: pos.azimuth(),
        }
    }
}
