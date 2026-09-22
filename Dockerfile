# syntax=docker/dockerfile:1
# ============================================================================
# Gitstars 运行时镜像
#
# 设计取舍：本 Dockerfile 只负责组装运行时环境，不执行 cargo / cargo-leptos 构建。
# 二进制与前端站点由 scripts/build-image.sh 在 **宿主机** 编译后 COPY 进来。
#
# 基础镜像必须是 debian:trixie-slim（glibc 2.41）而不是 bookworm-slim（glibc 2.36）：
# 产物对 glibc 的最低要求由**编译环境**决定——宿主 glibc（Arch 实测 2.44）会给产物
# 带上 GLIBC_2.38 这类高版本符号，放进 bookworm 后连加载器都过不去，
# 容器会每几秒重启一次且没有任何应用日志。build-image.sh 会先做符号校验再构建。
#
# 构建上下文为仓库根目录，构建前须先执行：
#   scripts/build-image.sh          # 自动完成编译 + 校验 + podman build + 冒烟
# ============================================================================
FROM docker.io/library/debian:trixie-slim

# Debian 镜像源。默认中科大 **http**：trixie-slim 基础镜像未预装 ca-certificates，
# 用 https 会在 apt-get update 阶段报 certificate verify failed。
# 如需换清华：--build-arg DEBIAN_MIRROR=http://mirrors.tuna.tsinghua.edu.cn
ARG DEBIAN_MIRROR=http://mirrors.ustc.edu.cn
ARG APP_UID=10001
ARG APP_GID=10001

# 时区须在安装 tzdata 之前设定，避免其进入交互式配置流程
ENV DEBIAN_FRONTEND=noninteractive \
    TZ=Asia/Shanghai

# ---- 1. 换源、装依赖、设时区 ----------------------------------------------
# 注意替换顺序：debian-security 必须排在 debian 之前，
# 否则 "http://deb.debian.org/debian" 会先吃掉 ".../debian-security" 的前缀。
RUN set -eux; \
    sed -i \
        -e "s|http://deb.debian.org/debian-security|${DEBIAN_MIRROR}/debian-security|g" \
        -e "s|http://deb.debian.org/debian|${DEBIAN_MIRROR}/debian|g" \
        /etc/apt/sources.list.d/debian.sources; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        tzdata \
        curl \
        tini \
    ; \
    ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime; \
    echo "${TZ}" > /etc/timezone; \
    rm -rf /var/lib/apt/lists/*

# ---- 2. 非 root 运行用户 ---------------------------------------------------
# 保留 /bin/bash 作为登录 shell，便于 docker exec 进入排查
RUN set -eux; \
    groupadd -g "${APP_GID}" gitstars; \
    useradd -u "${APP_UID}" -g "${APP_GID}" -M -d /app -s /bin/bash gitstars

# ---- 3. 应用目录 -----------------------------------------------------------
WORKDIR /app

# 服务端二进制（SSR + server functions，图标 sprite 已 include_str! 内嵌）
COPY --chown=gitstars:gitstars target/release/gitstars /app/gitstars

# 前端站点：wasm / js / css 与 public 静态资源，由 leptos_axum 按 LEPTOS_SITE_ROOT 提供
COPY --chown=gitstars:gitstars target/site/ /app/site/

# 许可证原文：MIT 要求「副本或实质部分」附带版权声明与许可声明，镜像也算分发。
COPY --chown=gitstars:gitstars LICENSE /app/LICENSE

# ---- 4. 数据卷 -------------------------------------------------------------
# /app/data  持久化 Stars 与排行榜 JSON 快照（GITSTARS_DATA_DIR）
# 注意：目录创建与 chown 必须发生在 VOLUME 声明之前，
# 否则这些写入落在卷挂载点之上，容器启动时会被空卷遮蔽。
RUN set -eux; \
    chmod +x /app/gitstars; \
    mkdir -p /app/data; \
    chown -R gitstars:gitstars /app

VOLUME ["/app/data"]

# ---- 5. 运行环境 -----------------------------------------------------------
# LEPTOS_* 是 leptos_config 读取的站点配置；镜像内没有 Cargo.toml，必须显式给定。
ENV LEPTOS_OUTPUT_NAME=gitstars \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_ENV=PROD \
    GITSTARS_DATA_DIR=/app/data \
    RUST_LOG=info

USER gitstars

EXPOSE 8080

# 应用自带 /healthz 端点。
# --noproxy '*' 必要：宿主若导出 http_proxy，podman-compose 会将其透传进容器，
# 使 curl 走代理访问 127.0.0.1 而返回 502，导致容器被误判为不健康。
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS --noproxy '*' http://127.0.0.1:8080/healthz || exit 1

# tini 作为 PID 1：正确转发信号并回收僵尸进程
ENTRYPOINT ["/usr/bin/tini", "--", "/app/gitstars"]

# ---- 6. 镜像元数据 ---------------------------------------------------------
# 版本号由 scripts/build-image.sh 从仓库根 VERSION 文件注入（--build-arg APP_VERSION）。
# 刻意放在文件末尾：标签值每次发版都会变化，若置于前部会使后续所有层的构建缓存失效。
ARG APP_VERSION=dev
LABEL org.opencontainers.image.title="gitstars" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.description="Gitstars：GitHub Stars 管理器与排行榜（Leptos 全栈）" \
      org.opencontainers.image.base.name="docker.io/library/debian:trixie-slim"