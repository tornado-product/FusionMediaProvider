#!/bin/bash
# 发布脚本 - 自动创建 tag 并触发发布流程

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否在 git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}错误: 不在 git 仓库中${NC}"
    exit 1
fi

# 获取当前版本
CURRENT_VERSION=$(grep -E '^version\s*=' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo -e "${GREEN}当前版本: ${CURRENT_VERSION}${NC}"

# 询问新版本号
read -p "请输入新版本号 (例如 1.0.2): " NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}错误: 版本号不能为空${NC}"
    exit 1
fi

# 验证版本号格式
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}错误: 版本号格式不正确，应为 x.y.z (例如 1.0.2)${NC}"
    exit 1
fi

# 检查是否有未提交的更改
if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}警告: 有未提交的更改${NC}"
    read -p "是否继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 检查 tag 是否已存在
if git rev-parse "v${NEW_VERSION}" >/dev/null 2>&1; then
    echo -e "${RED}错误: Tag v${NEW_VERSION} 已存在${NC}"
    exit 1
fi

# 更新版本号（如果需要）
if [ "$CURRENT_VERSION" != "$NEW_VERSION" ]; then
    echo -e "${YELLOW}更新版本号到 ${NEW_VERSION}...${NC}"
    
    # 更新 workspace 版本
    sed -i.bak "s/^version = .*/version = \"${NEW_VERSION}\"/" Cargo.toml
    rm Cargo.toml.bak
    
    # 提交版本更新
    git add Cargo.toml
    git commit -m "chore: bump version to ${NEW_VERSION}"
    git push
fi

# 创建 tag
echo -e "${GREEN}创建 tag v${NEW_VERSION}...${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# 推送 tag
echo -e "${GREEN}推送 tag...${NC}"
git push origin "v${NEW_VERSION}"

echo -e "${GREEN}✅ 发布流程已启动！${NC}"
echo -e "${YELLOW}GitHub Actions 将自动:${NC}"
echo "  1. 创建 GitHub Release"
echo "  2. 发布所有包到 crates.io"
echo ""
echo "查看进度: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
