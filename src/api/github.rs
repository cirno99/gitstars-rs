//! GitHub REST API 客户端（服务端使用）。

use serde::Deserialize;

use crate::models::{Repository, User};

const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "gitstars";

/// GitHub 调用错误。
#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("GitHub 未授权（401）")]
    Unauthorized,
    #[error("GitHub 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("GitHub 返回异常状态: {0}")]
    Status(u16),
    #[error("响应解析失败: {0}")]
    Decode(String),
}

/// 一页星标仓库的响应。
pub struct StarredPage {
    pub repositories: Vec<Repository>,
    pub etag: Option<String>,
    pub not_modified: bool,
}

/// README 接口响应。
#[derive(Debug, Deserialize)]
pub struct ReadmeResponse {
    /// base64 编码的内容。
    pub content: String,
    /// README 的 GitHub 页面地址，用于推导相对链接前缀。
    pub html_url: String,
}

pub struct GithubClient {
    client: reqwest::Client,
    token: String,
}

impl GithubClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap_or_default(),
            token: token.into(),
        }
    }

    async fn get(&self, url: &str, etag: Option<&str>) -> Result<reqwest::Response, GithubError> {
        let mut req = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .bearer_auth(&self.token);
        if let Some(etag) = etag {
            req = req.header("If-None-Match", etag);
        }
        let res = req.send().await?;
        match res.status().as_u16() {
            401 => Err(GithubError::Unauthorized),
            304 => Ok(res),
            status if (200..300).contains(&status) => Ok(res),
            status => Err(GithubError::Status(status)),
        }
    }

    /// `GET /user`
    pub async fn user(&self) -> Result<User, GithubError> {
        let res = self.get(&format!("{API_BASE}/user"), None).await?;
        res.json()
            .await
            .map_err(|e| GithubError::Decode(e.to_string()))
    }

    /// `GET /user/starred`，支持 ETag 条件请求。
    pub async fn starred_page(
        &self,
        page: u32,
        per_page: u32,
        etag: Option<&str>,
    ) -> Result<StarredPage, GithubError> {
        let url = format!("{API_BASE}/user/starred?page={page}&per_page={per_page}");
        let res = self.get(&url, etag).await?;
        if res.status().as_u16() == 304 {
            return Ok(StarredPage {
                repositories: Vec::new(),
                etag: None,
                not_modified: true,
            });
        }
        let etag = res
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let repositories = res
            .json()
            .await
            .map_err(|e| GithubError::Decode(e.to_string()))?;
        Ok(StarredPage {
            repositories,
            etag,
            not_modified: false,
        })
    }

    /// `GET /repos/{owner}/{name}/readme`
    pub async fn readme(&self, owner: &str, name: &str) -> Result<ReadmeResponse, GithubError> {
        let url = format!("{API_BASE}/repos/{owner}/{name}/readme");
        let res = self.get(&url, None).await?;
        res.json()
            .await
            .map_err(|e| GithubError::Decode(e.to_string()))
    }

    /// `POST /markdown`：把 Markdown 渲染成 HTML。
    pub async fn render_markdown(&self, text: &str) -> Result<String, GithubError> {
        let res = self
            .client
            .post(format!("{API_BASE}/markdown"))
            .header("Accept", "application/vnd.github+json")
            .bearer_auth(&self.token)
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(GithubError::Status(res.status().as_u16()));
        }
        res.text()
            .await
            .map_err(|e| GithubError::Decode(e.to_string()))
    }
}
