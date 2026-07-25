use std::collections::HashMap;
use std::sync::OnceLock;

fn vendor_db() -> &'static HashMap<&'static str, &'static str> {
    static DB: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    DB.get_or_init(|| {
        let data = include_str!("../data/oui.csv");
        let mut map = HashMap::new();
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((prefix, vendor)) = line.split_once(',') {
                map.insert(prefix, vendor);
            }
        }
        map
    })
}

pub fn lookup_vendor(mac: &str) -> Option<&'static str> {
    let prefix = &mac[..8].to_uppercase();
    let compressed: String = prefix.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if compressed.len() < 6 {
        return None;
    }
    let key = &compressed[..6];
    vendor_db().get(key).copied()
}
