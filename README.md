# Gitstars (Leptos)

一个用于整理、搜索和浏览 GitHub Stars 的仓库管理器，附带按编程语言分类的 GitHub 排行榜。

> ⚠️ **本项目是 [cfour-hi/gitstars](https://github.com/cfour-hi/gitstars) 的非官方重写版，由第三方独立开发，与原项目作者无任何关联，未获得其授权、认可或赞助。**
> 原项目**未声明任何开源许可证**，本项目也**不是 fork**，未复用其源代码。
> 详见[与原项目的关系](#与原项目的关系)、[免责声明](#免责声明)与 [NOTICE](./NOTICE)。

## 与原项目的关系

| | |
| --- | --- |
| **原项目** | [cfour-hi/gitstars](https://github.com/cfour-hi/gitstars)：Vue 3 + Vite + Pinia 单页应用，由 [cfour-hi](https://github.com/cfour-hi) 开发 |
| **本项目** | 用 Rust + Leptos 全栈**独立重写**，重新实现同样功能（Stars 整理、分类筛选、搜索、排序、README 预览、排行榜、中英文界面） |
| **关系** | **不是 fork**，未复用原项目源代码，也未获得原作者授权；只参考了它的功能与交互设计 |
| **数据源** | 排行榜继续使用原项目配套的 [cfour-hi/github-ranking](https://github.com/cfour-hi/github-ranking) |
| **品牌资源** | 原项目的 logo、字标、界面截图、字体文件**均已移除**（早期曾直接拷贝，现已全部删除）；界面图形改为自绘，背景改纯 CSS，字体改系统字体栈 |
| **许可证** | 原项目未声明许可证；本项目按 [MIT](./LICENSE) 发布，该许可证**仅覆盖本项目自身代码** |

原项目与本项目都是「客户端拿 GitHub 数据做展示」的工具，差异主要在技术栈与数据持久化方式，见文末[与原版本的差异](#与原版本的差异)。

## 免责声明

- **无关联**：本项目是 [cfour-hi/gitstars](https://github.com/cfour-hi/gitstars) 的**非官方重写版**，由第三方独立开发，与原项目作者**无任何隶属、合作或赞助关系**，**未获得其授权或认可**。
- **非 fork**：本项目未复用原项目的源代码。原项目未声明开源许可证，因此本项目**主动移除了原项目的全部品牌资源**（logo、字标、界面截图、字体），仅保留功能层面的设计参考。
- **商标**：「GitHub」名称与相关标志是 **GitHub, Inc.** 的注册商标，本项目与其无隶属关系，仅在「表示链接指向 GitHub」的语境下使用其图标。本项目名称不主张任何商标权。
- **数据与账号**：本项目只是 GitHub API 的客户端，不存储、不转发你的 Access Token 到任何第三方；但你需要自行确认其使用方式符合 [GitHub 服务条款](https://docs.github.com/site-policy/github-terms/github-terms-of-service)。
- **无担保**：本项目按「现状」提供，不附带任何明示或默示担保。因使用本项目产生的任何后果由使用者自行承担。

第三方图标与样式来源及各自许可证见 [NOTICE](./NOTICE)。

## 功能

- **Your Stars**：同步并浏览当前 GitHub 账号的 Starred Repositories。
- **分类筛选**：按 Topics 与主要编程语言自动归类。
- **快速搜索**：按开发者、仓库名、描述查找（300ms 防抖）。
- **灵活排序**：按 Star 时间或 Star 数量排序；tag 可按计数升/降序。
- **可展开描述**：仓库卡片描述默认折叠 2 行，可展开查看全文。
- **手动刷新**：一键强制跳过缓存重新拉取 Stars。
- **Gitstars Ranking**：按语言查看热门仓库 Top 100。
- **README 预览**：无需离开页面即可阅读仓库 README（含相对链接重写）。
- **中英文界面**：可在页面中切换。

## 技术栈

| 层 | 选型 |
| --- | --- |
| 框架 | Leptos 0.8（SSR + Hydration + server functions） |
| 服务端 | `leptos_axum` + `axum`（由 `cargo-leptos` 标准脚手架承载） |
| 样式 | Tailwind CSS v4（standalone CLI，无需 Node） |
| 序列化 | `serde` + `serde_json`（写）/ `simd-json`（读缓存） |
| 虚拟滚动 | tag 列表用社区 crate `leptos_virtual_scroller`；仓库列表用自研动态高度虚拟列表 |
| HTTP | `reqwest`（rustls） |
| 构建 | `cargo-leptos` |

## 架构

Leptos 全栈：所有业务逻辑（GitHub 调用、缓存、鉴权）都写在 `#[server]` 标注的 server function 里（`src/server_fn.rs`）。
前端通过 server function 取数据，不直接访问 `api.github.com`，因此：

- 无 CORS 问题；
- Access Token 只存在于服务端会话，浏览器只拿到 HttpOnly Cookie；
- 缓存命中时无需网络请求。

关键模块：

```
src/
├── app.rs            # App 外壳、路由、客户端初始化
├── server_fn.rs      # #[server] 函数（OAuth、stars、readme、ranking）
├── models.rs         # User / Repository / RankingData
├── state.rs          # 全局状态（信号）与派生逻辑
├── loaders.rs        # 前端加载动作
├── api/              # GitHub REST / 排行榜抓取
├── cache/            # JSON 缓存读写（原子写 + simd-json 解析）
├── stars.rs          # Stars 缓存策略
├── ranking.rs        # 排行榜缓存策略
├── auth.rs           # 服务端会话
├── readme.rs         # README 链接重写
└── components/       # UI 组件
```

## 数据持久化

为了「打开即用、不必每次请求 GitHub」，Stars 与排行榜数据会落盘为 JSON：

```
data/
├── stars/{login}.json          # 每个账号一份快照
└── ranking/
    ├── ranking.json            # 各语言仓库列表（含 all）
    └── languages.json          # 语言名列表
```

快照结构示例：

```jsonc
{
  "schema_version": 1,
  "login": "octocat",
  "fetched_at": 1790111854,       // Unix 秒
  "etag": "\"abc123\"",
  "repositories": [ /* 仓库对象 */ ]
}
```

刷新策略：

1. 打开时直接读缓存返回；
2. 超过 TTL（Stars 默认 15 分钟 / 排行榜 24 小时）时触发条件刷新；
3. 条件刷新带 `If-None-Match`，返回 `304` 则复用旧快照；
4. 前端「刷新」按钮可强制跳过 TTL 与 ETag 重新拉取；
5. 刷新失败时回退到旧缓存。

读取使用 `simd-json` 加速大文件反序列化（排行榜快照约 1.4MB），写入使用 `serde_json` 并采用「临时文件 + rename」原子替换。

## 环境变量

| 变量 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- |
| `GITSTARS_CLIENT_ID` | 是 | — | GitHub OAuth App Client ID |
| `GITHUB_CLIENT_SECRET` | 是 | — | GitHub OAuth App Client Secret（仅服务端） |
| `GITSTARS_DATA_DIR` | 否 | `./data` | 缓存目录 |
| `LEPTOS_SITE_ADDR` | 否 | `127.0.0.1:3000` | 监听地址（Docker 镜像内为 `0.0.0.0:8080`） |
| `GITSTARS_STARS_TTL_SECS` | 否 | `900` | Stars 缓存 TTL |
| `GITSTARS_RANKING_TTL_SECS` | 否 | `86400` | 排行榜缓存 TTL |
| `TLS_CERT_FILE` / `TLS_KEY_FILE` | 否 | — | 同时提供才启用 HTTPS |
| `GITSTARS_PORT` | 否 | `8080` | **仅 Docker**：映射到宿主机的端口 |
| `GITSTARS_CERT_DIR` | 否 | `./.certs` | **仅 Docker/脚本**：本地证书目录 |
| `GITSTARS_IMAGE_REPO` | 否 | `gitstars` | **仅构建/部署**：镜像仓库名，推送时需带前缀 |
| `GITSTARS_IMAGE_TAG` | 否 | `0.0.1` | **仅部署**：compose 运行哪个版本 |

> 项目根目录的 `.env` 会在启动时自动加载（`dotenvy`），已存在的真实环境变量优先，不会被覆盖。
> 缺少 `GITSTARS_CLIENT_ID` / `GITHUB_CLIENT_SECRET` 时启动日志会给出 `WARN` 提示。

## 本地开发

前置：

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
# Tailwind v4 standalone（放到 PATH 中即可，无需 Node）
curl -sL https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 \
  -o ~/.local/bin/tailwindcss && chmod +x ~/.local/bin/tailwindcss
```

启动：

```bash
cp .env.example .env      # 填入 Client ID / Secret
cargo leptos watch
```

默认访问 <http://127.0.0.1:3000>。

> ⚠️ 请用 **`127.0.0.1`** 或 `localhost` 打开，不要用 `0.0.0.0`。
> 登录链接里的 `redirect_uri` 取浏览器当前地址，用 `0.0.0.0` 会导致 GitHub 回调校验失败（表现为 404 或 `redirect_uri` 不匹配）。
> 代码会把 `0.0.0.0` 自动改写为 `127.0.0.1`，但仍建议直接访问 `127.0.0.1`。

### GitHub OAuth App

在 [GitHub Developer Settings](https://github.com/settings/developers) 新建 OAuth App：

- Homepage URL：`http://127.0.0.1:3000`
- Authorization callback URL：`http://127.0.0.1:3000`

回调地址必须与浏览器实际访问地址的**协议、主机、端口完全一致**。

一个 OAuth App 只能填 **一个** callback URL，因此：

- 本地开发：`http://127.0.0.1:3000`
- Docker（HTTPS）：`https://localhost:8080`

两者切换时需改回调地址，或建两个 OAuth App。

## Docker 部署

镜像采用「本机编译 + 精简运行时镜像」的方式，与 `opptrix-new` 的构建方式一致：
**Dockerfile 内不做任何编译**，只把宿主机编好的二进制与前端站点组装进运行时镜像。

```bash
cp .env.example .env              # 填入 Client ID / Secret
./scripts/build-image.sh          # 编译 + 构建镜像 + 冒烟测试
podman-compose up -d              # 启动（HTTP）
```

启用 HTTPS：

```bash
./scripts/setup-local-https.sh    # 需要 mkcert，生成 .certs/
podman-compose -f compose.yaml -f compose.tls.yaml up -d
```

访问 <http://localhost:8080>（或 HTTPS 下的 <https://localhost:8080>）。健康检查：`/healthz`。

### scripts/build-image.sh

```bash
scripts/build-image.sh                  # 编译 + 构建镜像（默认）
scripts/build-image.sh --skip-build     # 跳过编译（产物已存在）
scripts/build-image.sh --push           # 构建后推送到镜像仓库
scripts/build-image.sh --save           # 额外导出 tar 到 dist/
scripts/build-image.sh --version 1.2.3  # 指定版本（默认读 VERSION 文件）
scripts/build-image.sh --tag my/app:v1  # 完全自定义标签
scripts/build-image.sh --no-smoke       # 跳过容器冒烟
```

脚本会依次完成：

1. `cargo leptos build --release`（宿主机，复用 `target/` 缓存）；
2. **glibc 符号校验**：解析二进制的 `GLIBC_x.y` 上限，并与基础镜像的 glibc 对比，
   不满足就直接中止；
3. `podman build`（显式 `--format docker`，否则 OCI 格式会丢 `HEALTHCHECK`）；
4. **容器冒烟**：起容器轮询 `/healthz`，失败则打印日志并中止（不会推上去）；
5. 可选 `push` / `save`。

### 推送镜像

在 `.env` 里把仓库名换成带前缀的名字，然后推送：

```bash
# .env
GITSTARS_IMAGE_REPO=ghcr.io/<你的用户名>/gitstars
GITSTARS_IMAGE_TAG=0.0.1
```

```bash
scripts/build-image.sh --push
```

服务器上拉取并启动（把同样的 `GITSTARS_IMAGE_REPO` / `GITSTARS_IMAGE_TAG` 写进服务器的 `.env`）：

```bash
podman-compose pull && podman-compose up -d
```

没有仓库时也可以用 tar 搬运：`scripts/build-image.sh --save` → `podman load -i dist/gitstars-0.0.1.tar`。

### 一键部署到远端服务器

`scripts/deploy-server.sh` 把「同步版本号 → 构建 tar.gz → scp → md5 校验 → 远端 `podman load`」串成一条命令：

```bash
scripts/deploy-server.sh 0.0.1                # 构建 → 上传 → 加载
scripts/deploy-server.sh 0.0.1 --up           # 加载后再重启容器
scripts/deploy-server.sh 0.0.1 --up --tls     # 远端用 compose.tls.yaml 启动
scripts/deploy-server.sh 0.0.1 --skip-build   # 复用已存在的 tar.gz
```

可覆盖的环境变量：

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `DEPLOY_REMOTE` | `arch-server` | 远端 ssh 别名 |
| `DEPLOY_REMOTE_DIR` | `/opt/1panel/docker/compose/gitstars/` | 远端目标目录 |
| `DEPLOY_MAX_RETRY` | `5` | md5 不一致时的最大重传次数 |

两个设计点：

- **不上传 `.env`**：远端 `.env` 由服务器自己维护（Client ID / Secret / 端口等），脚本只传镜像包和 compose 文件，不会覆盖服务器上的配置。
- **版本号内联传入**：`--up` 执行的是 `GITSTARS_IMAGE_TAG=0.0.1 podman-compose ... up -d`。shell 环境优先于远端 `.env`，所以服务器 `.env` 里的 tag 无需同步；脚本末尾打印的手工命令也带上了这个变量。

md5 校验会对比本地与远端 tar.gz，不一致自动重传（最多 `DEPLOY_MAX_RETRY` 次），避免网络抖动导致远端加载到半个包。

### 为什么基础镜像是 trixie 而不是 bookworm

产物对 glibc 的最低要求由**编译环境**决定。宿主机是 Arch（glibc 2.44），编出的二进制会带上
`GLIBC_2.38` 这类高版本符号；`bookworm-slim` 只有 glibc 2.36，二进制进容器后连加载器都过不去，
表现为容器每隔几秒重启一次、且没有任何应用日志。
因此运行时基镜像用 `debian:trixie-slim`（glibc 2.41），并由脚本在构建前强制校验。

### compose 文件

| 文件 | 用途 |
| --- | --- |
| `compose.yaml` | 基础部署：HTTP、只读根文件系统、`gitstars-data` 数据卷 |
| `compose.tls.yaml` | 覆盖文件：挂载 `.certs/` 只读证书并开启 TLS |

## 测试与检查

```bash
cargo test --lib                                   # 单元测试
cargo clippy --all-targets --features ssr -- -D warnings
cargo clippy --lib --no-default-features --features hydrate \
  --target wasm32-unknown-unknown -- -D warnings
```

## 与原版本的差异

- 前端由 Vue3 + Pinia 重写为 Leptos 信号/组件。
- 原先手写的 Node HTTPS 服务由 Leptos server function + 框架自带宿主替代，无需单独后端工程。
- Access Token 从浏览器 `localStorage` 迁移到服务端会话 + HttpOnly Cookie。
- Stars / 排行榜从「浏览器 localStorage 缓存」升级为「服务端 JSON 快照 + TTL + ETag 条件刷新」。
- 新增手动刷新按钮与仓库描述展开。

## 许可证

本项目自身代码按 **MIT** 许可证发布：

```
MIT License
Copyright (c) 2026 cirno99
```

完整文本见 [LICENSE](./LICENSE)。

### 覆盖范围

| | |
| --- | --- |
| ✅ 覆盖 | `src/`、`style/`、`assets/`、`scripts/`、`Dockerfile`、compose 文件、文档等本项目自行编写的部分 |
| ⚠️ 不覆盖 | 第三方图标（IconPark / Octicons）与 `style/github-markdown.css`（github-markdown-css），它们遵循各自的许可证，见 [NOTICE](./NOTICE) |

### 关于原项目

本项目是 [cfour-hi/gitstars](https://github.com/cfour-hi/gitstars) 的**独立重写版**，非 fork，与原项目作者无关。
原项目**未声明任何开源许可证**，因此本 MIT 许可证**不适用于原项目**，也不构成对原项目任何权利的授权。
为避免混淆，本项目已移除原项目的全部品牌资源（logo、字标、界面截图、字体），详见 [NOTICE](./NOTICE)。