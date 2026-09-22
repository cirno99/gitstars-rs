pub mod github;
pub mod ranking;

/// 构造通用 HTTP 客户端。
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("gitstars")
        .build()
        .unwrap_or_default()
}
