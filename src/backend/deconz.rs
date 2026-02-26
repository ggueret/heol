use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use crate::config::{DeconzConnection, LightConfig};
use crate::light::LightCommand;
use super::LightBackend;

pub struct DeconzClient {
    client: Client,
    base_url: String,
}

impl DeconzClient {
    pub fn new(conn: &DeconzConnection) -> Self {
        let base_url = format!("http://{}:{}/api/{}", conn.host, conn.port, conn.api_key);
        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn put_state(&self, path: &str, body: &str) -> anyhow::Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .put(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("deCONZ PUT {path} returned {status}: {body}");
        }

        Ok(())
    }
}

pub struct DeconzBackend {
    clients: HashMap<String, Arc<DeconzClient>>,
}

impl DeconzBackend {
    pub fn new(profiles: &HashMap<String, DeconzConnection>) -> Self {
        let clients = profiles
            .iter()
            .map(|(name, conn)| (name.clone(), Arc::new(DeconzClient::new(conn))))
            .collect();
        Self { clients }
    }
}

#[async_trait]
impl LightBackend for DeconzBackend {
    async fn send(&self, light: &LightConfig, command: LightCommand) -> anyhow::Result<()> {
        let profile = light.backend.split_once('.')
            .map(|(_, name)| name)
            .unwrap_or(&light.backend);

        let client = self.clients.get(profile)
            .ok_or_else(|| anyhow::anyhow!("deconz profile '{}' not found", profile))?;

        match command {
            LightCommand::DeconzState { light_id, group_id, on, bri, ct } => {
                let payload = light_state_payload(on, bri, ct);
                let path = if let Some(id) = light_id {
                    format!("/lights/{id}/state")
                } else if let Some(id) = group_id {
                    format!("/groups/{id}/action")
                } else {
                    anyhow::bail!("deconz command missing both light_id and group_id");
                };
                client.put_state(&path, &payload).await?;
            }
            LightCommand::DeconzRgb { light_id, group_id, on, bri, xy } => {
                let payload = rgb_state_payload(on, bri, xy);
                let path = if let Some(id) = light_id.or(group_id) {
                    if light_id.is_some() {
                        format!("/lights/{id}/state")
                    } else {
                        format!("/groups/{id}/action")
                    }
                } else {
                    anyhow::bail!("deconz command missing both light_id and group_id");
                };
                client.put_state(&path, &payload).await?;
            }
            _ => anyhow::bail!("deconz backend received non-deconz command"),
        }

        Ok(())
    }

    async fn healthcheck(&self) -> anyhow::Result<()> {
        for (name, client) in &self.clients {
            let url = format!("{}/config", client.base_url);
            client.client.get(&url).send().await
                .map_err(|e| anyhow::anyhow!("deconz.{name}: healthcheck failed: {e}"))?;
        }
        Ok(())
    }
}

// --- Payload builders (public for testing) ---

pub fn light_state_payload(on: bool, bri: u8, ct: Option<u16>) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("on".to_string(), serde_json::Value::Bool(on));
    obj.insert("bri".to_string(), serde_json::Value::Number(bri.into()));
    if let Some(ct) = ct {
        obj.insert("ct".to_string(), serde_json::Value::Number(ct.into()));
    }
    serde_json::Value::Object(obj).to_string()
}

pub fn rgb_state_payload(on: bool, bri: u8, xy: (f64, f64)) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("on".to_string(), serde_json::Value::Bool(on));
    obj.insert("bri".to_string(), serde_json::Value::Number(bri.into()));
    obj.insert("xy".to_string(), serde_json::json!([xy.0, xy.1]));
    serde_json::Value::Object(obj).to_string()
}
