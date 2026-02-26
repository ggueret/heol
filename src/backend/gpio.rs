use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use async_trait::async_trait;
use crate::config::{GpioConnection, LightConfig};
use crate::light::LightCommand;
use super::LightBackend;

const CMD_HP: u32 = 86; // Hardware PWM

pub struct GpioBackend {
    connections: HashMap<String, Arc<Mutex<TcpStream>>>,
}

impl GpioBackend {
    pub async fn new(profiles: &HashMap<String, GpioConnection>) -> anyhow::Result<Self> {
        let mut connections = HashMap::new();
        for (name, conn) in profiles {
            let addr = format!("{}:{}", conn.host, conn.port);
            let stream = TcpStream::connect(&addr).await
                .map_err(|e| anyhow::anyhow!("gpio.{name}: failed to connect to {addr}: {e}"))?;
            connections.insert(name.clone(), Arc::new(Mutex::new(stream)));
        }
        Ok(Self { connections })
    }

    async fn send_hp(
        stream: &Mutex<TcpStream>,
        gpio: u8,
        frequency: u32,
        duty: u32,
    ) -> anyhow::Result<()> {
        let request = encode_hardware_pwm(gpio as u32, frequency, duty);
        let mut stream = stream.lock().await;
        stream.write_all(&request).await?;

        let mut resp = [0u8; 16];
        stream.read_exact(&mut resp).await?;
        decode_response(&resp)
    }
}

#[async_trait]
impl LightBackend for GpioBackend {
    async fn send(&self, light: &LightConfig, command: LightCommand) -> anyhow::Result<()> {
        let profile = light.backend.split_once('.')
            .map(|(_, name)| name)
            .unwrap_or(&light.backend);

        let stream = self.connections.get(profile)
            .ok_or_else(|| anyhow::anyhow!("gpio profile '{}' not connected", profile))?;

        let frequency = light.pwm_frequency.unwrap_or(10_000);

        match command {
            LightCommand::GpioPwm { pin, duty } => {
                Self::send_hp(stream, pin, frequency, duty).await?;
            }
            LightCommand::GpioDualPwm { cold_pin, warm_pin, cold_duty, warm_duty } => {
                Self::send_hp(stream, cold_pin, frequency, cold_duty).await?;
                Self::send_hp(stream, warm_pin, frequency, warm_duty).await?;
            }
            _ => anyhow::bail!("gpio backend received non-gpio command"),
        }

        Ok(())
    }

    async fn healthcheck(&self) -> anyhow::Result<()> {
        for (name, stream) in &self.connections {
            let stream = stream.lock().await;
            let _ = stream.peer_addr()
                .map_err(|e| anyhow::anyhow!("gpio.{name}: connection check failed: {e}"))?;
        }
        Ok(())
    }
}

// --- Wire protocol functions (public for testing) ---

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
