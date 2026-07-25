# fnapp — 飞牛 OS 应用仓库

飞牛 OS 第三方应用集合。

## 应用列表

| 应用 | 显示名 | 描述 | 最新版本 | 架构 | 包地址 |
|------|--------|------|----------|------|--------|
| fn-wol | WOL 远程唤醒 | 轻量级局域网远程唤醒工具 | v0.1.1 | amd64 / arm64 | [下载](https://github.com/xuewuerduo/fnapp/releases/tag/fn-wol-v0.1.1) |

> 完整信息见 [apps.json](apps.json)

## 目录结构

```
fnapp/
  fn-wol/       ← WOL 远程唤醒应用
  app-xxx/      ← 其他应用
.github/workflows/
  build-fn-wol.yml  ← fn-wol 独立 CI
```

## 开发指南

每个应用独立子目录，独立 CI 工作流。触发方式：

- **Tag 触发**: 推送 `{app}-v*` 格式 tag（如 `fn-wol-v0.1.1`）
- **路径触发**: 推送 `fnapp/{app}/**` 下文件变更

仅构建变更的应用，不影响其他应用。
