use chrono::Utc;
use std::time::Duration;
use tokio::sync::watch;
use crate::solar::{SolarEngine, SolarState};

pub struct Scheduler {
    engine: SolarEngine,
    tx: watch::Sender<Option<SolarState>>,
    interval: Duration,
}

impl Scheduler {
    pub fn new(
        engine: SolarEngine,
        tx: watch::Sender<Option<SolarState>>,
        interval: Duration,
    ) -> Self {
        Self { engine, tx, interval }
    }

    /// Run a single tick: compute solar position and broadcast it.
    pub async fn run_once(&self) {
        let now = Utc::now();
        let state = self.engine.position(now);
        let _ = self.tx.send(Some(state));
    }

    /// Main loop: tick at the configured interval.
    /// Returns when the shutdown signal is received.
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.run_once().await;
                }
                _ = shutdown.changed() => {
                    tracing::info!("scheduler shutting down");
                    break;
                }
            }
        }
    }

    /// Force an immediate tick (used by SIGHUP handler).
    pub fn force_tick(&self) {
        let now = Utc::now();
        let state = self.engine.position(now);
        let _ = self.tx.send(Some(state));
    }
}
