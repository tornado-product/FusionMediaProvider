# GitHub Actions Workflows

本项目包含以下自动化工作流：

## 📋 工作流说明

### 1. CI (`ci.yml`)

**触发条件**: 
- 推送到 `main`/`master`/`develop` 分支
- 创建 Pull Request

**功能**:
- ✅ 多平台测试 (Linux, Windows, macOS)
- ✅ 多 Rust 版本测试
- ✅ 代码格式化检查 (`cargo fmt`)
- ✅ 代码质量检查 (`cargo clippy`)
- ✅ 构建所有包
- ✅ 运行所有测试
- ✅ 构建 Release 二进制文件（仅 main/master 分支）

### 2. Release (`release.yml`)

**触发条件**: 
- 推送以 `v` 开头的 tag（如 `v1.0.2`）

**功能**:
- ✅ 自动创建 GitHub Release
- ✅ 从 CHANGELOG.md 提取发布说明
- ✅ 按依赖顺序发布所有包到 crates.io:
  1. `pixabay-sdk`
  2. `pexels-sdk`
  3. `fusion-media-provider`（依赖前两个）

**发布顺序**: 基础 SDK 先发布，等待索引更新后再发布依赖它们的包。

### 3. Publish (`publish.yml`)

**触发条件**: 
- 手动触发（workflow_dispatch）

**功能**:
- ✅ 手动发布单个或所有包到 crates.io
- ✅ 可选择要发布的包
- ✅ 可指定版本号

**使用场景**: 
- 修复发布问题
- 单独发布某个包
- 测试发布流程

### 4. Docs (`docs.yml`)

**触发条件**: 
- 推送到 `main`/`master` 分支
- 手动触发

**功能**:
- ✅ 构建所有包的文档
- ✅ 自动部署到 GitHub Pages
- ✅ 文档地址: `https://<username>.github.io/FusionMediaProvider/`

## 🔧 配置要求

### 必需的 Secrets

在 GitHub 仓库设置中添加以下 Secret：

1. **CRATES_IO_TOKEN** (必需)
   - 获取方式: [crates.io](https://crates.io) → Account Settings → API Tokens
   - 用于发布包到 crates.io

2. **GITHUB_TOKEN** (自动)
   - GitHub 自动提供，无需手动配置
   - 用于创建 Release 和部署文档

### 配置步骤

1. 进入仓库 Settings → Secrets and variables → Actions
2. 点击 "New repository secret"
3. 添加 `CRATES_IO_TOKEN`，值为你的 crates.io API token

## 🚀 使用流程

### 自动发布（推荐）

1. 更新版本号和 CHANGELOG
2. 提交更改
3. 创建并推送 tag:
   ```bash
   git tag -a v1.0.2 -m "Release v1.0.2"
   git push origin v1.0.2
   ```
4. GitHub Actions 自动完成发布

### 手动发布

1. 进入 GitHub Actions 页面
2. 选择 "Publish to crates.io" workflow
3. 点击 "Run workflow"
4. 选择包和版本
5. 点击 "Run workflow"

## 📊 查看状态

- **CI 状态**: 在仓库首页查看
- **发布进度**: Actions 标签页
- **Release**: Releases 页面
- **文档**: GitHub Pages（如果启用）

## 🐛 故障排查

### 发布失败

1. **检查 Token**: 确保 `CRATES_IO_TOKEN` 已正确设置
2. **检查版本**: 确保版本号未被使用
3. **检查依赖**: 确保依赖的包已发布
4. **查看日志**: 在 Actions 页面查看详细错误信息

### CI 失败

1. **格式化错误**: 运行 `cargo fmt --all`
2. **Clippy 错误**: 运行 `cargo clippy --all-targets --all-features`
3. **测试失败**: 检查测试代码和依赖

## 📚 相关文档

- [发布指南](../.github/PUBLISHING.md)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [crates.io 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)
