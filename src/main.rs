mod mac;
mod wol;
mod device;
mod network;
mod scanner;
mod server;

use std::path::Path;
use std::sync::{Arc, Mutex};
use device::DeviceStore;

fn main() {
    let port: u16 = std::env::var("TRIM_SERVICE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10101);

    let data_dir = std::env::var("DATA_DIR")
        .ok()
        .or_else(|| std::env::var("TRIM_PKGVAR").ok())
        .unwrap_or_else(|| ".".to_string());

    let data_path = Path::new(&data_dir).join("wol_devices.json");
    let data_path_str = data_path.to_str().unwrap_or("wol_devices.json");

    let store = Arc::new(Mutex::new(DeviceStore::new(data_path_str)));

    eprintln!("飞牛 WOL 远程唤醒工具");
    eprintln!("服务地址: http://0.0.0.0:{}", port);
    eprintln!("数据文件: {}", data_path_str);

    server::run(port, store);
}
