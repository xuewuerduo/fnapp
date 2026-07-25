# fnapp — 飞牛 OS 应用仓库

飞牛 OS 第三方应用集合。

## 应用列表

| 应用 | 显示名 | 描述 | 架构 | 包地址 |
|------|--------|------|------|--------|
| fn-wol | WOL 远程唤醒 | 轻量级局域网远程唤醒工具 | amd64 / arm64 | [下载最新](https://github.com/xuewuerduo/fnapp/releases/latest) |

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

每个应用独立子目录，独立 CI 工作流。

### 版本管理

版本号由 git tag 驱动，CI 自动完成构建与发布：

1. 修改代码后推送至 `master`
2. 打 tag 触发 CI 构建：`git tag {app}-v{version}`（如 `fn-wol-v0.1.4`）
3. CI 自动编译、打包（.fpk）、创建 GitHub Release 并上传产物

Tag 格式：`{app}-v{major}.{minor}.{patch}`

触发方式：

- **Tag 触发**: 推送 `{app}-v*` 格式 tag
- **路径触发**: 推送 `fnapp/{app}/**` 下文件变更（仅构建验证，不发布）

> 版本号只需改 tag，无需手动修改任何配置文件。
