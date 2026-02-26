#[cfg(feature = "gpio")]
mod tests {
    use heol::backend::gpio::{encode_hardware_pwm, decode_response};

    #[test]
    fn encode_hp_command() {
        let bytes = encode_hardware_pwm(17, 10000, 500_000);
        // cmd = 86 (HP)
        assert_eq!(u32::from_le_bytes(bytes[0..4].try_into().unwrap()), 86);
        // p1 = gpio 17
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 17);
        // p2 = frequency 10000
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 10000);
        // p3 = 4 (extension length)
        assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 4);
        // extension = dutycycle 500000
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 500_000);
    }

    #[test]
    fn decode_success_response() {
        let mut resp = [0u8; 16];
        // res field at offset 12 = 0 (success)
        resp[12..16].copy_from_slice(&0i32.to_le_bytes());
        let result = decode_response(&resp);
        assert!(result.is_ok());
    }

    #[test]
    fn decode_error_response() {
        let mut resp = [0u8; 16];
        // res field at offset 12 = -3 (error)
        resp[12..16].copy_from_slice(&(-3i32).to_le_bytes());
        let result = decode_response(&resp);
        assert!(result.is_err());
    }
}
