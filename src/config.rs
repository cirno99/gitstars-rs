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
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    client_id: std::env::var("GITSTARS_CLIENT_ID").unwrap_or_default(),
    client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
    data_dir: std::env::var("GITSTARS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data")),
});
/// 当前 Unix 时间戳（秒）。
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
