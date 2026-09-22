#!/usr/bin/env bash
# ============================================================================
# Gitstars 一键部署到远端服务器
#
# 流程：
#   1. 同步版本号到 VERSION 与本地 .env 的 GITSTARS_IMAGE_TAG
#   2. scripts/build-image.sh --gzip 构建并导出 tar.gz
#   3. scp 镜像包与 compose 文件到远端
#      （**不传 .env**：远端 .env 由服务器自己维护，含 Client ID / Secret / 端口等）
#   4. 校验本地 / 远端 tar.gz 的 md5，一致后进入下一步（不一致则重传）
#   5. 在远端 gunzip -c | podman load 加载镜像
#   6. --up 时顺带在远端拉起 podman-compose（版本号以环境变量内联传入，不依赖远端 .env）
#
# 用法：
#   scripts/deploy-server.sh 1.0.0                # 构建 → 上传 → 加载
#   scripts/deploy-server.sh 1.0.0 --up           # 加载后再重启容器
#   scripts/deploy-server.sh 1.0.0 --up --tls     # 远端用 compose.tls.yaml 启动
#   scripts/deploy-server.sh 1.0.0 --skip-build   # 复用已存在的 tar.gz
#
# 可覆盖的环境变量（一般无需设置）：
#   DEPLOY_REMOTE      远端 ssh 别名（默认 arch-server）
#   DEPLOY_REMOTE_DIR  远端目标目录（默认 /opt/1panel/docker/compose/gitstars/）
#   DEPLOY_MAX_RETRY   md5 校验失败时的最大重传次数（默认 5）
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION_FILE="$ROOT/VERSION"
ENV_FILE="$ROOT/.env"
COMPOSE_FILE="$ROOT/compose.yaml"
COMPOSE_TLS_FILE="$ROOT/compose.tls.yaml"

REMOTE="${DEPLOY_REMOTE:-arch-server}"
REMOTE_DIR="${DEPLOY_REMOTE_DIR:-/opt/1panel/docker/compose/gitstars/}"
MAX_RETRY="${DEPLOY_MAX_RETRY:-5}"

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m  ✓\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m  !\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m错误：\033[0m %s\n' "$*" >&2; exit 1; }

# ---- 参数解析 --------------------------------------------------------------
DO_UP=0
USE_TLS=0
SKIP_BUILD=0

usage() { awk 'NR>1 && /^set -euo/{exit} NR>1{sub(/^# ?/,""); print}' "$0"; }

POSITIONAL=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --up)             DO_UP=1;      shift ;;
        --tls)            USE_TLS=1;    shift ;;
        --skip-build)     SKIP_BUILD=1; shift ;;
        -h|--help)        usage; exit 0 ;;
        -*)               die "未知参数：$1（--help 查看用法）" ;;
        *)                POSITIONAL+=("$1"); shift ;;
    esac
done

if [[ ${#POSITIONAL[@]} -ne 1 ]]; then
    die "用法：$(basename "$0") <版本号> [--up] [--tls]，例如 $(basename "$0") 1.0.0"
fi
VERSION="${POSITIONAL[0]}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "版本号格式非法：$VERSION（应形如 1.0.0）"
fi

# ---- 前置检查 --------------------------------------------------------------
[[ -f "$VERSION_FILE" ]]   || die "缺少 $VERSION_FILE"
[[ -f "$ENV_FILE" ]]       || die "缺少 $ENV_FILE"
[[ -f "$COMPOSE_FILE" ]]   || die "缺少 $COMPOSE_FILE"
if [[ $USE_TLS -eq 1 ]]; then
    [[ -f "$COMPOSE_TLS_FILE" ]] || die "缺少 $COMPOSE_TLS_FILE"
fi
command -v scp    >/dev/null || die "未找到 scp"
command -v ssh    >/dev/null || die "未找到 ssh"
command -v md5sum >/dev/null || die "未找到 md5sum"
command -v gzip   >/dev/null || die "未找到 gzip"

# 镜像仓库名（决定 tar.gz 文件名前缀），与 build-image.sh 读取同一来源
IMAGE_REPO="gitstars"
if grep -q '^GITSTARS_IMAGE_REPO=' "$ENV_FILE"; then
    IMAGE_REPO="$(grep '^GITSTARS_IMAGE_REPO=' "$ENV_FILE" | tail -n1 | cut -d= -f2- | tr -d '[:space:]')"
fi
IMAGE_NAME="${IMAGE_REPO##*/}"
TARBALL_NAME="${IMAGE_NAME}-${VERSION}.tar.gz"
TARBALL="$ROOT/dist/$TARBALL_NAME"

# ---- 1. 同步版本号 ---------------------------------------------------------
info "同步版本号 $VERSION 到 VERSION 与 .env"
printf '%s\n' "$VERSION" > "$VERSION_FILE"
if grep -q '^GITSTARS_IMAGE_TAG=' "$ENV_FILE"; then
    sed -i "s|^GITSTARS_IMAGE_TAG=.*|GITSTARS_IMAGE_TAG=${VERSION}|" "$ENV_FILE"
else
    printf '\nGITSTARS_IMAGE_TAG=%s\n' "$VERSION" >> "$ENV_FILE"
fi
ok "VERSION=$(tr -d '[:space:]' < "$VERSION_FILE")"
ok ".env GITSTARS_IMAGE_TAG=$(grep '^GITSTARS_IMAGE_TAG=' "$ENV_FILE" | tail -n1 | cut -d= -f2-)"

# ---- 2. 构建并导出镜像包 ---------------------------------------------------
if [[ $SKIP_BUILD -eq 1 ]]; then
    info "跳过构建（--skip-build）"
    [[ -f "$TARBALL" ]] || die "--skip-build 但找不到产物：$TARBALL"
else
    info "构建镜像：scripts/build-image.sh --gzip"
    "$ROOT/scripts/build-image.sh" --gzip
fi
[[ -f "$TARBALL" ]] || die "构建后未找到产物：$TARBALL"
ok "镜像包已生成：$TARBALL（$(du -h "$TARBALL" | cut -f1)）"

# ---- 3. 上传到远端 ---------------------------------------------------------
info "上传到 $REMOTE:$REMOTE_DIR"
ssh "$REMOTE" "mkdir -p '$REMOTE_DIR'"

UPLOAD_FILES=("$TARBALL" "$COMPOSE_FILE")
if [[ $USE_TLS -eq 1 ]]; then
    UPLOAD_FILES+=("$COMPOSE_TLS_FILE")
fi
scp "${UPLOAD_FILES[@]}" "$REMOTE:$REMOTE_DIR"
ok "上传完成（远端 .env 未改动）"

# ---- 4. md5 校验（不一致则重传）-------------------------------------------
local_md5() { md5sum "$TARBALL" | awk '{print $1}'; }
remote_md5() {
    ssh "$REMOTE" "md5sum '${REMOTE_DIR}${TARBALL_NAME}' 2>/dev/null" 2>/dev/null \
        | awk '{print $1}' || true
}

info "校验 md5"
LMD5="$(local_md5)"
RMD5="$(remote_md5)"
retry=0
while [[ "$LMD5" != "$RMD5" ]]; do
    warn "md5 不一致：本地=$LMD5 远端=${RMD5:-<无>}"
    retry=$((retry + 1))
    if (( retry > MAX_RETRY )); then
        die "已重传 $MAX_RETRY 次仍不一致，请检查网络或远端目录权限"
    fi
    info "第 $retry 次重传..."
    scp "$TARBALL" "$REMOTE:$REMOTE_DIR"
    LMD5="$(local_md5)"
    RMD5="$(remote_md5)"
done
ok "md5 一致：$LMD5"

# ---- 5. 远端加载镜像 -------------------------------------------------------
info "在 $REMOTE 上加载镜像"
ssh "$REMOTE" "gunzip -c '${REMOTE_DIR}${TARBALL_NAME}' | podman load"
ok "镜像加载完成"

# ---- 6. 可选：远端拉起容器 -------------------------------------------------
# 版本号用环境变量内联传入：shell 环境优先于远端 .env，因此不必去改服务器的配置文件。
COMPOSE_ARGS="-f compose.yaml"
if [[ $USE_TLS -eq 1 ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS -f compose.tls.yaml"
fi
UP_CMD="cd '$REMOTE_DIR' && GITSTARS_IMAGE_TAG='$VERSION' podman-compose $COMPOSE_ARGS"

if [[ $DO_UP -eq 1 ]]; then
    info "在 $REMOTE 上重启容器（podman-compose $COMPOSE_ARGS up -d，镜像版本 $VERSION）"
    ssh "$REMOTE" "$UP_CMD up -d"
    ok "容器已更新"
    ssh "$REMOTE" "$UP_CMD ps" || true
fi

info "部署完成（版本 $VERSION）"
if [[ $DO_UP -eq 0 ]]; then
    echo "  如需启动/更新容器，请在 $REMOTE 上执行："
    echo "    $UP_CMD up -d"
fi