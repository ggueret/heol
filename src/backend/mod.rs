#[cfg(feature = "deconz")]
pub mod deconz;
#[cfg(feature = "gpio")]
pub mod gpio;

use crate::config::LightConfig;
use crate::light::LightCommand;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait LightBackend: Send + Sync {
    async fn send(&self, light: &LightConfig, command: LightCommand) -> anyhow::Result<()>;
    async fn healthcheck(&self) -> anyhow::Result<()>;
}

pub struct DryRunBackend {
    inner: Arc<dyn LightBackend>,
}

impl DryRunBackend {
    pub fn new(inner: Arc<dyn LightBackend>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl LightBackend for DryRunBackend {
    async fn send(&self, light: &LightConfig, command: LightCommand) -> anyhow::Result<()> {
        tracing::info!(
            light = %light.name,
            command = ?command,
            "[dry-run] would send command"
        );
        Ok(())
    }

    async fn healthcheck(&self) -> anyhow::Result<()> {
        self.inner.healthcheck().await
    }
}
