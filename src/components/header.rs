use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icon::Icon;
use crate::constants::BRAND_URI;
use crate::i18n::Lang;
use crate::server_fn::logout;
use crate::state::{Theme, app_ctx};

/// 主题切换按钮：跟随系统 → 浅色 → 深色 循环。
///
/// `class` 用于适配不同底色（顶栏 / 落地页）。
#[component]
pub fn ThemeToggle(#[prop(optional, into)] class: String) -> impl IntoView {
    let ctx = app_ctx();
    let class = if class.is_empty() {
        "icon-btn".to_owned()
    } else {
        format!("icon-btn {class}")
    };

    view! {
        <button
            type="button"
            class=class
            title=move || {
                let lang = ctx.lang.get();
                match ctx.theme.get() {
                    Theme::System => lang.theme_system(),
                    Theme::Light => lang.theme_light(),
                    Theme::Dark => lang.theme_dark(),
                }
            }
            on:click=move |_| ctx.theme.update(|theme| *theme = theme.next())
        >
            {move || {
                view! { <Icon name=ctx.theme.get().icon() class="text-lg"/> }
            }}
        </button>
    }
}

/// 顶部栏：用户信息、主题/语言切换、退出登录、仓库链接。
#[component]
pub fn Header() -> impl IntoView {
    let ctx = app_ctx();
    let lang = ctx.lang;

    view! {
        <header class="flex h-16 flex-none items-center gap-3 border-b border-line bg-surface/80 px-5 backdrop-blur-xl">
            <div class="flex min-w-0 items-center gap-3">
                {move || {
                    ctx.user.get().map(|user| {
                        let name = user.name.clone().unwrap_or_else(|| user.login.clone());
                        let repos_url = format!("{}?tab=repositories", user.html_url);
                        view! {
                            <a
                                href=user.html_url.clone()
                                class="flex-none"
                                rel="noopener noreferrer"
                            >
                                <img
                                    src=user.avatar_url.clone()
                                    alt=""
                                    loading="lazy"
                                    class="h-10 w-10 rounded-full ring-2 ring-primary/25 transition duration-200 hover:ring-primary/60"
                                />
                            </a>
                            <a href=repos_url class="group min-w-0" rel="noopener noreferrer">
                                <h2 class="truncate text-base font-semibold text-ink transition group-hover:text-primary-strong">
                                    {lang.get().user_title(&name)}
                                    <Icon
                                        name="share"
                                        class="ml-1 text-xs text-faint transition group-hover:text-primary-strong"
                                    />
                                </h2>
                            </a>
                        }
                    })
                }}
            </div>

            <span class="flex-auto"></span>

            <div class="flex flex-none items-center gap-0.5">
                <ThemeToggle/>
                <button
                    type="button"
                    class="btn-ghost"
                    title=move || {
                        if lang.get() == Lang::Zh { "Switch to English" } else { "切换到中文" }
                    }
                    on:click=move |_| lang.update(|current| *current = current.toggle())
                >
                    <Icon name="translate" class="text-base"/>
                    <span>{move || if lang.get() == Lang::Zh { "中" } else { "En" }}</span>
                </button>
            </div>

            <span class="h-5 w-px flex-none bg-line"></span>

            <button
                type="button"
                class="btn-ghost btn-ghost-danger"
                on:click=move |_| {
                    spawn_local(async move {
                        let _ = logout().await;
                        ctx.user.set(None);
                        ctx.stars.set(Vec::new());
                        ctx.login_url.set(String::new());
                    });
                }
            >
                <Icon name="logout" class="text-base"/>
                {move || lang.get().logout()}
            </button>

            <a href=BRAND_URI class="icon-btn" title="GitHub" rel="noopener noreferrer">
                <Icon name="github" class="text-lg"/>
            </a>
        </header>
    }
}
