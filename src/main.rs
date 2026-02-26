use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use clap::Parser;
use tokio::sync::watch;
use tracing_subscriber::EnvFilter;
use heol::config::{Config, LightType};
use heol::curve::ColorCurve;
use heol::solar::SolarEngine;
use heol::scheduler::Scheduler;
use heol::backend::LightBackend;
#[cfg(feature = "gpio")]
use heol::backend::gpio::GpioBackend;
#[cfg(feature = "deconz")]
use heol::backend::deconz::DeconzBackend;
use heol::backend::DryRunBackend;
use heol::zone::resolve_zone_target;
use heol::light::adapt_light;

#[derive(Parser)]
#[command(name = "heol", about = "Solar-synchronized lighting controller")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run the lighting daemon
    Run {
        /// Path to config file
        #[arg(short, long, default_value = "heol.toml")]
        config: PathBuf,

        /// Log commands instead of sending them
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("heol=info"))
        )
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Run { config: config_path, dry_run } => {
            run(config_path, dry_run).await?;
        }
    }

    Ok(())
}

async fn run(config_path: PathBuf, dry_run: bool) -> anyhow::Result<()> {
    // Load config
    let config_str = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", config_path.display()))?;
    let config = Config::from_toml(&config_str)?
        .with_env_overrides();
    config.validate()?;
    tracing::info!(config = %config_path.display(), "configuration loaded");

    // Build solar engine
    let engine = SolarEngine::new(
        config.location.latitude,
        config.location.longitude,
        config.location.elevation,
    );

    // Build global color curve
    let global_curve = config.color_curve
        .as_ref()
        .map(|kf| ColorCurve::new(kf.clone()))
        .unwrap_or_else(ColorCurve::builtin);

    // Build backends
    let mut backends: std::collections::HashMap<String, Arc<dyn LightBackend>> =
        std::collections::HashMap::new();

    #[cfg(feature = "gpio")]
    if !config.backends.gpio.is_empty() {
        let gpio = Arc::new(GpioBackend::new(&config.backends.gpio).await?);
        for name in config.backends.gpio.keys() {
            let backend: Arc<dyn LightBackend> = if dry_run {
                Arc::new(DryRunBackend::new(gpio.clone()))
            } else {
                gpio.clone()
            };
            backends.insert(format!("gpio.{name}"), backend);
        }
    }

    #[cfg(feature = "deconz")]
    if !config.backends.deconz.is_empty() {
        let deconz = Arc::new(DeconzBackend::new(&config.backends.deconz));
        for name in config.backends.deconz.keys() {
            let backend: Arc<dyn LightBackend> = if dry_run {
                Arc::new(DryRunBackend::new(deconz.clone()))
            } else {
                deconz.clone()
            };
            backends.insert(format!("deconz.{name}"), backend);
        }
    }

    // Healthcheck all backends
    if !dry_run {
        for (name, backend) in &backends {
            backend.healthcheck().await
                .map_err(|e| anyhow::anyhow!("healthcheck failed for {name}: {e}"))?;
            tracing::info!(backend = %name, "healthcheck passed");
        }
    }

    // Setup scheduler
    let (solar_tx, solar_rx) = watch::channel(None);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let interval = Duration::from_secs(config.defaults.interval as u64);
    let scheduler = Arc::new(Scheduler::new(engine, solar_tx, interval));

    // Spawn scheduler
    let sched = scheduler.clone();
    let sched_shutdown = shutdown_rx.clone();
    tokio::spawn(async move {
        sched.run(sched_shutdown).await;
    });

    // Spawn zone tasks
    for zone_config in &config.zone {
        let zone_name = zone_config.name.clone();
        let mut rx = solar_rx.clone();
        let zone = zone_config.clone();
        let curve = global_curve.clone();
        let zone_backends = backends.clone();

        tokio::spawn(async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                let solar_state = match rx.borrow().clone() {
                    Some(s) => s,
                    None => continue,
                };

                let elevation = solar_state.elevation;
                let target = resolve_zone_target(&zone, &curve, elevation);

                tracing::debug!(
                    zone = %zone_name,
                    brightness = target.brightness,
                    temp_k = target.color_temp_k,
                    "zone target computed"
                );

                // Send commands to each light
                for light in &zone.light {
                    let backend_type = light.backend.split('.').next().unwrap_or("unknown");
                    let light_type = parse_light_type(light);
                    let command = adapt_light(light_type, &target, backend_type);

                    // Fill in pin/light_id from config
                    let command = fill_command(command, light);

                    if let Some(backend) = zone_backends.get(&light.backend) {
                        if let Err(e) = backend.send(light, command).await {
                            tracing::warn!(
                                light = %light.name,
                                backend = %light.backend,
                                error = %e,
                                "failed to send command"
                            );
                        }
                    } else {
                        tracing::error!(
                            light = %light.name,
                            backend = %light.backend,
                            "backend not found"
                        );
                    }
                }
            }
        });
    }

    // Handle signals
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sighup = signal(SignalKind::hangup())?;
        let mut sigint = signal(SignalKind::interrupt())?;

        loop {
            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("received SIGTERM, shutting down");
                    let _ = shutdown_tx.send(true);
                    break;
                }
                _ = sigint.recv() => {
                    tracing::info!("received SIGINT, shutting down");
                    let _ = shutdown_tx.send(true);
                    break;
                }
                _ = sighup.recv() => {
                    tracing::info!("received SIGHUP, forcing immediate tick");
                    scheduler.force_tick();
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
        tracing::info!("received Ctrl+C, shutting down");
        let _ = shutdown_tx.send(true);
    }

    Ok(())
}

fn parse_light_type(light: &heol::config::LightConfig) -> LightType {
    match light.light_type.as_str() {
        "mono" => LightType::Mono { temp: light.temp.unwrap_or(4500) },
        "dual" => LightType::Dual {
            cold_temp: light.cold_temp.unwrap_or(6500),
            warm_temp: light.warm_temp.unwrap_or(2700),
        },
        "rgb" => LightType::Rgb,
        "wrgb" => LightType::Wrgb { white_temp: light.white_temp.unwrap_or(4000) },
        _ => LightType::Mono { temp: 4500 },
    }
}

fn fill_command(
    command: heol::light::LightCommand,
    light: &heol::config::LightConfig,
) -> heol::light::LightCommand {
    use heol::light::LightCommand::*;
    match command {
        GpioPwm { duty, .. } => GpioPwm {
            pin: light.pin.unwrap_or(0),
            duty,
        },
        GpioDualPwm { cold_duty, warm_duty, .. } => GpioDualPwm {
            cold_pin: light.cold_pin.unwrap_or(0),
            warm_pin: light.warm_pin.unwrap_or(0),
            cold_duty,
            warm_duty,
        },
        DeconzState { on, bri, ct, .. } => DeconzState {
            light_id: light.light_id,
            group_id: light.group_id,
            on,
            bri,
            ct,
        },
        DeconzRgb { on, bri, xy, .. } => DeconzRgb {
            light_id: light.light_id,
            group_id: light.group_id,
            on,
            bri,
            xy,
        },
    }
}
