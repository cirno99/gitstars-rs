//! 全局状态（对应原项目 4 个 Pinia store）。

use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::i18n::Lang;
use crate::models::{RankingData, Repository, User};

/// tag 来源：自己的星标 / GitHub 排行榜。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagSrc {
    Star,
    Ranking,
}

/// tag 类型：topic / language。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    Topic,
    Language,
}

/// tag 计数排序。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascend,
    Descend,
}

/// 仓库排序。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoSort {
    Time,
    Star,
}

/// 主题偏好。
///
/// `System` 表示跟随系统配色，具体深浅由 `prefers-color-scheme` 决定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn from_code(code: &str) -> Self {
        match code {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    /// sprite 里的图标名（去掉 `icon-` 前缀）。
    pub fn icon(self) -> &'static str {
        match self {
            Theme::System => "monitor",
            Theme::Light => "sun",
            Theme::Dark => "moon",
        }
    }

    /// 点击后的下一个状态：跟随系统 → 浅色 → 深色 → 跟随系统。
    pub fn next(self) -> Self {
        match self {
            Theme::System => Theme::Light,
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::System,
        }
    }

    /// 实际写入 `<html data-theme>` 的值。
    pub fn resolve(self, prefers_dark: bool) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System if prefers_dark => "dark",
            Theme::System => "light",
        }
    }
}

/// 全局状态集合，通过 context 在组件间共享。
#[derive(Clone, Copy)]
pub struct AppCtx {
    pub lang: RwSignal<Lang>,
    pub theme: RwSignal<Theme>,
    pub user: RwSignal<Option<User>>,
    pub auth_checked: RwSignal<bool>,
    pub login_url: RwSignal<String>,
    pub stars: RwSignal<Vec<Repository>>,
    pub stars_loading: RwSignal<bool>,
    pub stars_error: RwSignal<Option<String>>,
    pub ranking: RwSignal<RankingData>,
    pub ranking_loading: RwSignal<bool>,
    pub selected_id: RwSignal<Option<i64>>,
    pub repo_filter: RwSignal<String>,
    pub repo_sort: RwSignal<RepoSort>,
    pub tag_src: RwSignal<TagSrc>,
    pub selected_tag: RwSignal<String>,
    pub selected_tag_type: RwSignal<TagType>,
    pub tag_nav: RwSignal<TagType>,
    pub tag_filter: RwSignal<String>,
    pub tag_sort: RwSignal<SortOrder>,
    pub selected_language: RwSignal<String>,
    pub ranking_filter: RwSignal<String>,
    /// 当前展开描述的仓库 id（同一时刻最多一个）。
    pub expanded: RwSignal<Option<i64>>,
}

impl AppCtx {
    pub fn new() -> Self {
        Self {
            lang: RwSignal::new(Lang::Zh),
            theme: RwSignal::new(Theme::System),
            user: RwSignal::new(None),
            auth_checked: RwSignal::new(false),
            login_url: RwSignal::new(String::new()),
            stars: RwSignal::new(Vec::new()),
            stars_loading: RwSignal::new(true),
            stars_error: RwSignal::new(None),
            ranking: RwSignal::new(RankingData::default()),
            ranking_loading: RwSignal::new(false),
            selected_id: RwSignal::new(None),
            repo_filter: RwSignal::new(String::new()),
            repo_sort: RwSignal::new(RepoSort::Time),
            tag_src: RwSignal::new(TagSrc::Star),
            selected_tag: RwSignal::new(String::new()),
            selected_tag_type: RwSignal::new(TagType::Topic),
            tag_nav: RwSignal::new(TagType::Topic),
            tag_filter: RwSignal::new(String::new()),
            tag_sort: RwSignal::new(SortOrder::Descend),
            selected_language: RwSignal::new(String::new()),
            ranking_filter: RwSignal::new(String::new()),
            expanded: RwSignal::new(None),
        }
    }
}

impl Default for AppCtx {
    fn default() -> Self {
        Self::new()
    }
}

/// 提供全局状态并返回句柄。
pub fn provide_app_ctx() -> AppCtx {
    let ctx = AppCtx::new();
    provide_context(ctx);
    ctx
}

/// 取用全局状态。
pub fn app_ctx() -> AppCtx {
    use_context::<AppCtx>().expect("AppCtx 未通过 provide_context 提供")
}

/// 统计 Topics：`topic -> [repo id]`。
pub fn topic_map(repos: &[Repository]) -> BTreeMap<String, Vec<i64>> {
    let mut map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for repo in repos {
        for topic in &repo.topics {
            map.entry(topic.clone()).or_default().push(repo.id);
        }
    }
    map
}

/// 统计 Languages：`language -> [repo id]`。
pub fn language_map(repos: &[Repository]) -> BTreeMap<String, Vec<i64>> {
    let mut map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for repo in repos {
        if let Some(language) = &repo.language {
            map.entry(language.clone()).or_default().push(repo.id);
        }
    }
    map
}

/// tag 列表（带计数），按搜索词过滤并按计数排序。
pub fn tag_list(
    map: &BTreeMap<String, Vec<i64>>,
    filter: &str,
    sort: SortOrder,
) -> Vec<(String, usize)> {
    let filter = filter.to_lowercase();
    let mut list: Vec<(String, usize)> = map
        .iter()
        .filter(|(name, _)| filter.is_empty() || name.to_lowercase().contains(&filter))
        .map(|(name, ids)| (name.clone(), ids.len()))
        .collect();
    match sort {
        SortOrder::Ascend => list.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))),
        SortOrder::Descend => list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))),
    }
    list
}

/// 当前条件下应展示的仓库列表。
pub fn filtered_repositories(ctx: &AppCtx) -> Vec<Repository> {
    let tag_src = ctx.tag_src.get();
    let stars = ctx.stars.get();
    let ranking = ctx.ranking.get();
    let selected_tag = ctx.selected_tag.get();
    let selected_tag_type = ctx.selected_tag_type.get();
    let selected_language = ctx.selected_language.get();
    let filter = ctx.repo_filter.get();
    let sort = ctx.repo_sort.get();

    let mut list: Vec<Repository> = match tag_src {
        TagSrc::Star => {
            if selected_tag.is_empty() {
                stars.clone()
            } else {
                let map = match selected_tag_type {
                    TagType::Topic => topic_map(&stars),
                    TagType::Language => language_map(&stars),
                };
                match map.get(&selected_tag) {
                    Some(ids) => stars
                        .iter()
                        .filter(|r| ids.contains(&r.id))
                        .cloned()
                        .collect(),
                    None => Vec::new(),
                }
            }
        }
        TagSrc::Ranking => {
            if selected_language.is_empty() {
                ranking.repos.get("all").cloned().unwrap_or_default()
            } else {
                ranking
                    .repos
                    .get(&selected_language)
                    .cloned()
                    .unwrap_or_default()
            }
        }
    };

    if !filter.is_empty() {
        let filter = filter.to_lowercase();
        list.retain(|repo| {
            repo.owner.login.to_lowercase().contains(&filter)
                || repo.name.to_lowercase().contains(&filter)
                || repo
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
        });
    }

    if tag_src == TagSrc::Star && sort == RepoSort::Star {
        list.sort_by_key(|repo| std::cmp::Reverse(repo.stargazers_count));
    }

    list
}

/// 当前选中的仓库。
pub fn selected_repository(ctx: &AppCtx) -> Option<Repository> {
    let id = ctx.selected_id.get()?;
    ctx.stars
        .get()
        .into_iter()
        .find(|r| r.id == id)
        .or_else(|| {
            ctx.ranking
                .get()
                .all_repositories()
                .into_iter()
                .find(|r| r.id == id)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RepoOwner;

    fn repo(id: i64, lang: Option<&str>, topics: &[&str], stars: i64) -> Repository {
        Repository {
            id,
            name: format!("repo{id}"),
            owner: RepoOwner {
                login: "owner".into(),
                html_url: String::new(),
                avatar_url: String::new(),
            },
            description: None,
            html_url: String::new(),
            homepage: None,
            language: lang.map(str::to_owned),
            topics: topics.iter().map(|t| (*t).to_owned()).collect(),
            stargazers_count: stars,
            forks_count: 0,
            ranking: None,
        }
    }

    #[test]
    fn topic_map_groups_ids() {
        let repos = vec![
            repo(1, None, &["cli", "rust"], 0),
            repo(2, None, &["cli"], 0),
        ];
        let map = topic_map(&repos);
        assert_eq!(map.get("cli").unwrap(), &vec![1, 2]);
        assert_eq!(map.get("rust").unwrap(), &vec![1]);
    }

    #[test]
    fn language_map_skips_none() {
        let repos = vec![repo(1, Some("Rust"), &[], 0), repo(2, None, &[], 0)];
        let map = language_map(&repos);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("Rust").unwrap(), &vec![1]);
    }

    #[test]
    fn tag_list_sorts_by_count_descending() {
        let mut map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
        map.insert("a".into(), vec![1]);
        map.insert("b".into(), vec![1, 2, 3]);
        let list = tag_list(&map, "", SortOrder::Descend);
        assert_eq!(list[0].0, "b");
        assert_eq!(list[0].1, 3);
    }

    #[test]
    fn tag_list_filters_by_substring() {
        let mut map: BTreeMap<String, Vec<i64>> = BTreeMap::new();
        map.insert("Rust".into(), vec![1]);
        map.insert("Go".into(), vec![2]);
        let list = tag_list(&map, "ru", SortOrder::Descend);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "Rust");
    }
}
