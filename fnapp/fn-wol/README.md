# 飞牛 WOL 远程唤醒工具

轻量级局域网远程唤醒应用，专为飞牛 OS 设计。

## 功能特性

- **一键扫描局域网设备** — 自动获取网段，扫描在线设备 IP + MAC
- **手动添加设备** — 适合关机但支持 WOL 的设备，MAC 格式自动兼容
- **一键唤醒** — 发送魔术包远程开机
- **设备列表管理** — 备注编辑、删除，MAC 自动去重
- **数据持久化** — JSON 存储，重启不丢失
- **极致轻量** — 静态编译约 1.5MB，零系统依赖，长期稳定运行
- **跨平台** — 支持 ARM64 / x86_64 Linux

## 使用方法

1. 下载对应架构的二进制文件
2. 上传到飞牛 OS（或任何 Linux 设备）
3. 运行：
   ```bash
   chmod +x fn-wol
   ./fn-wol
   ```
4. 浏览器打开 `http://飞牛IP:10101`

## 操作流程

1. 点击 **扫描设备** → 自动列出在线设备
2. 点击设备名称 → 填写备注（电脑、NAS、笔记本等）
3. 点击 **唤醒** → 远程开机
4. 不需要的设备可删除

## 手动添加设备

适合不在线但支持 WOL 的设备：
- 输入 MAC 地址（支持 `aa:bb:cc:dd:ee:ff`、`aa-bb-cc-dd-ee-ff`、`aabbccddeeff` 格式）
- 填写备注名称
- IP 地址可选

## 数据备份

设备列表保存在程序同目录的 `wol_devices.json`，可直接复制备份。

## 自行编译

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 musl target
rustup target add aarch64-unknown-linux-musl
rustup target add x86_64-unknown-linux-musl

# 编译（ARM64）
cargo build --release --target aarch64-unknown-linux-musl

# 编译（x86_64）
cargo build --release --target x86_64-unknown-linux-musl
```

二进制文件位于 `target/<架构>/release/fn-wol`。

## 技术参数

| 项目 | 参数 |
|------|------|
| 端口 | 10101（固定） |
| 架构 | ARM64 / x86_64 |
| 编译 | musl 静态链接 |
| 体积 | ~1.5MB |
| 依赖 | 零系统依赖 |
| 运行权限 | 普通 用户（无需 root） |

## 在飞牛 APP 中添加

端口固定 `10101`，可直接在飞牛 APP 中添加为自定义应用。
