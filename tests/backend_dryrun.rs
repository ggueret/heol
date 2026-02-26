use heol::backend::{DryRunBackend, LightBackend};
use heol::config::LightConfig;
use heol::light::LightCommand;

struct MockBackend {
    calls: std::sync::Mutex<Vec<String>>,
}

impl MockBackend {
    fn new() -> Self {
        Self { calls: std::sync::Mutex::new(Vec::new()) }
    }
}

#[async_trait::async_trait]
impl LightBackend for MockBackend {
    async fn send(&self, light: &LightConfig, _command: LightCommand) -> anyhow::Result<()> {
        self.calls.lock().unwrap().push(light.name.clone());
        Ok(())
    }

    async fn healthcheck(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn dryrun_does_not_call_inner_send() {
    let mock = std::sync::Arc::new(MockBackend::new());
    let dryrun = DryRunBackend::new(mock.clone());

    let light = dummy_light();
    let cmd = LightCommand::GpioPwm { pin: 17, duty: 500000 };
    dryrun.send(&light, cmd).await.unwrap();

    assert!(mock.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn dryrun_delegates_healthcheck() {
    let mock = std::sync::Arc::new(MockBackend::new());
    let dryrun = DryRunBackend::new(mock);
    assert!(dryrun.healthcheck().await.is_ok());
}

fn dummy_light() -> LightConfig {
    LightConfig {
        name: "test".to_string(),
        light_type: "mono".to_string(),
        backend: "gpio.local".to_string(),
        temp: Some(4500),
        cold_temp: None,
        warm_temp: None,
        white_temp: None,
        pin: Some(17),
        cold_pin: None,
        warm_pin: None,
        pwm_frequency: None,
        light_id: None,
        group_id: None,
    }
}
