#!/usr/bin/env bash
# =============================================================================
# release.sh - 一键在 GitHub 和 Gitee 上创建 Release 发布页面
# =============================================================================
# 用法:
#   ./release.sh <version>             发布指定版本（如 v1.3.4）
#   ./release.sh <version> --draft      创建草稿版本（仅 GitHub 支持）
#   ./release.sh <version> --notes "..." 自定义 Release Notes
#
# 依赖:
#   - gh (GitHub CLI): brew install gh
#   - curl + jq (用于 Gitee API)
#
# 环境变量 (可选):
#   GITEE_TOKEN: Gitee 个人访问令牌（不设置则跳过 Gitee）
#   GITHUB_TOKEN: GitHub 令牌（可选，gh 默认使用 SSH/OAuth）
# =============================================================================

set -euo pipefail

# ── 配置 ────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_ROOT/release"
VERSION_FILE="$PROJECT_ROOT/Cargo.toml"

# 颜色输出
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*"; }

# ── 参数解析 ─────────────────────────────────────────────────────────────
VERSION=""
DRAFT=false
NOTES=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --draft) DRAFT=true; shift ;;
        --notes) NOTES="$2"; shift 2 ;;
        --notes=*) NOTES="${1#*=}"; shift ;;
        -h|--help)
            sed -n '2,20p' "$0" | grep -v '^#!/'
            exit 0
            ;;
        *) VERSION="$1"; shift ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    # 从 Cargo.toml 读取版本
    VERSION=$(grep '^version' "$VERSION_FILE" | cut -d '"' -f2)
    info "未指定版本号，使用 Cargo.toml 中的版本: v$VERSION"
fi

# 确保 v 前缀
[[ "$VERSION" != v* ]] && VERSION="v$VERSION"

# ── 检查依赖 ─────────────────────────────────────────────────────────────
check_deps() {
    if ! command -v gh &>/dev/null; then
        err "gh (GitHub CLI) 未安装，请运行: brew install gh"
        exit 1
    fi
    if ! gh auth status &>/dev/null; then
        err "gh 未登录，请运行: gh auth login"
        exit 1
    fi
    if ! command -v curl &>/dev/null; then
        err "curl 未安装"
        exit 1
    fi
    if ! command -v jq &>/dev/null; then
        err "jq 未安装，请运行: brew install jq"
        exit 1
    fi
}

# ── 检查 Release 文件 ─────────────────────────────────────────────────────
check_assets() {
    if [[ ! -d "$RELEASE_DIR" ]]; then
        err "release/ 目录不存在，请先运行: make release"
        exit 1
    fi

    ASSETS=()
    while IFS= read -r -d '' file; do
        ASSETS+=("$file")
    done < <(find "$RELEASE_DIR" -type f -maxdepth 1 -print0)

    if [[ ${#ASSETS[@]} -eq 0 ]]; then
        err "release/ 目录中没有文件，请先运行: make release"
        exit 1
    fi

    info "待发布的文件 (${#ASSETS[@]} 个):"
    for f in "${ASSETS[@]}"; do
        echo "    $(basename "$f") ($(du -h "$f" | cut -f1))"
    done
}

# ── 生成 Release Notes ─────────────────────────────────────────────────────
generate_notes() {
    if [[ -n "$NOTES" ]]; then
        echo "$NOTES"
        return
    fi

    # 尝试从上一个 tag 生成 changelog
    local prev_tag
    prev_tag=$(git tag --sort=-creatordate | grep -v "$VERSION" | head -1 2>/dev/null || true)

    cat <<EOF
## 🚀 $VERSION

### 📦 构建产物
- $(ls "$RELEASE_DIR" | grep -v plugin-loader | head -1)
- plugin-loader (Go 插件加载器)

### 🔧 变更
- $(git log --oneline "$prev_tag..HEAD" 2>/dev/null | head -5 | sed 's/^/- /' || echo "- 新版本发布")

### 📝 完整变更日志
查看 [Commits](https://github.com/Azithromycin15/rabber-stm32-linktool/commits/master)
EOF
}

# ── GitHub Release ─────────────────────────────────────────────────────────
publish_github() {
    info "━━━ 创建 GitHub Release ($VERSION) ━━━"

    local gh_args=()
    for f in "${ASSETS[@]}"; do
        gh_args+=("$f")
    done

    local draft_flag=""
    $DRAFT && draft_flag="--draft"

    local notes_content
    notes_content=$(generate_notes)

    if gh release view "$VERSION" &>/dev/null 2>&1; then
        warn "GitHub Release $VERSION 已存在，跳过"
        return 0
    fi

    gh release create "$VERSION" \
        --title "$VERSION" \
        --notes "$notes_content" \
        $draft_flag \
        "${gh_args[@]}" \
        --repo Azithromycin15/rabber-stm32-linktool

    ok "GitHub Release 发布成功: https://github.com/Azithromycin15/rabber-stm32-linktool/releases/tag/$VERSION"
}

# ── Gitee Release ──────────────────────────────────────────────────────────
publish_gitee() {
    info "━━━ 创建 Gitee Release ($VERSION) ━━━"

    if [[ -z "${GITEE_TOKEN:-}" ]]; then
        warn "未设置 GITEE_TOKEN 环境变量，跳过 Gitee Release 创建"
        warn "获取 Token: https://gitee.com/profile/personal_access_tokens"
        warn "使用方式: export GITEE_TOKEN=your_token"
        return 0
    fi

    local notes_content
    notes_content=$(generate_notes)

    # 创建 Release (Gitee API v5)
    local response
    response=$(curl -s -X POST "https://gitee.com/api/v5/repos/kroazithromycin/rabber-stm32-linktool/releases" \
        -H "Content-Type: application/json" \
        -d "$(jq -n \
            --arg tag "$VERSION" \
            --arg name "$VERSION" \
            --arg body "$notes_content" \
            --arg target "master" \
            '{tag_name: $tag, name: $name, body: $body, target_commitish: $target, prerelease: false}')" \
        -H "Authorization: token $GITEE_TOKEN")

    local release_id
    release_id=$(echo "$response" | jq -r '.id // empty')

    if [[ -z "$release_id" ]]; then
        local err_msg
        err_msg=$(echo "$response" | jq -r '.message // "未知错误"')
        err "Gitee Release 创建失败: $err_msg"
        return 1
    fi

    ok "Gitee Release 创建成功 (ID: $release_id)"

    # 上传附件
    for f in "${ASSETS[@]}"; do
        local fname
        fname=$(basename "$f")
        info "  上传: $fname ..."
        curl -s -X POST \
            "https://gitee.com/api/v5/repos/kroazithromycin/rabber-stm32-linktool/releases/$release_id/attach_files" \
            -F "file=@$f" \
            -H "Authorization: token $GITEE_TOKEN" > /dev/null
        ok "  $fname ✓"
    done

    ok "Gitee Release 发布成功: https://gitee.com/kroazithromycin/rabber-stm32-linktool/releases"
}

# ── 推送 Tag ───────────────────────────────────────────────────────────────
push_tag() {
    if git tag -l | grep -q "^$VERSION$"; then
        warn "Git tag $VERSION 已存在，跳过创建"
    else
        info "创建 Git tag: $VERSION"
        git tag -a "$VERSION" -m "Release $VERSION"
    fi

    info "推送 tag 到 origin (GitHub + Gitee)..."
    git push origin "$VERSION"
    ok "Tag $VERSION 已推送"
}

# ── 主流程 ─────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo "  ╔══════════════════════════════════════════════╗"
    echo "  ║     🚀 Rabber Release Publisher             ║"
    echo "  ║     Version: $VERSION                          ║"
    echo "  ╚══════════════════════════════════════════════╝"
    echo ""

    check_deps
    check_assets

    # 1. 推送 tag
    push_tag

    # 2. GitHub Release
    publish_github

    # 3. Gitee Release
    publish_gitee

    echo ""
    ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    ok "  发布完成! 版本: $VERSION"
    ok "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

main
