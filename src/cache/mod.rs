//! 本地 JSON 缓存：写入用 `serde_json`，读取用 `simd-json` 加速解析。
//!
//! 采用「临时文件 + rename」的原子写入，避免进程中断留下半截文件。

use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::config::CONFIG;

/// 缓存 schema 版本，结构变更时递增即可让旧缓存失效。
pub const SCHEMA_VERSION: u32 = 1;

/// 数据根目录。
pub fn data_dir() -> PathBuf {
    CONFIG.data_dir.clone()
}

/// 某个账号的 Stars 缓存路径。
pub fn stars_path(login: &str) -> PathBuf {
    data_dir().join("stars").join(format!("{login}.json"))
}

/// 排行榜仓库缓存路径。
pub fn ranking_repos_path() -> PathBuf {
    data_dir().join("ranking").join("ranking.json")
}

/// 排行榜语言列表缓存路径。
pub fn ranking_languages_path() -> PathBuf {
    data_dir().join("ranking").join("languages.json")
}

/// 同步读取并解析 JSON；文件缺失或损坏时返回 `None`（不 panic）。
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let mut bytes = std::fs::read(path).ok()?;
    simd_json::serde::from_slice::<T>(&mut bytes).ok()
}

/// 同步原子写入 JSON。
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = tmp_path(path);
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// 异步读取，文件 IO 放到阻塞线程池。
pub async fn read_json_async<T>(path: PathBuf) -> Option<T>
where
    T: DeserializeOwned + Send + 'static,
{
    tokio::task::spawn_blocking(move || read_json::<T>(&path))
        .await
        .ok()
        .flatten()
}

/// 异步写入，文件 IO 放到阻塞线程池。
pub async fn write_json_async<T>(path: PathBuf, value: T) -> io::Result<()>
where
    T: Serialize + Send + 'static,
{
    tokio::task::spawn_blocking(move || write_json(&path, &value))
        .await
        .map_err(io::Error::other)?
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        schema_version: u32,
        name: String,
        values: Vec<i64>,
    }

    fn temp_file(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let unique = format!(
            "gitstars-cache-{tag}-{}-{}.json",
            std::process::id(),
            crate::config::now_secs()
        );
        p.push(unique);
        p
    }

    #[test]
    fn write_then_read_round_trips() {
        let path = temp_file("roundtrip");
        let sample = Sample {
            schema_version: SCHEMA_VERSION,
            name: "hello".into(),
            values: vec![1, 2, 3],
        };
        write_json(&path, &sample).expect("写入成功");
        let loaded: Sample = read_json(&path).expect("读取成功");
        assert_eq!(loaded, sample);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_missing_file_returns_none() {
        let path = temp_file("missing");
        let loaded: Option<Sample> = read_json(&path);
        assert!(loaded.is_none());
    }

    #[test]
    fn read_corrupt_file_returns_none() {
        let path = temp_file("corrupt");
        std::fs::write(&path, b"{ not json").unwrap();
        let loaded: Option<Sample> = read_json(&path);
        assert!(loaded.is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn write_is_atomic_leaves_no_tmp_file() {
        let path = temp_file("atomic");
        write_json(
            &path,
            &Sample {
                schema_version: SCHEMA_VERSION,
                name: "x".into(),
                values: vec![],
            },
        )
        .unwrap();
        assert!(!tmp_path(&path).exists());
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }
}
