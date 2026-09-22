//! 品牌与本地存储常量。
//!
//! 注意：本项目是独立重写版，与原项目 `cfour-hi/gitstars` 无任何关联，
//! 这里不再保留原作者的署名常量。

/// 本项目名称（仅作为界面标题使用）。
pub const BRAND: &str = "gitstars";

/// 本项目仓库地址。
///
/// 顶栏的 GitHub 图标指向这里。**部署前请改成你自己的仓库**，
/// 不要指向原项目仓库，否则会被误认为原作者的项目。
pub const BRAND_URI: &str = "https://github.com/cirno99/gitstars-rs";

/// GitHub 站点根地址。
pub const GITHUB_COM: &str = "https://github.com";

/// 语言本地存储键（带 `_rs` 后缀，避免与原项目在同一 origin 下串味）。
pub const LANG_KEY: &str = "gitstars_rs_lang";

/// 主题本地存储键。
pub const THEME_KEY: &str = "gitstars_rs_theme";
