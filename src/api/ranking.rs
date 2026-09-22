//! 拉取 `cfour-hi/github-ranking` 的排行榜数据（服务端使用）。

use serde::de::DeserializeOwned;

use super::github::GithubError;

pub const LANGUAGES_URL: &str =
    "https://raw.githubusercontent.com/cfour-hi/github-ranking/main/languages.json";
pub const RANKING_URL: &str =
    "https://raw.githubusercontent.com/cfour-hi/github-ranking/main/ranking.json";

const USER_AGENT: &str = "gitstars";

/// 带 ETag 的一次抓取结果。
pub struct Fetched<T> {
    pub data: Option<T>,
    pub etag: Option<String>,
    pub not_modified: bool,
}

/// 抓取 JSON，支持 `If-None-Match`。`data` 为 `None` 表示 304。
pub async fn fetch_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
) -> Result<Fetched<T>, GithubError> {
    let mut req = client.get(url).header("User-Agent", USER_AGENT);
    if let Some(etag) = etag {
        req = req.header("If-None-Match", etag);
    }
    let res = req.send().await?;
    if res.status().as_u16() == 304 {
        return Ok(Fetched {
            data: None,
            etag: None,
            not_modified: true,
        });
    }
    if !res.status().is_success() {
        return Err(GithubError::Status(res.status().as_u16()));
    }
    let etag = res
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let data = res
        .json()
        .await
        .map_err(|e| GithubError::Decode(e.to_string()))?;
    Ok(Fetched {
        data: Some(data),
        etag,
        not_modified: false,
    })
}

/// 构造用于排行榜抓取的 HTTP 客户端。
pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_default()
}
