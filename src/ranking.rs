//! 排行榜的「缓存优先」逻辑：只读快照，手动刷新才访问网络。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::ranking::{self, Fetched};
use crate::cache::{self, SCHEMA_VERSION};
use crate::config::now_secs;
use crate::models::{RankingData, Repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RankingReposCache {
    schema_version: u32,
    fetched_at: i64,
    #[serde(default)]
    etag: Option<String>,
    data: HashMap<String, Vec<Repository>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguagesCache {
    schema_version: u32,
    fetched_at: i64,
    #[serde(default)]
    etag: Option<String>,
    data: Vec<String>,
}

/// 两个快照都可用（schema 匹配）时返回其数据。
///
/// 抽成独立函数，避免 `if !force { if let ... { if let ... { ... } } }` 三层嵌套。
fn usable_cache(
    repos: Option<&RankingReposCache>,
    langs: Option<&LanguagesCache>,
) -> Option<RankingData> {
    let (repos, langs) = (repos?, langs?);
    if repos.schema_version != SCHEMA_VERSION || langs.schema_version != SCHEMA_VERSION {
        return None;
    }
    Some(RankingData {
        languages: langs.data.clone(),
        repos: repos.data.clone(),
    })
}

/// 读取排行榜：有可用快照直接返回，否则（或 `force` 时）访问网络。
///
/// 不在后台做任何隐式刷新：只有用户手动触发（`force = true`）才会重新拉取。
pub async fn load(force: bool) -> Result<RankingData, String> {
    let repos_path = cache::ranking_repos_path();
    let langs_path = cache::ranking_languages_path();
    let cached_repos: Option<RankingReposCache> = cache::read_json_async(repos_path.clone()).await;
    let cached_langs: Option<LanguagesCache> = cache::read_json_async(langs_path.clone()).await;
    let now = now_secs();

    if !force {
        if let Some(data) = usable_cache(cached_repos.as_ref(), cached_langs.as_ref()) {
            return Ok(data);
        }
    }

    let client = ranking::build_client();
    let repos_fetched = ranking::fetch_json::<HashMap<String, Vec<Repository>>>(
        &client,
        ranking::RANKING_URL,
        cached_repos.as_ref().and_then(|c| c.etag.as_deref()),
    )
    .await;
    let langs_fetched = ranking::fetch_json::<Vec<String>>(
        &client,
        ranking::LANGUAGES_URL,
        cached_langs.as_ref().and_then(|c| c.etag.as_deref()),
    )
    .await;

    let repos_data = match repos_fetched {
        Ok(Fetched {
            not_modified: true, ..
        }) => cached_repos.as_ref().map(|c| c.data.clone()),
        Ok(Fetched { data, etag, .. }) => {
            if let Some(data) = data {
                let snapshot = RankingReposCache {
                    schema_version: SCHEMA_VERSION,
                    fetched_at: now,
                    etag,
                    data: data.clone(),
                };
                if let Err(e) = cache::write_json_async(repos_path, snapshot).await {
                    tracing::warn!("写入排行榜缓存失败: {e}");
                }
                Some(data)
            } else {
                cached_repos.as_ref().map(|c| c.data.clone())
            }
        }
        Err(e) => {
            tracing::warn!("拉取排行榜失败: {e}");
            cached_repos.as_ref().map(|c| c.data.clone())
        }
    };

    let langs_data = match langs_fetched {
        Ok(Fetched {
            not_modified: true, ..
        }) => cached_langs.as_ref().map(|c| c.data.clone()),
        Ok(Fetched { data, etag, .. }) => {
            if let Some(data) = data {
                let snapshot = LanguagesCache {
                    schema_version: SCHEMA_VERSION,
                    fetched_at: now,
                    etag,
                    data: data.clone(),
                };
                if let Err(e) = cache::write_json_async(langs_path, snapshot).await {
                    tracing::warn!("写入语言列表缓存失败: {e}");
                }
                Some(data)
            } else {
                cached_langs.as_ref().map(|c| c.data.clone())
            }
        }
        Err(e) => {
            tracing::warn!("拉取语言列表失败: {e}");
            cached_langs.as_ref().map(|c| c.data.clone())
        }
    };

    match (repos_data, langs_data) {
        (Some(repos), Some(languages)) => Ok(RankingData { languages, repos }),
        _ => Err("排行榜数据不可用".into()),
    }
}
