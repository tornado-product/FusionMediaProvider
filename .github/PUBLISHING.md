# 发布指南

本文档说明如何发布新版本到 GitHub Release 和 crates.io。

## 发布流程

### 1. 更新版本号

在发布之前，需要更新所有包的版本号：

```bash
# 更新 workspace 版本（在根目录 Cargo.toml）
# 更新各个包的版本（如果需要独立版本）
```

### 2. 更新 CHANGELOG.md

在 `CHANGELOG.md` 中添加新版本的变更说明：

```markdown
## [1.0.2] - 2024-01-01

### Added
- 新功能

### Changed
- 改进

### Fixed
- 修复
```

### 3. 提交更改

```bash
git add .
git commit -m "chore: bump version to 1.0.2"
git push
```

### 4. 创建 Git Tag

创建并推送版本 tag：

```bash
# 创建带注释的 tag
git tag -a v1.0.2 -m "Release v1.0.2"

# 推送 tag
git push origin v1.0.2
```

**重要**: Tag 名称必须以 `v` 开头，例如 `v1.0.2`。

### 5. 自动发布

推送 tag 后，GitHub Actions 会自动：

1. ✅ 创建 GitHub Release
2. ✅ 发布所有包到 crates.io:
   - `pixabay-sdk`
   - `pexels-sdk`
   - `fusion-media-provider`

## 手动发布到 crates.io

如果需要手动发布单个包：

1. 在 GitHub Actions 页面选择 "Publish to crates.io" workflow
2. 点击 "Run workflow"
3. 选择要发布的包和版本
4. 点击 "Run workflow"

## 前置要求

### 1. crates.io Token

在发布到 crates.io 之前，需要：

1. 登录 [crates.io](https://crates.io)
2. 进入 Account Settings → API Tokens
3. 创建新的 token（或使用现有 token）
4. 在 GitHub 仓库设置中添加 Secret：
   - Settings → Secrets and variables → Actions
   - 添加 Secret: `CRATES_IO_TOKEN`
   - 值为你的 crates.io token

### 2. 首次发布检查清单

在首次发布之前，确保：

- [ ] 所有包的 `Cargo.toml` 配置正确
- [ ] 所有包都有有效的 `license` 字段
- [ ] 所有包都有 `description` 和 `repository` 字段
- [ ] 所有包都有 `readme` 字段（如果适用）
- [ ] 所有包都有 `keywords` 和 `categories`
- [ ] 代码已通过 `cargo clippy` 检查
- [ ] 代码已通过 `cargo fmt` 格式化
- [ ] 所有测试通过

### 3. 验证发布

发布后验证：

```bash
# 检查包是否已发布
cargo search pixabay-sdk
cargo search pexels-sdk
cargo search fusion-media-provider

# 测试安装
cargo install pixabay-sdk-cli
```

## 版本号规则

遵循 [语义化版本](https://semver.org/lang/zh-CN/)：

- **主版本号**: 不兼容的 API 修改
- **次版本号**: 向下兼容的功能性新增
- **修订号**: 向下兼容的问题修正

## 故障排查

### 发布失败

如果发布失败，检查：

1. **Token 是否正确**: 确保 `CRATES_IO_TOKEN` secret 已正确设置
2. **版本号是否冲突**: 确保版本号未被使用
3. **依赖关系**: 确保 workspace 依赖关系正确
4. **Cargo.toml**: 确保所有必需字段都已填写

### 部分包发布失败

如果某个包发布失败：

1. 检查该包的 `Cargo.toml` 配置
2. 手动发布该包：
   ```bash
   cd <package-path>
   cargo publish --token <your-token>
   ```

## 相关链接

- [crates.io 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [语义化版本](https://semver.org/lang/zh-CN/)
