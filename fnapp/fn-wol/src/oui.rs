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
/// 支持 CSV 格式 1: PREFIX,VENDOR（精简格式）
/// 支持 CSV 格式 2: Registry,Assignment,Organization Name,...（IEEE 原始格式）
pub fn init(path: Option<&str>) {
    let db = match path {
        Some(p) if Path::new(p).exists() => {
            let content = std::fs::read_to_string(p).unwrap_or_default();
            let mut map = HashMap::new();
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .from_reader(content.as_bytes());
            for result in rdr.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                if record.len() < 2 {
                    continue;
                }
                let prefix = if record.len() >= 3 {
                    // IEEE 格式: Registry,Assignment,OrgName,...
                    record.get(1).unwrap_or("")
                } else {
                    // 精简格式: PREFIX,VENDOR
                    record.get(0).unwrap_or("")
                };
                let vendor = if record.len() >= 3 {
                    record.get(2).unwrap_or("")
                } else {
                    record.get(1).unwrap_or("")
                };
                let prefix = prefix.trim();
                let vendor = vendor.trim();
                if prefix.is_empty() || vendor.is_empty() {
                    continue;
                }
                if prefix.len() != 6 || !prefix.chars().all(|c| c.is_ascii_hexdigit()) {
                    continue;
                }
                if vendor == "Organization Name" || record.get(0).unwrap_or("") == "Registry" {
                    continue;
                }
                map.insert(prefix.to_string(), vendor.to_string());
            }
            map
        }
        _ => HashMap::new(),
    };
    let _ = DB.set(db);
}

/// 仅查本地 OUI 库（毫秒级），用于扫描等需要快速返回的场景
pub fn lookup_vendor_local(mac: &str) -> Option<String> {
    let prefix = extract_prefix(mac)?;
    if let Some(db) = DB.get() {
        return db.get(&prefix).cloned();
    }
    None
}

/// 查本地库 + 联网兜底（可能耗时 3s+），用于异步补充查询
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
    if vendor.is_empty()
        || vendor.contains("Not Found")
        || vendor.contains("rate limit")
        || vendor.starts_with('{')
        || vendor.starts_with('[')
    {
        None
    } else {
        Some(vendor)
    }
}
