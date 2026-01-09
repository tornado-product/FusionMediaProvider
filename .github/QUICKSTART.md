# CI/CD 快速开始指南

## 🚀 5 分钟设置

### 1. 配置 crates.io Token

1. 访问 [crates.io](https://crates.io) 并登录
2. 进入 Account Settings → API Tokens
3. 点击 "New Token"
4. 输入 token 名称（如 "GitHub Actions"）
5. 复制生成的 token

### 2. 添加 GitHub Secret

1. 进入你的 GitHub 仓库
2. 点击 Settings → Secrets and variables → Actions
3. 点击 "New repository secret"
4. 名称: `CRATES_IO_TOKEN`
5. 值: 粘贴刚才复制的 token
6. 点击 "Add secret"

### 3. 测试 CI

推送代码到仓库，CI 会自动运行：

```bash
git add .
git commit -m "test: CI workflow"
git push
```

在 GitHub 仓库的 Actions 标签页查看运行状态。

### 4. 发布第一个版本

```bash
# 1. 更新版本号（在 Cargo.toml 中）
# 2. 更新 CHANGELOG.md
# 3. 提交更改
git add .
git commit -m "chore: prepare release v1.0.2"
git push

# 4. 创建并推送 tag
git tag -a v1.0.2 -m "Release v1.0.2"
git push origin v1.0.2
```

推送 tag 后，GitHub Actions 会自动：
- ✅ 创建 GitHub Release
- ✅ 发布所有包到 crates.io

## 📝 使用发布脚本（可选）

### Linux/macOS

```bash
chmod +x scripts/release.sh
./scripts/release.sh
```

### Windows (PowerShell)

```powershell
.\scripts\release.ps1 -Version "1.0.2"
```

## ✅ 验证发布

发布完成后，验证：

```bash
# 检查 crates.io
cargo search pixabay-sdk
cargo search pexels-sdk
cargo search fusion-media-provider

# 检查 GitHub Release
# 访问: https://github.com/<username>/<repo>/releases
```

## 🎯 工作流概览

```
推送代码 → CI 测试 → 通过 ✅
    ↓
创建 Tag → 自动发布 → GitHub Release + crates.io ✅
```

## 📚 更多信息

- [详细发布指南](PUBLISHING.md)
- [工作流说明](workflows/README.md)
