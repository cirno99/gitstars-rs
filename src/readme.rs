//! README 相对链接重写（与原项目 `repository-content/tool.js` 行为一致）。

use std::sync::LazyLock;

use regex::Regex;

static ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)(href|src)="([^"]+)""#).expect("正则编译失败"));

/// 把 README HTML 中的相对 `href` / `src` 补全为 GitHub 上的绝对地址。
///
/// * `url_prefix` 形如 `https://github.com/owner/repo/blob/main/`
/// * `src` 属性额外把 `/blob/` 换成 `/raw/`
pub fn rewrite_urls(html: &str, url_prefix: &str) -> String {
    ATTR_RE
        .replace_all(html, |caps: &regex::Captures<'_>| {
            let attr = &caps[1];
            let url = &caps[2];
            let rewritten = rewrite_one(url, url_prefix);
            let rewritten = if attr.eq_ignore_ascii_case("src") {
                rewritten.replace("/blob/", "/raw/")
            } else {
                rewritten
            };
            format!("{attr}=\"{rewritten}\"")
        })
        .into_owned()
}

fn rewrite_one(url: &str, url_prefix: &str) -> String {
    if url.starts_with("http") {
        return url.to_owned();
    }
    if let Some(rest) = url.strip_prefix("./") {
        return format!("{url_prefix}{rest}");
    }
    format!("{url_prefix}{url}")
}

/// 由 README 的 `html_url` 推导相对链接前缀。
///
/// `https://github.com/owner/repo/blob/main/README.md` → `https://github.com/owner/repo/blob/main/`
pub fn url_prefix_from_readme(html_url: &str) -> String {
    html_url
        .strip_suffix("README.md")
        .unwrap_or(html_url)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = "https://github.com/owner/repo/blob/main/";

    #[test]
    fn absolute_urls_are_untouched() {
        let html = r#"<a href="https://example.com/a">x</a>"#;
        assert_eq!(rewrite_urls(html, PREFIX), html);
    }

    #[test]
    fn relative_href_is_prefixed() {
        let html = r#"<a href="./docs/a.md">x</a>"#;
        let expected = format!(r#"<a href="{PREFIX}docs/a.md">x</a>"#);
        assert_eq!(rewrite_urls(html, PREFIX), expected);
    }

    #[test]
    fn bare_relative_href_is_prefixed() {
        let html = r#"<a href="docs/a.md">x</a>"#;
        let expected = format!(r#"<a href="{PREFIX}docs/a.md">x</a>"#);
        assert_eq!(rewrite_urls(html, PREFIX), expected);
    }

    #[test]
    fn src_blob_is_rewritten_to_raw() {
        let html = r#"<img src="docs/a.png"/>"#;
        let expected = r#"<img src="https://github.com/owner/repo/raw/main/docs/a.png"/>"#;
        assert_eq!(rewrite_urls(html, PREFIX), expected);
    }

    #[test]
    fn url_prefix_strips_readme_suffix() {
        assert_eq!(
            url_prefix_from_readme("https://github.com/o/r/blob/main/README.md"),
            "https://github.com/o/r/blob/main/"
        );
        assert_eq!(
            url_prefix_from_readme("https://github.com/o/r/blob/main/"),
            "https://github.com/o/r/blob/main/"
        );
    }
}
