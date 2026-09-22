#!/usr/bin/env bash
# ============================================================================
# 构建 Gitstars 镜像
#
# 流程：cargo leptos build --release（宿主机）→ glibc 符号校验 → 镜像构建
#       → 容器冒烟（/healthz）→ 可选推送 / 导出 tar
#
# 镜像版本号独立于 Rust crate 版本，唯一来源是仓库根目录的 VERSION 文件
# （也可用 --version 临时覆盖），标签形如 gitstars:1.0.0，禁止 latest。
#
# 为什么在宿主机编译：宿主已装好 wasm32 target / cargo-leptos / tailwindcss，
# 复用 target/ 缓存比在容器里全量重编快一个数量级。代价是产物的 glibc 符号
# 下限由宿主决定（Arch 实测 glibc 2.44，产物要求 GLIBC_2.38），
# 因此基础镜像必须用 trixie（glibc 2.41）而非 bookworm（2.36）。
# 本脚本会在构建前校验这一点，避免打出「容器起不来且没有任何日志」的镜像。
#
# 用法：
#   scripts/build-image.sh                  # 编译 + 构建镜像
#   scripts/build-image.sh --skip-build     # 跳过编译（产物已存在）
#   scripts/build-image.sh --push           # 构建后推送到镜像仓库
#   scripts/build-image.sh --save           # 额外导出 tar 到 dist/
#   scripts/build-image.sh --gzip           # 额外导出 tar.gz 到 dist/（deploy-server.sh 用）
#   scripts/build-image.sh --version 1.2.3  # 指定版本
#   scripts/build-image.sh --tag my/app:v1  # 完全自定义标签
#   scripts/build-image.sh --no-smoke       # 跳过容器冒烟
#
# 镜像仓库名可用 .env 的 GITSTARS_IMAGE_REPO 覆盖（默认 gitstars，
# 推送前请改成带仓库前缀的名字，例如 ghcr.io/<你的用户名>/gitstars）。
# ============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
DO_PUSH=0
DO_SAVE=0
DO_GZIP=0
DO_SMOKE=1
TAG_OVERRIDE=""
IMAGE_VERSION=""

info() { printf '\033[1;34m[gitstars]\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m[gitstars]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[gitstars]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[gitstars]\033[0m %s\n' "$*" >&2; exit 1; }

# .env 提供镜像仓库名（GITSTARS_IMAGE_REPO）与部署侧版本（GITSTARS_IMAGE_TAG）
if [[ -f .env ]]; then
    # shellcheck disable=SC1091
    set -a; source .env; set +a
fi
IMAGE_REPO="${GITSTARS_IMAGE_REPO:-gitstars}"
# 去掉 registry / namespace 前缀，供导出文件名使用
IMAGE_NAME="${IMAGE_REPO##*/}"

# ---- 参数解析 --------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build) SKIP_BUILD=1; shift ;;
        --push)       DO_PUSH=1;    shift ;;
        --save)       DO_SAVE=1;    shift ;;
        --gzip)       DO_GZIP=1;    shift ;;
        --no-smoke)   DO_SMOKE=0;   shift ;;
        --version)    IMAGE_VERSION="$2"; shift 2 ;;
        --tag)        TAG_OVERRIDE="$2";  shift 2 ;;
        -h|--help)    awk 'NR>1 && /^set -euo/{exit} NR>1{sub(/^# ?/,""); print}' "$0"; exit 0 ;;
        *)            die "未知参数：$1（--help 查看用法）" ;;
    esac
done

# ---- 版本与标签 ------------------------------------------------------------
if [[ -n "$TAG_OVERRIDE" ]]; then
    TAG="$TAG_OVERRIDE"
    IMAGE_VERSION="${TAG##*:}"
else
    if [[ -z "$IMAGE_VERSION" ]]; then
        [[ -f VERSION ]] || die "缺少 VERSION 文件，请写入形如 1.0.0 的版本号"
        IMAGE_VERSION="$(tr -d '[:space:]' < VERSION)"
    fi
    TAG="${IMAGE_REPO}:${IMAGE_VERSION}"
fi
[[ "$TAG" == *:latest ]] && die "禁止使用 latest 标签，请在 VERSION 中维护镜像版本号"
[[ -n "$IMAGE_VERSION" ]] || die "无法确定镜像版本号"

# ---- 容器引擎 --------------------------------------------------------------
if command -v podman >/dev/null 2>&1; then
    ENGINE=podman
elif command -v docker >/dev/null 2>&1; then
    ENGINE=docker
else
    die "未找到 podman 或 docker"
fi
info "容器引擎：$ENGINE"

# ---- 1. 编译 ---------------------------------------------------------------
if [[ $SKIP_BUILD -eq 1 ]]; then
    info "跳过编译（--skip-build）"
else
    command -v cargo-leptos >/dev/null 2>&1 || die "未安装 cargo-leptos：cargo install cargo-leptos --locked"
    command -v tailwindcss >/dev/null 2>&1 || die "未找到 tailwindcss：请安装 Tailwind v4 standalone CLI 并加入 PATH"
    info "编译（cargo leptos build --release）..."
    cargo leptos build --release
fi

BIN="target/release/gitstars"
SITE="target/site"
[[ -x "$BIN" ]] || die "$BIN 不存在或不可执行，请先执行 cargo leptos build --release"
[[ -d "$SITE" ]] || die "$SITE 不存在，请先执行 cargo leptos build --release"
ok "二进制大小：$(du -h "$BIN" | cut -f1)，前端站点：$(du -sh "$SITE" | cut -f1)"

# 防呆：cargo leptos build（不带 --release）会把 target/site 覆盖成 debug 产物，
# wasm 会从 ~1MB 涨到十几 MB。--skip-build 时很容易踩到，这里提前拦一下。
WASM_SIZE=$(stat -c %s "$SITE/pkg/gitstars.wasm" 2>/dev/null || echo 0)
if [[ "$WASM_SIZE" -gt 5000000 ]]; then
    warn "$SITE/pkg/gitstars.wasm 有 $((WASM_SIZE / 1024 / 1024))MB，看起来是 debug 构建。"
    warn "请先执行 cargo leptos build --release 重新生成产物，或去掉 --skip-build 重跑本脚本。"
    die "已中止，避免打出体积臃肿的镜像。"
fi

# ---- 2. glibc 符号校验 -----------------------------------------------------
BASE_IMAGE="$(awk '/^FROM /{print $2; exit}' Dockerfile)"
[[ -n "$BASE_IMAGE" ]] || die "无法从 Dockerfile 解析基础镜像"

BIN_GLIBC="$(objdump -T "$BIN" 2>/dev/null \
    | grep -o 'GLIBC_[0-9]\+\(\.[0-9]\+\)*' \
    | sed 's/^GLIBC_//' | sort -uV | tail -1)"
[[ -n "$BIN_GLIBC" ]] || die "无法解析 $BIN 的 glibc 符号要求"

info "校验 glibc：产物要求 ${BIN_GLIBC}，基础镜像 ${BASE_IMAGE} ..."
BASE_GLIBC="$("$ENGINE" run --rm "$BASE_IMAGE" sh -c 'ldd --version | head -1' 2>/dev/null | awk '{print $NF}')"
[[ -n "$BASE_GLIBC" ]] || die "无法读取 $BASE_IMAGE 的 glibc 版本（镜像可能未拉取）"

if [[ "$(printf '%s\n%s\n' "$BIN_GLIBC" "$BASE_GLIBC" | sort -V | tail -1)" != "$BASE_GLIBC" ]]; then
    die "产物要求 glibc ${BIN_GLIBC}，而 ${BASE_IMAGE} 只有 ${BASE_GLIBC}：
     二进制会在容器启动时因找不到符号而立刻退出（且没有任何应用日志）。
     解决办法：把 Dockerfile 的 FROM 换成更新的 debian 版本，或在同版本容器内编译。已中止。"
fi
ok "glibc 校验通过（产物 ${BIN_GLIBC} ≤ 基础镜像 ${BASE_GLIBC}）"

# ---- 3. 构建镜像 -----------------------------------------------------------
BUILD_ARGS=(
    --file Dockerfile
    --tag "$TAG"
    --build-arg "APP_VERSION=${IMAGE_VERSION}"
)
# podman 默认输出 OCI 格式，会静默丢弃 Dockerfile 中的 HEALTHCHECK，
# 必须显式指定 docker 格式，否则容器健康状态无法被 compose depends_on 感知。
if [[ "$ENGINE" == "podman" ]]; then
    BUILD_ARGS+=(--format docker)
fi

info "构建镜像 $TAG（版本 $IMAGE_VERSION）"
"$ENGINE" build "${BUILD_ARGS[@]}" .
ok "镜像构建完成：$TAG"

# ---- 4. 容器冒烟 -----------------------------------------------------------
if [[ $DO_SMOKE -eq 1 ]]; then
    SMOKE_NAME="gitstars-smoke-$$"
    SMOKE_PORT="${GITSTARS_SMOKE_PORT:-18080}"
    info "冒烟测试：$SMOKE_NAME（127.0.0.1:${SMOKE_PORT} → 8080）"

    cleanup_smoke() { "$ENGINE" rm -f "$SMOKE_NAME" >/dev/null 2>&1 || true; }
    trap cleanup_smoke EXIT

    "$ENGINE" run -d --rm --name "$SMOKE_NAME" \
        -p "127.0.0.1:${SMOKE_PORT}:8080" \
        "$TAG" >/dev/null

    HEALTHY=0
    for _ in $(seq 1 30); do
        if curl -fsS --noproxy '*' "http://127.0.0.1:${SMOKE_PORT}/healthz" >/dev/null 2>&1; then
            HEALTHY=1
            break
        fi
        sleep 1
    done

    if [[ $HEALTHY -ne 1 ]]; then
        warn "冒烟失败，容器日志："
        "$ENGINE" logs "$SMOKE_NAME" >&2 || true
        cleanup_smoke
        trap - EXIT
        die "镜像未通过 /healthz 冒烟测试，已中止（未推送）"
    fi
    ok "冒烟通过：/healthz 正常"

    INDEX_CODE="$(curl -s --noproxy '*' -o /dev/null -w '%{http_code}' "http://127.0.0.1:${SMOKE_PORT}/" || true)"
    [[ "$INDEX_CODE" == "200" ]] || warn "首页返回 ${INDEX_CODE}（预期 200），请检查前端站点是否完整"

    cleanup_smoke
    trap - EXIT
fi

# ---- 5. 推送 / 导出 --------------------------------------------------------
if [[ $DO_PUSH -eq 1 ]]; then
    if [[ "$TAG" != */* ]]; then
        warn "标签 $TAG 没有仓库前缀，podman push 会推到默认 registry。"
        warn "建议在 .env 里设置 GITSTARS_IMAGE_REPO=ghcr.io/<用户名>/gitstars 后重跑。"
    fi
    info "推送 $TAG ..."
    "$ENGINE" push "$TAG"
    ok "推送完成"
fi

if [[ $DO_SAVE -eq 1 ]]; then
    mkdir -p dist
    OUTPUT="dist/${IMAGE_NAME}-${IMAGE_VERSION}.tar"
    info "导出镜像到 $OUTPUT ..."
    "$ENGINE" save -o "$OUTPUT" "$TAG"
    ok "已导出：$OUTPUT（$(du -h "$OUTPUT" | cut -f1)）"
fi

if [[ $DO_GZIP -eq 1 ]]; then
    mkdir -p dist
    OUTPUT="dist/${IMAGE_NAME}-${IMAGE_VERSION}.tar.gz"
    info "导出镜像（gzip）到 $OUTPUT ..."
    "$ENGINE" save "$TAG" | gzip -c > "$OUTPUT"
    ok "已导出：$OUTPUT（$(du -h "$OUTPUT" | cut -f1)）"
fi

cat <<EOF

完成。镜像：$TAG

  启动（HTTP）：   GITSTARS_IMAGE_TAG=${IMAGE_VERSION} podman-compose up -d
  启动（HTTPS）：  GITSTARS_IMAGE_TAG=${IMAGE_VERSION} podman-compose -f compose.yaml -f compose.tls.yaml up -d
  服务器上导入：   podman load -i dist/gitstars-${IMAGE_VERSION}.tar

EOF