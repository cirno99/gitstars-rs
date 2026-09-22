use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::header::Header;
use crate::components::icon::{Icon, IconSprite};
use crate::components::repo_content::RepoContent;
use crate::components::repo_list::RepoList;
use crate::components::sidebar::Sidebar;
use crate::components::unauth::Unauth;
#[cfg(feature = "hydrate")]
use crate::state::Theme;
use crate::state::{AppCtx, app_ctx, provide_app_ctx};

/// 服务端渲染的 HTML 外壳。
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="zh">
            <head>
                <meta charset="utf-8"/>
                <script inner_html=theme_init_script()></script>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <meta
                    name="description"
                    content="Gitstars：GitHub Stars 管理器与排行榜（Leptos 重写版）"
                />
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let ctx = provide_app_ctx();
    setup_client(ctx);

    view! {
        <Stylesheet id="leptos" href="/pkg/gitstars.css"/>
        <Title text="Gitstars"/>
        <IconSprite/>
        <Router>
            <Routes fallback=|| view! { <p class="p-8">"404"</p> }>
                <Route path=path!("/") view=Home/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let ctx = app_ctx();

    view! {
        <Show
            when=move || ctx.auth_checked.get()
            fallback=|| view! {
                <div class="grid h-screen w-full place-items-center bg-canvas">
                    <Icon name="loading" class="animate-spin text-2xl text-primary/60"/>
                </div>
            }
        >
            <Show when=move || ctx.user.get().is_some() fallback=|| view! { <Unauth/> }>
                <div class="flex h-screen overflow-hidden bg-canvas">
                    <Sidebar/>
                    <div class="flex min-w-0 flex-auto flex-col">
                        <Header/>
                        <div class="flex min-h-0 flex-auto">
                            <RepoList/>
                            <RepoContent/>
                        </div>
                    </div>
                </div>
            </Show>
        </Show>
    }
}

/// 主题初始化脚本。
///
/// 必须内联在 `<head>` 里同步执行：在首帧之前就把 `data-theme` 写到 `<html>` 上，
/// 否则深色用户会先看到一帧浅色（主题闪烁）。
///
/// 脚本同时监听系统配色变化，但 `resolve` 每次都会重新读 localStorage，
/// 因此用户显式选定浅/深色时不会被系统变化覆盖。
fn theme_init_script() -> String {
    format!(
        r#"(function(){{var K="{key}";function r(){{var t=null;try{{t=localStorage.getItem(K)}}catch(e){{}}
if(t!=="light"&&t!=="dark")t="system";
var d=t==="dark"||(t==="system"&&window.matchMedia("(prefers-color-scheme: dark)").matches);
document.documentElement.dataset.theme=d?"dark":"light"}}r();
try{{window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change",r)}}catch(e){{}}}})()"#,
        key = crate::constants::THEME_KEY
    )
}

/// 客户端初始化（仅在 hydrate 构建中存在）。
#[cfg(feature = "hydrate")]
fn setup_client(ctx: AppCtx) {
    use crate::constants::{LANG_KEY, THEME_KEY};
    use crate::i18n::Lang;
    use crate::server_fn::{exchange_code, login_url, me};
    use crate::state::Theme;
    use leptos::task::spawn_local;
    use wasm_bindgen::JsValue;

    let lang_initialized = StoredValue::new(false);

    // 语言：首次从 localStorage 读取，之后任何变化都写回。
    Effect::new(move |_| {
        let current = ctx.lang.get();
        if !lang_initialized.get_value() {
            lang_initialized.set_value(true);
            if let Some(stored) = read_local_storage(LANG_KEY) {
                let stored_lang = Lang::from_code(&stored);
                if stored_lang != current {
                    ctx.lang.set(stored_lang);
                    return;
                }
            }
        }
        write_local_storage(LANG_KEY, current.code());
    });

    // 主题：首次从 localStorage 读取，之后任何变化都写回并同步到 <html data-theme>。
    let theme_initialized = StoredValue::new(false);
    Effect::new(move |_| {
        let current = ctx.theme.get();
        if !theme_initialized.get_value() {
            theme_initialized.set_value(true);
            if let Some(stored) = read_local_storage(THEME_KEY) {
                let stored_theme = Theme::from_code(&stored);
                if stored_theme != current {
                    ctx.theme.set(stored_theme);
                    return;
                }
            }
        }
        write_local_storage(THEME_KEY, current.code());
        apply_theme(current);
    });

    // 鉴权 + OAuth 回调处理。
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let origin = normalize_origin(&window.location().origin().unwrap_or_default());
        let href = window.location().href().unwrap_or_default();
        let secure = origin.starts_with("https");
        let code = query_param(&href, "code");
        let cleaned = remove_code_param(&href);

        spawn_local(async move {
            if let Some(code) = code {
                match exchange_code(code, secure).await {
                    Ok(Some(user)) => ctx.user.set(Some(user)),
                    Ok(None) => {}
                    Err(err) => leptos::logging::warn!("OAuth 交换失败: {err}"),
                }
                if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                    let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&cleaned));
                }
            }

            if let Ok(url) = login_url(origin).await {
                ctx.login_url.set(url);
            }

            match me().await {
                Ok(Some(user)) => {
                    ctx.user.set(Some(user));
                    crate::loaders::load_stars(ctx, false);
                }
                Ok(None) => {}
                Err(err) => leptos::logging::warn!("获取用户信息失败: {err}"),
            }
            ctx.auth_checked.set(true);
        });
    });
}

/// 服务端构建下无需客户端初始化。
#[cfg(not(feature = "hydrate"))]
fn setup_client(_ctx: AppCtx) {}

/// 把主题写到 `<html data-theme>` 上。
///
/// 内联脚本只负责首次渲染，这里负责用户交互后的同步。
#[cfg(feature = "hydrate")]
fn apply_theme(theme: Theme) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let prefers_dark = window
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map(|query| query.matches())
        .unwrap_or(false);
    if let Some(root) = window.document().and_then(|doc| doc.document_element()) {
        let _ = root.set_attribute("data-theme", theme.resolve(prefers_dark));
    }
}

#[cfg(feature = "hydrate")]
fn read_local_storage(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

#[cfg(feature = "hydrate")]
fn write_local_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item(key, value);
    }
}

#[cfg(feature = "hydrate")]
fn query_param(href: &str, key: &str) -> Option<String> {
    let query = href.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| value.to_string())
    })
}

#[cfg(feature = "hydrate")]
fn remove_code_param(href: &str) -> String {
    let Some((base, query)) = href.split_once('?') else {
        return href.to_string();
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.starts_with("code="))
        .collect();
    if kept.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", kept.join("&"))
    }
}

/// 浏览器地址栏里的 `0.0.0.0` 不是 GitHub 认可的回调主机，
/// 统一改写为 `127.0.0.1`，避免登录时 `redirect_uri` 不匹配。
#[cfg(feature = "hydrate")]
fn normalize_origin(origin: &str) -> String {
    origin.replace("//0.0.0.0", "//127.0.0.1")
}
