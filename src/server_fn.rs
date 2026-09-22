//! 前后端共用的 server function。

use leptos::prelude::*;

use crate::models::{RankingData, Repository, User};

/// 生成 GitHub OAuth 授权地址。
#[server]
pub async fn login_url(origin: String) -> Result<String, ServerFnError> {
    let client_id = crate::config::CONFIG.client_id.clone();
    Ok(format!(
        "https://github.com/login/oauth/authorize?client_id={client_id}&redirect_uri={origin}&scope=public_repo"
    ))
}

/// 用 OAuth `code` 换取 token，建立服务端会话，并下发 HttpOnly Cookie。
#[server]
pub async fn exchange_code(code: String, secure: bool) -> Result<Option<User>, ServerFnError> {
    let cfg = &*crate::config::CONFIG;
    let http = crate::api::http_client();
    let res = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "code": code,
            "client_id": cfg.client_id,
            "client_secret": cfg.client_secret,
        }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = match data.get("access_token").and_then(|v| v.as_str()) {
        Some(token) => token.to_owned(),
        None => {
            let reason = data
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("oauth_failed");
            return Err(ServerFnError::new(format!("OAuth 失败: {reason}")));
        }
    };

    let client = crate::api::github::GithubClient::new(token.clone());
    let user = client
        .user()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let session_id = crate::auth::create_session(token, user.clone());
    set_cookie_header(crate::auth::session_cookie(&session_id, secure));

    Ok(Some(user))
}

/// 退出登录。
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    if let Some(session_id) = session_id_from_cookie().await {
        crate::auth::remove_session(&session_id);
    }
    set_cookie_header(crate::auth::clear_cookie(false));

    Ok(())
}

/// 下发 `Set-Cookie` 响应头；非 SSR 上下文下静默跳过。
#[cfg(feature = "ssr")]
fn set_cookie_header(raw: String) {
    use axum::http::header::{HeaderValue, SET_COOKIE};
    use leptos_axum::ResponseOptions;

    if let (Some(resp), Ok(value)) = (
        use_context::<ResponseOptions>(),
        HeaderValue::from_str(&raw),
    ) {
        resp.insert_header(SET_COOKIE, value);
    }
}

/// 从请求 Cookie 中取出会话 ID。
#[cfg(feature = "ssr")]
async fn session_id_from_cookie() -> Option<String> {
    use axum_extra::extract::cookie::CookieJar;

    let jar = leptos_axum::extract::<CookieJar>().await.ok()?;
    let cookie = jar.get(crate::auth::COOKIE_NAME)?;
    Some(cookie.value().to_owned())
}

/// 当前登录用户。
#[server]
pub async fn me() -> Result<Option<User>, ServerFnError> {
    Ok(current_session().await.map(|s| s.user))
}

/// 星标仓库（缓存优先）。
#[server]
pub async fn starred_repos(force: bool) -> Result<Vec<Repository>, ServerFnError> {
    let session = current_session()
        .await
        .ok_or_else(|| ServerFnError::new("unauthorized"))?;
    crate::stars::load(&session, force)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// 读取并渲染仓库 README。
#[server]
pub async fn readme(owner: String, name: String) -> Result<String, ServerFnError> {
    use base64::Engine;

    let session = current_session()
        .await
        .ok_or_else(|| ServerFnError::new("unauthorized"))?;
    let client = crate::api::github::GithubClient::new(session.token);
    let resp = client
        .readme(&owner, &name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let compact: String = resp.content.split_whitespace().collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(compact)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let text = String::from_utf8(bytes).map_err(|e| ServerFnError::new(e.to_string()))?;

    let html = client
        .render_markdown(&text)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let prefix = crate::readme::url_prefix_from_readme(&resp.html_url);
    Ok(crate::readme::rewrite_urls(&html, &prefix))
}

/// 排行榜数据（缓存优先）。
#[server]
pub async fn ranking(force: bool) -> Result<RankingData, ServerFnError> {
    crate::ranking::load(force)
        .await
        .map_err(ServerFnError::new)
}

/// 从 Cookie 读取当前会话（仅服务端可用）。
#[cfg(feature = "ssr")]
async fn current_session() -> Option<crate::auth::Session> {
    use axum_extra::extract::cookie::CookieJar;

    let jar = leptos_axum::extract::<CookieJar>().await.ok()?;
    let session_id = jar.get(crate::auth::COOKIE_NAME)?.value().to_owned();
    crate::auth::get_session(&session_id)
}
