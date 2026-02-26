#[cfg(feature = "deconz")]
mod tests {
    use heol::backend::deconz::{light_state_payload, rgb_state_payload};

    #[test]
    fn state_payload_on_with_ct() {
        let payload = light_state_payload(true, 200, Some(250));
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["on"], true);
        assert_eq!(json["bri"], 200);
        assert_eq!(json["ct"], 250);
    }

    #[test]
    fn state_payload_off() {
        let payload = light_state_payload(false, 0, None);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["on"], false);
        assert_eq!(json["bri"], 0);
        assert!(json.get("ct").is_none());
    }

    #[test]
    fn rgb_payload_with_xy() {
        let payload = rgb_state_payload(true, 200, (0.3127, 0.3290));
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["on"], true);
        assert_eq!(json["bri"], 200);
        let xy = json["xy"].as_array().unwrap();
        assert!((xy[0].as_f64().unwrap() - 0.3127).abs() < 0.001);
        assert!((xy[1].as_f64().unwrap() - 0.3290).abs() < 0.001);
    }
}
