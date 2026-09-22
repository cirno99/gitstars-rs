//! 星标仓库的「缓存优先 + 条件刷新」逻辑。

use serde::{Deserialize, Serialize};

use crate::api::github::{GithubClient, GithubError};
use crate::auth::Session;
use crate::cache::{self, SCHEMA_VERSION};
use crate::config::{CONFIG, now_secs};
use crate::models::Repository;

/// 落盘的 Stars 快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarsCache {
    pub schema_version: u32,
    pub login: String,
    /// 抓取时间（Unix 秒）。
    pub fetched_at: i64,
    /// 第一页的 ETag。
    #[serde(default)]
    pub etag: Option<String>,
    pub repositories: Vec<Repository>,
}

/// Stars 加载错误。
#[derive(Debug, thiserror::Error)]
pub enum StarsError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("{0}")]
    Other(String),
}

enum FetchOutcome {
    NotModified,
    Fresh {
        repositories: Vec<Repository>,
        etag: Option<String>,
    },
}

const PER_PAGE: u32 = 100;

/// 读取 Stars：命中新鲜缓存直接返回，否则条件刷新，刷新失败时回退旧缓存。
pub async fn load(session: &Session, force: bool) -> Result<Vec<Repository>, StarsError> {
    let path = cache::stars_path(&session.user.login);
    let cached: Option<StarsCache> = cache::read_json_async(path.clone()).await;
    let now = now_secs();

    if let Some(cache) = &cached {
        let fresh = cache.schema_version == SCHEMA_VERSION
            && !force
            && now.saturating_sub(cache.fetched_at) < CONFIG.stars_ttl;
        if fresh {
            return Ok(cache.repositories.clone());
        }
    }

    let client = GithubClient::new(session.token.clone());
    let cached_etag = cached.as_ref().and_then(|c| c.etag.as_deref());

    match fetch_all(&client, cached_etag).await {
        Ok(FetchOutcome::NotModified) => match cached {
            Some(mut cache) => {
                cache.fetched_at = now;
                if let Err(e) = cache::write_json_async(path, cache.clone()).await {
                    tracing::warn!("写入 Stars 缓存失败: {e}");
                }
                Ok(cache.repositories)
            }
            None => Err(StarsError::Other("缓存缺失但返回 304".into())),
        },
        Ok(FetchOutcome::Fresh { repositories, etag }) => {
            let snapshot = StarsCache {
                schema_version: SCHEMA_VERSION,
                login: session.user.login.clone(),
                fetched_at: now,
                etag,
                repositories: repositories.clone(),
            };
            if let Err(e) = cache::write_json_async(path, snapshot).await {
                tracing::warn!("写入 Stars 缓存失败: {e}");
            }
            Ok(repositories)
        }
        Err(GithubError::Unauthorized) => Err(StarsError::Unauthorized),
        Err(e) => match cached {
            Some(cache) => {
                tracing::warn!("刷新 Stars 失败，回退旧缓存: {e}");
                Ok(cache.repositories)
            }
            None => Err(StarsError::Other(e.to_string())),
        },
    }
}

async fn fetch_all(client: &GithubClient, etag: Option<&str>) -> Result<FetchOutcome, GithubError> {
    let mut all = Vec::new();
    let mut page = 1;
    let mut first_etag = None;

    loop {
        let resp = client
            .starred_page(page, PER_PAGE, if page == 1 { etag } else { None })
            .await?;
        if page == 1 {
            if resp.not_modified {
                return Ok(FetchOutcome::NotModified);
            }
            first_etag = resp.etag.clone();
        }
        let count = resp.repositories.len();
        all.extend(resp.repositories);
        if count < PER_PAGE as usize {
            break;
        }
        page += 1;
    }

    Ok(FetchOutcome::Fresh {
        repositories: all,
        etag: first_etag,
    })
}
