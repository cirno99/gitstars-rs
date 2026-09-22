//! 极简中英文文案。
//!
//! 文案为本项目自行撰写，未沿用原项目 `src/i18n/{zh,en}.js` 的文本。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Lang::En,
            _ => Lang::Zh,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Lang::Zh => Lang::En,
            Lang::En => Lang::Zh,
        }
    }

    pub fn all(self) -> &'static str {
        match self {
            Lang::Zh => "全部",
            Lang::En => "All",
        }
    }

    pub fn search(self) -> &'static str {
        match self {
            Lang::Zh => "搜索",
            Lang::En => "Search",
        }
    }

    pub fn readme_tip(self) -> &'static str {
        match self {
            Lang::Zh => "从左侧选一个仓库查看 README",
            Lang::En => "Select a repository on the left to preview its README",
        }
    }

    pub fn login_tip(self) -> &'static str {
        match self {
            Lang::Zh => "使用 GitHub 账号登录",
            Lang::En => "Sign in with GitHub",
        }
    }

    pub fn user_title(self, username: &str) -> String {
        match self {
            Lang::Zh => format!("{username} 收藏的仓库"),
            Lang::En => format!("{username}'s starred repositories"),
        }
    }

    pub fn sort_ascend(self) -> &'static str {
        match self {
            Lang::Zh => "升序",
            Lang::En => "Ascending",
        }
    }

    pub fn sort_descend(self) -> &'static str {
        match self {
            Lang::Zh => "降序",
            Lang::En => "Descending",
        }
    }

    pub fn category_star(self) -> &'static str {
        match self {
            Lang::Zh => "我的星标",
            Lang::En => "Your Stars",
        }
    }

    pub fn category_ranking(self) -> &'static str {
        match self {
            Lang::Zh => "热门榜单",
            Lang::En => "Trending",
        }
    }

    pub fn repo_filter_tip(self) -> &'static str {
        match self {
            Lang::Zh => "搜索作者、仓库名或描述",
            Lang::En => "Search author, name or description",
        }
    }

    pub fn repo_sort_star(self) -> &'static str {
        match self {
            Lang::Zh => "按 Star 数排序",
            Lang::En => "Sort by stars",
        }
    }

    pub fn repo_sort_time(self) -> &'static str {
        match self {
            Lang::Zh => "按收藏时间排序",
            Lang::En => "Sort by starred time",
        }
    }

    pub fn repo_updating(self) -> &'static str {
        match self {
            Lang::Zh => "正在同步…",
            Lang::En => "Syncing…",
        }
    }

    pub fn refresh(self) -> &'static str {
        match self {
            Lang::Zh => "刷新",
            Lang::En => "Refresh",
        }
    }

    pub fn expand(self) -> &'static str {
        match self {
            Lang::Zh => "展开全文",
            Lang::En => "Show more",
        }
    }

    pub fn collapse(self) -> &'static str {
        match self {
            Lang::Zh => "收起",
            Lang::En => "Show less",
        }
    }

    pub fn logout(self) -> &'static str {
        match self {
            Lang::Zh => "退出登录",
            Lang::En => "Logout",
        }
    }

    /// 列表为空时的提示。
    pub fn no_result(self) -> &'static str {
        match self {
            Lang::Zh => "没有匹配的仓库",
            Lang::En => "No matching repositories",
        }
    }

    /// 主题切换按钮的提示文案。
    pub fn theme_system(self) -> &'static str {
        match self {
            Lang::Zh => "主题：跟随系统",
            Lang::En => "Theme: system",
        }
    }

    pub fn theme_light(self) -> &'static str {
        match self {
            Lang::Zh => "主题：浅色",
            Lang::En => "Theme: light",
        }
    }

    pub fn theme_dark(self) -> &'static str {
        match self {
            Lang::Zh => "主题：深色",
            Lang::En => "Theme: dark",
        }
    }

    /// 未登录落地页的副标题。
    pub fn tagline(self) -> &'static str {
        match self {
            Lang::Zh => "整理你的 GitHub Stars，顺便看看大家在看什么",
            Lang::En => "Organize your GitHub Stars, and see what others are starring",
        }
    }

    /// 落地页免责声明。
    pub fn disclaimer(self) -> &'static str {
        match self {
            Lang::Zh => "非官方重写版 · 与 GitHub, Inc. 及原项目作者均无关联",
            Lang::En => {
                "Unofficial rewrite · Not affiliated with GitHub, Inc. or the original project's author"
            }
        }
    }
}
