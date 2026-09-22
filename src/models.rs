use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// GitHub 用户信息（仅保留界面用到的字段）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub html_url: String,
}

/// 仓库所有者（仅保留界面用到的字段）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoOwner {
    pub login: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub avatar_url: String,
}

/// GitHub 仓库对象（仅保留界面用到的字段）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub owner: RepoOwner,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub stargazers_count: i64,
    #[serde(default)]
    pub forks_count: i64,
    /// 排行榜名次（1..100），仅排行榜数据带此字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking: Option<u32>,
}

/// 排行榜数据：语言列表 + 各语言下的仓库。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RankingData {
    /// 语言名列表（不含 `all`）。
    #[serde(default)]
    pub languages: Vec<String>,
    /// `language -> [Repository]`，包含 `all` 键。
    #[serde(default)]
    pub repos: HashMap<String, Vec<Repository>>,
}

impl RankingData {
    /// 所有语言仓库去重后的集合（对应原 `rankingStore.repositories`）。
    pub fn all_repositories(&self) -> Vec<Repository> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for repos in self.repos.values() {
            for repo in repos {
                if seen.insert(repo.id) {
                    out.push(repo.clone());
                }
            }
        }
        out
    }
}
