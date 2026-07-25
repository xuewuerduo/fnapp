use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

use crate::device::{Device, DeviceStore};
use crate::mac;
use crate::scanner;
use crate::wol;

pub fn run(port: u16, store: Arc<Mutex<DeviceStore>>) {
    let addr = format!("0.0.0.0:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法启动服务器: {}", e);
            std::process::exit(1);
        }
    };
    eprintln!("WOL 服务已启动: http://0.0.0.0:{}", port);

    for request in server.incoming_requests() {
        handle_request(request, &store);
    }
}

fn make_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn json_response(status: u16, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(make_header("Content-Type", "application/json; charset=utf-8"))
        .with_header(make_header("Access-Control-Allow-Origin", "*"))
}

fn static_response(content_type: &str, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_header(make_header("Content-Type", content_type))
}

fn url_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| s.to_string())
}

use std::time::{SystemTime, UNIX_EPOCH};

fn current_time() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

fn handle_request(mut request: tiny_http::Request, store: &Arc<Mutex<DeviceStore>>) {
    let url = request.url().to_string();
    let method = request.method().as_str().to_string();
    let path = url.split('?').next().unwrap_or("/").to_string();

    if method == "OPTIONS" {
        let _ = request.respond(
            Response::empty(204)
                .with_header(make_header("Access-Control-Allow-Origin", "*"))
                .with_header(make_header(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                ))
                .with_header(make_header("Access-Control-Allow-Headers", "Content-Type")),
        );
        return;
    }

    match (method.as_str(), path.as_str()) {
        ("GET", "/") => {
            let html = include_str!("../static/index.html");
            let _ = request.respond(static_response("text/html; charset=utf-8", html));
        }
        ("GET", "/style.css") => {
            let css = include_str!("../static/style.css");
            let _ = request.respond(static_response("text/css; charset=utf-8", css));
        }
        ("GET", "/app.js") => {
            let js = include_str!("../static/app.js");
            let _ = request.respond(static_response(
                "application/javascript; charset=utf-8",
                js,
            ));
        }

        ("GET", "/api/devices") => {
            let store = store.lock().unwrap();
            let json = serde_json::to_string(store.list()).unwrap_or_else(|_| "[]".to_string());
            let _ = request.respond(json_response(200, &json));
        }

        ("POST", "/api/devices") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);

            #[derive(serde::Deserialize)]
            struct AddRequest {
                mac: String,
                name: String,
                ip: Option<String>,
            }

            match serde_json::from_str::<AddRequest>(&body) {
                Ok(req) => match mac::normalize_mac(&req.mac) {
                    Some(normalized) => {
                        let vendor = crate::oui::lookup_vendor(&normalized).map(|s| s.to_string());
                        let device = Device {
                            mac: normalized,
                            ip: req.ip.filter(|s| !s.is_empty()),
                            name: req.name,
                            vendor,
                            last_seen: None,
                        };
                        let mut store = store.lock().unwrap();
                        if store.add(device) {
                            let _ = store.save();
                            let _ = request.respond(json_response(200, r#"{"ok":true}"#));
                        } else {
                            let _ = request.respond(json_response(409, r#"{"error":"设备已存在"}"#));
                        }
                    }
                    None => {
                        let _ = request.respond(json_response(400, r#"{"error":"MAC 地址格式无效"}"#));
                    }
                },
                Err(_) => {
                    let _ = request.respond(json_response(400, r#"{"error":"请求格式错误"}"#));
                }
            }
        }

        ("POST", "/api/scan") => match scanner::scan_network() {
            Ok(results) => {
                let now = current_time();
                let mut store = store.lock().unwrap();

                let devices_json: Vec<_> = results.iter().map(|r| {
                    let exists = store.exists(&r.mac);
                    // 已收藏的设备更新 IP 和在线时间
                    if exists {
                        store.update_existing(&r.mac, &r.ip, &now);
                    }
                    serde_json::json!({
                        "ip": r.ip,
                        "mac": r.mac,
                        "vendor": r.vendor,
                        "exists": exists
                    })
                }).collect();
                let _ = store.save();

                let json = serde_json::json!({
                    "ok": true,
                    "count": results.len(),
                    "devices": devices_json
                })
                .to_string();

                let _ = request.respond(json_response(200, &json));
            }
            Err(e) => {
                let _ = request.respond(json_response(500, &format!(r#"{{"error":"{}"}}"#, e)));
            }
        },

        ("GET", _) if path.starts_with("/api/vendor") => {
            let mac = url.split('?').nth(1).unwrap_or("")
                .split('&').find_map(|p| p.strip_prefix("mac=")).unwrap_or("");
            let vendor = crate::oui::lookup_vendor(&mac);
            let json = serde_json::json!({ "vendor": vendor }).to_string();
            let _ = request.respond(json_response(200, &json));
        }

        _ if path.starts_with("/api/devices/") => {
            let suffix = &path["/api/devices/".len()..];
            let parts: Vec<&str> = suffix.split('/').collect();
            let mac_param = url_decode(parts[0]);

            match (method.as_str(), parts.get(1).map(|s| *s)) {
                ("DELETE", None) => {
                    let mut store = store.lock().unwrap();
                    if store.remove(&mac_param) {
                        let _ = store.save();
                        let _ = request.respond(json_response(200, r#"{"ok":true}"#));
                    } else {
                        let _ = request.respond(json_response(404, r#"{"error":"设备不存在"}"#));
                    }
                }
                ("PUT", None) => {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);

                    #[derive(serde::Deserialize)]
                    struct UpdateRequest {
                        name: String,
                    }

                    match serde_json::from_str::<UpdateRequest>(&body) {
                        Ok(req) => {
                            let mut store = store.lock().unwrap();
                            if store.update_name(&mac_param, &req.name) {
                                let _ = store.save();
                                let _ = request.respond(json_response(200, r#"{"ok":true}"#));
                            } else {
                                let _ = request.respond(json_response(404, r#"{"error":"设备不存在"}"#));
                            }
                        }
                        Err(_) => {
                            let _ = request.respond(json_response(400, r#"{"error":"请求格式错误"}"#));
                        }
                    }
                }
                ("POST", Some("wake")) => match mac::parse_mac(&mac_param) {
                    Some(mac_bytes) => match wol::send_wol(&mac_bytes) {
                        Ok(()) => {
                            let _ = request.respond(json_response(200, r#"{"ok":true}"#));
                        }
                        Err(e) => {
                            let _ = request.respond(json_response(500, &format!(r#"{{"error":"{}"}}"#, e)));
                        }
                    },
                    None => {
                        let _ = request.respond(json_response(400, r#"{"error":"MAC 地址格式无效"}"#));
                    }
                },
                _ => {
                    let _ = request.respond(json_response(404, r#"{"error":"not found"}"#));
                }
            }
        }

        _ => {
            let _ = request.respond(json_response(404, r#"{"error":"not found"}"#));
        }
    }
}
