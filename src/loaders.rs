//! 加载动作：调用 server function 并写入全局状态。

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::server_fn::{ranking, starred_repos};
use crate::state::AppCtx;

/// 加载星标仓库。`force = true` 时跳过缓存 TTL。
pub fn load_stars(ctx: AppCtx, force: bool) {
    ctx.stars_loading.set(true);
    ctx.stars_error.set(None);
    spawn_local(async move {
        match starred_repos(force).await {
            Ok(repos) => {
                ctx.stars.set(repos);
                ctx.stars_error.set(None);
            }
            Err(err) => {
                let message = err.to_string();
                if message.contains("unauthorized") {
                    ctx.user.set(None);
                } else {
                    ctx.stars_error.set(Some(message));
                }
            }
        }
        ctx.stars_loading.set(false);
    });
}

/// 加载排行榜（已加载且非强制时直接返回）。
pub fn load_ranking(ctx: AppCtx, force: bool) {
    if !force
        && (!ctx.ranking.get_untracked().repos.is_empty() || ctx.ranking_loading.get_untracked())
    {
        return;
    }
    ctx.ranking_loading.set(true);
    spawn_local(async move {
        match ranking(force).await {
            Ok(data) => ctx.ranking.set(data),
            Err(err) => leptos::logging::warn!("加载排行榜失败: {err}"),
        }
        ctx.ranking_loading.set(false);
    });
}
