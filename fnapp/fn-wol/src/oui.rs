use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;

static DB: OnceLock<HashMap<String, String>> = OnceLock::new();
static ONLINE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

fn online_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    ONLINE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn extract_prefix(mac: &str) -> Option<String> {
    let key: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase();
    if key.len() < 6 {
        return None;
    }
    Some(key[..6].to_string())
}

/// 初始化 OUI 数据库（加载 CSV 文件），不传路径则跳过文件加载
/// 格式 1: PREFIX,VENDOR（精简格式）
/// 格式 2: Registry,Assignment,Organization Name,...（IEEE 原始格式）
pub fn init(path: Option<&str>) {
    let db = match path {
        Some(p) if Path::new(p).exists() => {
            let content = std::fs::read_to_string(p).unwrap_or_default();
            let mut map = HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("Registry") {
                    continue;
                }
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() < 2 {
                    continue;
                }
                let (prefix, vendor) = if fields.len() >= 3 {
                    (fields[1].trim(), fields[2].trim().trim_matches('"'))
                } else {
                    (fields[0].trim(), fields[1].trim())
                };
                if prefix.len() != 6 || !prefix.chars().all(|c| c.is_ascii_hexdigit()) {
                    continue;
                }
                if !vendor.is_empty() {
                    map.insert(prefix.to_string(), vendor.to_string());
                }
            }
            map
        }
        _ => HashMap::new(),
    };
    let _ = DB.set(db);
}

pub fn lookup_vendor(mac: &str) -> Option<String> {
    let prefix = extract_prefix(mac)?;

    if let Some(db) = DB.get() {
        if let Some(vendor) = db.get(&prefix) {
            return Some(vendor.clone());
        }
    }

    {
        let oc = online_cache().lock().unwrap();
        if let Some(result) = oc.get(&prefix) {
            return result.clone();
        }
    }

    let result = query_online(&prefix);
    {
        let mut oc = online_cache().lock().unwrap();
        oc.insert(prefix, result.clone());
    }
    result
}

fn query_online(prefix: &str) -> Option<String> {
    let url = format!("https://api.macvendors.com/{}", prefix);
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("--max-time")
        .arg("3")
        .arg(&url)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let vendor = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if vendor.is_empty() || vendor.contains("Not Found") || vendor.contains("rate limit") {
        None
    } else {
        Some(vendor)
    }
}
