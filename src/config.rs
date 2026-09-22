use std::path::PathBuf;
use std::sync::LazyLock;

/// 运行时配置，全部来自环境变量。
#[derive(Debug)]
pub struct Config {
    /// GitHub OAuth App Client ID。
    pub client_id: String,
    /// GitHub OAuth App Client Secret（仅服务端持有）。
    pub client_secret: String,
    /// 缓存目录。
    pub data_dir: PathBuf,
    /// Stars 缓存有效期（秒）。
    pub stars_ttl: i64,
    /// 排行榜缓存有效期（秒）。
    pub ranking_ttl: i64,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    client_id: std::env::var("GITSTARS_CLIENT_ID").unwrap_or_default(),
    client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
    data_dir: std::env::var("GITSTARS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data")),
    stars_ttl: parse_env_i64("GITSTARS_STARS_TTL_SECS", 900),
    ranking_ttl: parse_env_i64("GITSTARS_RANKING_TTL_SECS", 86_400),
});

fn parse_env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// 当前 Unix 时间戳（秒）。
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
