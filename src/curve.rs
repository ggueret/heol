use crate::config::ColorKeyframe;

#[derive(Debug, Clone)]
pub struct ColorCurve {
    keyframes: Vec<ColorKeyframe>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetState {
    pub brightness: f64,
    pub color_temp_k: f64,
}

impl ColorCurve {
    pub fn new(keyframes: Vec<ColorKeyframe>) -> Self {
        Self { keyframes }
    }

    pub fn builtin() -> Self {
        Self::new(vec![
            ColorKeyframe { elevation: -6.0, temp: 2000, brightness: 0.02 },
            ColorKeyframe { elevation: -1.0, temp: 2500, brightness: 0.15 },
            ColorKeyframe { elevation: 0.0, temp: 3200, brightness: 0.35 },
            ColorKeyframe { elevation: 5.0, temp: 4200, brightness: 0.60 },
            ColorKeyframe { elevation: 15.0, temp: 5500, brightness: 0.85 },
            ColorKeyframe { elevation: 30.0, temp: 6500, brightness: 1.0 },
        ])
    }

    pub fn resolve(&self, elevation: f64) -> TargetState {
        let kf = &self.keyframes;

        if kf.is_empty() {
            return TargetState { brightness: 0.0, color_temp_k: 4000.0 };
        }

        // Clamp below
        if elevation <= kf[0].elevation {
            return TargetState {
                brightness: kf[0].brightness,
                color_temp_k: kf[0].temp as f64,
            };
        }

        // Clamp above
        if elevation >= kf[kf.len() - 1].elevation {
            return TargetState {
                brightness: kf[kf.len() - 1].brightness,
                color_temp_k: kf[kf.len() - 1].temp as f64,
            };
        }

        // Find surrounding keyframes and interpolate
        for window in kf.windows(2) {
            let (lo, hi) = (&window[0], &window[1]);
            if elevation >= lo.elevation && elevation <= hi.elevation {
                let t = (elevation - lo.elevation) / (hi.elevation - lo.elevation);
                return TargetState {
                    brightness: lo.brightness + t * (hi.brightness - lo.brightness),
                    color_temp_k: lo.temp as f64 + t * (hi.temp as f64 - lo.temp as f64),
                };
            }
        }

        unreachable!("elevation should be within keyframe range")
    }
}
