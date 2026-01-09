# PowerShell 发布脚本 - 自动创建 tag 并触发发布流程

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

# 颜色输出函数
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

# 检查是否在 git 仓库中
if (-not (Test-Path .git)) {
    Write-ColorOutput Red "错误: 不在 git 仓库中"
    exit 1
}

# 获取当前版本
$CargoToml = Get-Content Cargo.toml
$CurrentVersion = ($CargoToml | Select-String -Pattern '^version\s*=' | Select-Object -First 1).Line -replace '.*"([^"]+)".*', '$1'

Write-ColorOutput Green "当前版本: $CurrentVersion"

# 验证版本号格式
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-ColorOutput Red "错误: 版本号格式不正确，应为 x.y.z (例如 1.0.2)"
    exit 1
}

# 检查是否有未提交的更改
$Status = git status --porcelain
if ($Status) {
    Write-ColorOutput Yellow "警告: 有未提交的更改"
    $Continue = Read-Host "是否继续? (y/N)"
    if ($Continue -ne 'y' -and $Continue -ne 'Y') {
        exit 1
    }
}

# 检查 tag 是否已存在
$TagExists = git rev-parse "v$Version" 2>$null
if ($TagExists) {
    Write-ColorOutput Red "错误: Tag v$Version 已存在"
    exit 1
}

# 更新版本号（如果需要）
if ($CurrentVersion -ne $Version) {
    Write-ColorOutput Yellow "更新版本号到 $Version..."
    
    # 更新 workspace 版本
    (Get-Content Cargo.toml) -replace '^version = .*', "version = `"$Version`"" | Set-Content Cargo.toml
    
    # 提交版本更新
    git add Cargo.toml
    git commit -m "chore: bump version to $Version"
    git push
}

# 创建 tag
Write-ColorOutput Green "创建 tag v$Version..."
git tag -a "v$Version" -m "Release v$Version"

# 推送 tag
Write-ColorOutput Green "推送 tag..."
git push origin "v$Version"

Write-ColorOutput Green "✅ 发布流程已启动！"
Write-ColorOutput Yellow "GitHub Actions 将自动:"
Write-Output "  1. 创建 GitHub Release"
Write-Output "  2. 发布所有包到 crates.io"
Write-Output ""
$RemoteUrl = git config --get remote.origin.url
$RepoPath = $RemoteUrl -replace '.*github.com[:/](.*)\.git', '$1'
Write-Output "查看进度: https://github.com/$RepoPath/actions"
