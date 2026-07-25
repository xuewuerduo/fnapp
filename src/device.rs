use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub mac: String,
    pub ip: Option<String>,
    pub name: String,
    pub last_seen: Option<String>,
}

pub struct DeviceStore {
    devices: Vec<Device>,
    file_path: String,
}

impl DeviceStore {
    pub fn new(file_path: &str) -> Self {
        let devices = Self::load(file_path);
        Self {
            devices,
            file_path: file_path.to_string(),
        }
    }

    fn load(path: &str) -> Vec<Device> {
        if !Path::new(path).exists() {
            return Vec::new();
        }
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.devices)
            .map_err(|e| e.to_string())?;
        fs::write(&self.file_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list(&self) -> &[Device] {
        &self.devices
    }

    /// 添加设备（基于 MAC 去重），成功返回 true
    pub fn add(&mut self, device: Device) -> bool {
        if self.devices.iter().any(|d| d.mac == device.mac) {
            return false;
        }
        self.devices.push(device);
        true
    }

    /// 更新或添加设备（扫描时使用），MAC 已存在则更新 IP/last_seen
    pub fn upsert(&mut self, device: Device) {
        if let Some(existing) = self.devices.iter_mut().find(|d| d.mac == device.mac) {
            existing.ip = device.ip;
            existing.last_seen = device.last_seen;
        } else {
            self.devices.push(device);
        }
    }

    pub fn update_name(&mut self, mac: &str, name: &str) -> bool {
        if let Some(device) = self.devices.iter_mut().find(|d| d.mac == mac) {
            device.name = name.to_string();
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, mac: &str) -> bool {
        let before = self.devices.len();
        self.devices.retain(|d| d.mac != mac);
        self.devices.len() < before
    }
}
