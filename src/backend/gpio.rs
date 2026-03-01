use super::LightBackend;
use crate::config::{GpioConnection, LightConfig};
use crate::light::LightCommand;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

const CMD_MODES: u32 = 0; // Set GPIO mode
const CMD_HP: u32 = 86; // Hardware PWM
const MODE_ALT5: u32 = 2; // ALT5 function (hardware PWM)

pub struct GpioBackend {
    connections: HashMap<String, Arc<Mutex<TcpStream>>>,
    initialized_pins: Arc<Mutex<std::collections::HashSet<(String, u8)>>>,
}

impl GpioBackend {
    pub async fn new(profiles: &HashMap<String, GpioConnection>) -> anyhow::Result<Self> {
        let mut connections = HashMap::new();
        for (name, conn) in profiles {
            let addr = format!("{}:{}", conn.host, conn.port);
            let stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| anyhow::anyhow!("gpio.{name}: failed to connect to {addr}: {e}"))?;
            connections.insert(name.clone(), Arc::new(Mutex::new(stream)));
        }
        Ok(Self {
            connections,
            initialized_pins: Arc::new(Mutex::new(std::collections::HashSet::new())),
        })
    }

    async fn ensure_pin_mode(
        &self,
        profile: &str,
        stream: &mut TcpStream,
        gpio: u8,
    ) -> anyhow::Result<()> {
        let key = (profile.to_string(), gpio);
        let mut pins = self.initialized_pins.lock().await;
        if pins.contains(&key) {
            return Ok(());
        }
        let request = encode_set_mode(gpio as u32, MODE_ALT5);
        stream.write_all(&request).await?;
        let mut resp = [0u8; 16];
        stream.read_exact(&mut resp).await?;
        decode_response(&resp)?;
        pins.insert(key);
        tracing::debug!(profile, gpio, "pin initialized to ALT5");
        Ok(())
    }

    async fn write_hp(
        stream: &mut TcpStream,
        gpio: u8,
        frequency: u32,
        duty: u32,
    ) -> anyhow::Result<()> {
        let request = encode_hardware_pwm(gpio as u32, frequency, duty);
        stream.write_all(&request).await?;
        let mut resp = [0u8; 16];
        stream.read_exact(&mut resp).await?;
        decode_response(&resp)
    }
}

#[async_trait]
impl LightBackend for GpioBackend {
    async fn send(&self, light: &LightConfig, command: LightCommand) -> anyhow::Result<()> {
        let profile = light
            .backend
            .split_once('.')
            .map(|(_, name)| name)
            .unwrap_or(&light.backend);

        let stream = self
            .connections
            .get(profile)
            .ok_or_else(|| anyhow::anyhow!("gpio profile '{}' not connected", profile))?;

        let frequency = light.pwm_frequency.unwrap_or(10_000);

        let mut stream = stream.lock().await;

        let invert = |duty: u32| -> u32 {
            if light.inverted {
                1_000_000 - duty
            } else {
                duty
            }
        };

        match command {
            LightCommand::GpioPwm { pin, duty } => {
                let actual_duty = invert(duty);
                tracing::debug!(light = %light.name, pin, duty, actual_duty, inverted = light.inverted, "gpio pwm");
                self.ensure_pin_mode(profile, &mut stream, pin).await?;
                Self::write_hp(&mut stream, pin, frequency, actual_duty).await?;
            }
            LightCommand::GpioDualPwm {
                cold_pin,
                warm_pin,
                cold_duty,
                warm_duty,
            } => {
                let actual_cold = invert(cold_duty);
                let actual_warm = invert(warm_duty);
                tracing::debug!(
                    light = %light.name,
                    cold_pin, warm_pin,
                    cold_duty, warm_duty,
                    actual_cold, actual_warm,
                    inverted = light.inverted,
                    "gpio dual pwm"
                );
                self.ensure_pin_mode(profile, &mut stream, cold_pin).await?;
                self.ensure_pin_mode(profile, &mut stream, warm_pin).await?;
                Self::write_hp(&mut stream, cold_pin, frequency, actual_cold).await?;
                Self::write_hp(&mut stream, warm_pin, frequency, actual_warm).await?;
            }
            _ => anyhow::bail!("gpio backend received non-gpio command"),
        }

        Ok(())
    }

    async fn healthcheck(&self) -> anyhow::Result<()> {
        for (name, stream) in &self.connections {
            let stream = stream.lock().await;
            let _ = stream
                .peer_addr()
                .map_err(|e| anyhow::anyhow!("gpio.{name}: connection check failed: {e}"))?;
        }
        Ok(())
    }
}

// --- Wire protocol functions (public for testing) ---

pub fn encode_set_mode(gpio: u32, mode: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    buf[0..4].copy_from_slice(&CMD_MODES.to_le_bytes());
    buf[4..8].copy_from_slice(&gpio.to_le_bytes());
    buf[8..12].copy_from_slice(&mode.to_le_bytes());
    buf[12..16].copy_from_slice(&0u32.to_le_bytes());
    buf
}

pub fn encode_hardware_pwm(gpio: u32, frequency: u32, dutycycle: u32) -> [u8; 20] {
    let mut buf = [0u8; 20];
    buf[0..4].copy_from_slice(&CMD_HP.to_le_bytes());
    buf[4..8].copy_from_slice(&gpio.to_le_bytes());
    buf[8..12].copy_from_slice(&frequency.to_le_bytes());
    buf[12..16].copy_from_slice(&4u32.to_le_bytes()); // extension length
    buf[16..20].copy_from_slice(&dutycycle.to_le_bytes());
    buf
}

pub fn decode_response(resp: &[u8; 16]) -> anyhow::Result<()> {
    let res = i32::from_le_bytes(resp[12..16].try_into().unwrap());
    if res < 0 {
        anyhow::bail!("pigpiod error code: {res}");
    }
    Ok(())
}
