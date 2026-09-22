#![recursion_limit = "512"]

use gitstars::app::{App, shell};
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};

#[tokio::main]
async fn main() {
    // 先加载项目根目录的 `.env`（无论通过 cargo-leptos、Docker 还是直接运行二进制）。
    // 已存在的真实环境变量优先级更高，不会被覆盖。
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let conf = get_configuration(None).expect("读取 Leptos 配置失败");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    warn_if_oauth_unconfigured();

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .leptos_routes(&leptos_options, routes, {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    match (
        std::env::var("TLS_CERT_FILE").ok(),
        std::env::var("TLS_KEY_FILE").ok(),
    ) {
        (Some(cert_file), Some(key_file)) => {
            let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_file, key_file)
                .await
                .expect("加载 TLS 证书失败");
            tracing::info!("Gitstars 已启动（HTTPS）: https://{addr}");
            axum_server::bind_rustls(addr, tls)
                .serve(app.into_make_service())
                .await
                .expect("服务器运行失败");
        }
        _ => {
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .unwrap_or_else(|e| panic!("监听 {addr} 失败: {e}"));
            tracing::info!("Gitstars 已启动: http://{addr}");
            axum::serve(listener, app.into_make_service())
                .await
                .expect("服务器运行失败");
        }
    }
}

/// 缺少 OAuth 凭据时给出明确提示，避免登录时只看到一个空的 `client_id=`。
fn warn_if_oauth_unconfigured() {
    let cfg = &*gitstars::config::CONFIG;
    if cfg.client_id.is_empty() {
        tracing::warn!(
            "未配置 GITSTARS_CLIENT_ID：登录链接会是空的。请在项目根目录的 .env 中填写 GitHub OAuth App 的 Client ID"
        );
    }
    if cfg.client_secret.is_empty() {
        tracing::warn!("未配置 GITHUB_CLIENT_SECRET：OAuth 回调换取 token 会失败");
    }
}

async fn healthz() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
