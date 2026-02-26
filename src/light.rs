use crate::config::LightType;
use crate::curve::TargetState;

const MAX_GPIO_DUTY: u32 = 1_000_000;

#[derive(Debug, Clone)]
pub enum LightCommand {
    GpioPwm {
        pin: u8,
        duty: u32,
    },
    GpioDualPwm {
        cold_pin: u8,
        warm_pin: u8,
        cold_duty: u32,
        warm_duty: u32,
    },
    DeconzState {
        light_id: Option<u16>,
        group_id: Option<u16>,
        on: bool,
        bri: u8,
        ct: Option<u16>,
    },
    DeconzRgb {
        light_id: Option<u16>,
        group_id: Option<u16>,
        on: bool,
        bri: u8,
        xy: (f64, f64),
    },
}

/// Adapt a TargetState to a LightCommand based on light type and backend.
/// `backend_type` is "gpio" or "deconz" (the prefix before the dot).
pub fn adapt_light(
    light_type: LightType,
    target: &TargetState,
    backend_type: &str,
) -> LightCommand {
    let on = target.brightness > 0.001;
    let bri = (target.brightness * 255.0).round() as u8;

    match (light_type, backend_type) {
        (LightType::Mono { .. }, "gpio") => {
            let duty = (target.brightness * MAX_GPIO_DUTY as f64).round() as u32;
            LightCommand::GpioPwm { pin: 0, duty }
        }
        (LightType::Mono { .. }, _) => LightCommand::DeconzState {
            light_id: None,
            group_id: None,
            on,
            bri,
            ct: Some(kelvin_to_mireds(target.color_temp_k)),
        },
        (
            LightType::Dual {
                cold_temp,
                warm_temp,
            },
            "gpio",
        ) => {
            let (cold_duty, warm_duty) =
                dual_gpio_duties(target.brightness, target.color_temp_k, cold_temp, warm_temp);
            LightCommand::GpioDualPwm {
                cold_pin: 0,
                warm_pin: 0,
                cold_duty,
                warm_duty,
            }
        }
        (LightType::Dual { .. }, _) => LightCommand::DeconzState {
            light_id: None,
            group_id: None,
            on,
            bri,
            ct: Some(kelvin_to_mireds(target.color_temp_k)),
        },
        (LightType::Rgb, _) | (LightType::Wrgb { .. }, _) => {
            let xy = kelvin_to_cie_xy(target.color_temp_k);
            LightCommand::DeconzRgb {
                light_id: None,
                group_id: None,
                on,
                bri,
                xy,
            }
        }
    }
}

fn dual_gpio_duties(
    brightness: f64,
    target_temp: f64,
    cold_temp: u16,
    warm_temp: u16,
) -> (u32, u32) {
    let range = cold_temp as f64 - warm_temp as f64;
    let ratio = ((target_temp - warm_temp as f64) / range).clamp(0.0, 1.0);
    let cold_duty = (brightness * ratio * MAX_GPIO_DUTY as f64).round() as u32;
    let warm_duty = (brightness * (1.0 - ratio) * MAX_GPIO_DUTY as f64).round() as u32;
    (cold_duty, warm_duty)
}

fn kelvin_to_mireds(kelvin: f64) -> u16 {
    (1_000_000.0 / kelvin).round() as u16
}

/// Convert color temperature in Kelvin to CIE 1931 xy coordinates.
/// Uses Planckian locus approximation (Kim et al. 2002).
fn kelvin_to_cie_xy(kelvin: f64) -> (f64, f64) {
    let t = kelvin;
    let t2 = t * t;
    let t3 = t2 * t;

    let x = if t <= 4000.0 {
        -0.2661239e9 / t3 - 0.2343589e6 / t2 + 0.8776956e3 / t + 0.179910
    } else {
        -3.0258469e9 / t3 + 2.1070379e6 / t2 + 0.2226347e3 / t + 0.240390
    };

    let x2 = x * x;
    let x3 = x2 * x;

    let y = if t <= 2222.0 {
        -1.1063814 * x3 - 1.34811020 * x2 + 2.18555832 * x - 0.20219683
    } else if t <= 4000.0 {
        -0.9549476 * x3 - 1.37418593 * x2 + 2.09137015 * x - 0.16748867
    } else {
        3.0817580 * x3 - 5.87338670 * x2 + 3.75112997 * x - 0.37001483
    };

    (x, y)
}
