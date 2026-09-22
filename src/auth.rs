//! 服务端会话：Access Token 只保存在服务端内存，浏览器只拿到 HttpOnly Cookie。

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::models::User;

/// 会话 Cookie 名。
pub const COOKIE_NAME: &str = "gitstars_session";

/// 一个已登录会话。
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub user: User,
}

static SESSIONS: LazyLock<Mutex<HashMap<String, Session>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lock() -> std::sync::MutexGuard<'static, HashMap<String, Session>> {
    // 会话表不会出现跨线程不一致，锁中毒时直接复用内部数据。
    SESSIONS.lock().unwrap_or_else(|e| e.into_inner())
}

/// 创建会话并返回会话 ID。
pub fn create_session(token: String, user: User) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    lock().insert(id.clone(), Session { token, user });
    id
}

/// 读取会话。
pub fn get_session(id: &str) -> Option<Session> {
    lock().get(id).cloned()
}

/// 删除会话。
pub fn remove_session(id: &str) {
    lock().remove(id);
}

/// 构造会话 Cookie 的 `Set-Cookie` 值。
pub fn session_cookie(session_id: &str, secure: bool) -> String {
    let mut cookie = format!("{COOKIE_NAME}={session_id}; Path=/; HttpOnly; SameSite=Lax");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// 构造清除会话 Cookie 的 `Set-Cookie` 值。
pub fn clear_cookie(secure: bool) -> String {
    let mut cookie = format!("{COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}
