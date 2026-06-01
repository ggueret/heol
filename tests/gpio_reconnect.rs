#[cfg(feature = "gpio")]
mod tests {
    use heol::backend::{LightBackend, gpio::GpioBackend};
    use heol::config::{GpioConnection, LightConfig};
    use heol::light::LightCommand;
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::oneshot;

    const SUCCESS_RESPONSE: [u8; 16] = [0u8; 16];
    const CMD_MODES: u32 = 0; // set GPIO mode

    /// Serve one pigpiod request and return the command code from its header.
    async fn serve_one_request(sock: &mut TcpStream) -> std::io::Result<u32> {
        let mut header = [0u8; 16];
        sock.read_exact(&mut header).await?;
        let cmd = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let ext_len = u32::from_le_bytes(header[12..16].try_into().unwrap());
        if ext_len > 0 {
            let mut ext = vec![0u8; ext_len as usize];
            sock.read_exact(&mut ext).await?;
        }
        sock.write_all(&SUCCESS_RESPONSE).await?;
        Ok(cmd)
    }

    fn dummy_light(profile: &str, pin: u8) -> LightConfig {
        LightConfig {
            name: "test".to_string(),
            light_type: "single".to_string(),
            backend: format!("gpio.{profile}"),
            temp: Some(4500),
            cold_temp: None,
            warm_temp: None,
            white_temp: None,
            pin: Some(pin),
            cold_pin: None,
            warm_pin: None,
            pwm_frequency: Some(1000),
            inverted: false,
            light_id: None,
            group_id: None,
        }
    }

    #[tokio::test]
    async fn reconnects_after_broken_pipe() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Fake pigpiod: serve the first send() (set_mode + hardware_pwm = 2 requests),
        // then drop the connection. Accept a second connection and serve it indefinitely.
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                serve_one_request(&mut sock).await.unwrap();
            }
            drop(sock);

            let (mut sock, _) = listener.accept().await.unwrap();
            while serve_one_request(&mut sock).await.is_ok() {}
        });

        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            GpioConnection {
                host: "127.0.0.1".to_string(),
                port,
            },
        );
        let backend = GpioBackend::new(&profiles).await.unwrap();

        let light = dummy_light("test", 18);

        // First send uses the initial connection.
        backend
            .send(
                &light,
                LightCommand::GpioPwm {
                    pin: 18,
                    duty: 500_000,
                },
            )
            .await
            .expect("first send should succeed");

        // Second send: the server has dropped the connection. The backend must
        // detect the broken pipe, reconnect, and retry successfully.
        backend
            .send(
                &light,
                LightCommand::GpioPwm {
                    pin: 18,
                    duty: 600_000,
                },
            )
            .await
            .expect("second send should reconnect and succeed");

        server.abort();
    }

    #[tokio::test]
    async fn reinitializes_pin_mode_after_reconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Fake pigpiod: serve the first send (set_mode + hardware_pwm), drop the
        // connection, then report the first command seen on the reconnection.
        let (tx, rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                serve_one_request(&mut sock).await.unwrap();
            }
            drop(sock);

            let (mut sock, _) = listener.accept().await.unwrap();
            let first_cmd = serve_one_request(&mut sock).await.unwrap();
            tx.send(first_cmd).unwrap();
            while serve_one_request(&mut sock).await.is_ok() {}
        });

        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            GpioConnection {
                host: "127.0.0.1".to_string(),
                port,
            },
        );
        let backend = GpioBackend::new(&profiles).await.unwrap();

        let light = dummy_light("test", 18);

        backend
            .send(
                &light,
                LightCommand::GpioPwm {
                    pin: 18,
                    duty: 500_000,
                },
            )
            .await
            .expect("first send should succeed");

        // The connection has dropped (e.g. pigpiod restarted): the new process
        // lost the pin mode, so the reconnecting send must re-issue set_mode
        // before hardware_pwm, not just resend the PWM command.
        backend
            .send(
                &light,
                LightCommand::GpioPwm {
                    pin: 18,
                    duty: 600_000,
                },
            )
            .await
            .expect("second send should reconnect and succeed");

        let first_cmd = rx.await.unwrap();
        assert_eq!(
            first_cmd, CMD_MODES,
            "after reconnect the pin must be re-initialized (set_mode) before hardware_pwm, got {first_cmd}"
        );

        server.abort();
    }
}
