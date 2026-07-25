/// 解析 MAC 地址字符串为 6 字节数组
/// 兼容格式: aa:bb:cc:dd:ee:ff / AA-BB-CC-DD-EE-FF / aabbccddeeff
pub fn parse_mac(input: &str) -> Option<[u8; 6]> {
    let cleaned: String = input.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if cleaned.len() != 12 {
        return None;
    }
    let mut bytes = [0u8; 6];
    for i in 0..6 {
        bytes[i] = u8::from_str_radix(&cleaned[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

/// 格式化 MAC 字节为小写冒号分隔格式: aa:bb:cc:dd:ee:ff
pub fn format_mac(bytes: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

/// 解析并标准化 MAC 地址，返回统一格式
pub fn normalize_mac(input: &str) -> Option<String> {
    parse_mac(input).map(|bytes| format_mac(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_colon() {
        assert_eq!(parse_mac("aa:bb:cc:dd:ee:ff"), Some([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]));
    }

    #[test]
    fn test_parse_dash() {
        assert_eq!(parse_mac("AA-BB-CC-DD-EE-FF"), Some([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]));
    }

    #[test]
    fn test_parse_plain() {
        assert_eq!(parse_mac("aabbccddeeff"), Some([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_mac("invalid"), None);
        assert_eq!(parse_mac("aa:bb:cc"), None);
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize_mac("AA-BB-CC-DD-EE-FF"), Some("aa:bb:cc:dd:ee:ff".to_string()));
        assert_eq!(normalize_mac("aabbccddeeff"), Some("aa:bb:cc:dd:ee:ff".to_string()));
    }
}
