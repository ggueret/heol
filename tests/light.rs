use heol::config::LightType;
use heol::curve::TargetState;
use heol::light::{LightCommand, adapt_light};

#[test]
fn mono_ignores_color_temp() {
    let target = TargetState {
        brightness: 0.85,
        color_temp_k: 5200.0,
    };
    let cmd = adapt_light(LightType::Mono { temp: 4500 }, &target, "gpio");
    match cmd {
        LightCommand::GpioPwm { duty, .. } => {
            let expected = (0.85 * 1_000_000.0) as u32;
            assert!((duty as i64 - expected as i64).unsigned_abs() < 100);
        }
        _ => panic!("expected GpioPwm, got {cmd:?}"),
    }
}

#[test]
fn dual_gpio_interpolates_channels() {
    let target = TargetState {
        brightness: 1.0,
        color_temp_k: 4600.0,
    };
    let lt = LightType::Dual {
        cold_temp: 6500,
        warm_temp: 2700,
    };
    let cmd = adapt_light(lt, &target, "gpio");
    match cmd {
        LightCommand::GpioDualPwm {
            cold_duty,
            warm_duty,
            ..
        } => {
            // ratio = (4600 - 2700) / (6500 - 2700) = 1900 / 3800 = 0.5
            let total = cold_duty + warm_duty;
            assert!(total > 900_000, "total duty too low: {total}");
            let ratio = cold_duty as f64 / total as f64;
            assert!((ratio - 0.5).abs() < 0.05, "ratio was {ratio}");
        }
        _ => panic!("expected GpioDualPwm, got {cmd:?}"),
    }
}

#[test]
fn dual_deconz_converts_to_mireds() {
    let target = TargetState {
        brightness: 0.5,
        color_temp_k: 4000.0,
    };
    let lt = LightType::Dual {
        cold_temp: 6500,
        warm_temp: 2700,
    };
    let cmd = adapt_light(lt, &target, "deconz");
    match cmd {
        LightCommand::DeconzState { bri, ct, on, .. } => {
            assert!(on);
            assert_eq!(bri, 128); // (0.5 * 255.0).round() = 128
            assert_eq!(ct.unwrap(), 250); // 1_000_000 / 4000 = 250
        }
        _ => panic!("expected DeconzState, got {cmd:?}"),
    }
}

#[test]
fn dual_clamps_temp_to_range() {
    // Target temp below warm range
    let target = TargetState {
        brightness: 1.0,
        color_temp_k: 1500.0,
    };
    let lt = LightType::Dual {
        cold_temp: 6500,
        warm_temp: 2700,
    };
    let cmd = adapt_light(lt, &target, "gpio");
    match cmd {
        LightCommand::GpioDualPwm {
            cold_duty,
            warm_duty,
            ..
        } => {
            // Should be fully warm
            assert_eq!(cold_duty, 0);
            assert!(warm_duty > 900_000);
        }
        _ => panic!("expected GpioDualPwm"),
    }
}

#[test]
fn zero_brightness_is_off() {
    let target = TargetState {
        brightness: 0.0,
        color_temp_k: 5000.0,
    };
    let cmd = adapt_light(LightType::Mono { temp: 4500 }, &target, "deconz");
    match cmd {
        LightCommand::DeconzState { on, bri, .. } => {
            assert!(!on);
            assert_eq!(bri, 0);
        }
        _ => panic!("expected DeconzState"),
    }
}

#[test]
fn rgb_deconz_produces_xy() {
    let target = TargetState {
        brightness: 0.8,
        color_temp_k: 5500.0,
    };
    let cmd = adapt_light(LightType::Rgb, &target, "deconz");
    match cmd {
        LightCommand::DeconzRgb { bri, xy, on, .. } => {
            assert!(on);
            assert_eq!(bri, 204); // 0.8 * 255
            // CIE xy for ~5500K should be roughly (0.33, 0.34)
            assert!(xy.0 > 0.30 && xy.0 < 0.40, "x was {}", xy.0);
            assert!(xy.1 > 0.30 && xy.1 < 0.40, "y was {}", xy.1);
        }
        _ => panic!("expected DeconzRgb, got {cmd:?}"),
    }
}
