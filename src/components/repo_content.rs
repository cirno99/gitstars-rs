use leptos::html::Article;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icon::Icon;
use crate::server_fn::readme;
use crate::state::{self, app_ctx};

/// 右侧 README 预览区。
#[component]
pub fn RepoContent() -> impl IntoView {
    let ctx = app_ctx();
    let readme_html = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let article_ref = NodeRef::<Article>::new();

    let selected = Memo::new(move |_| state::selected_repository(&ctx));

    Effect::new(move |_| {
        let Some(repo) = selected.get() else {
            readme_html.set(String::new());
            return;
        };
        let id = repo.id;
        readme_html.set(String::new());
        loading.set(true);
        spawn_local(async move {
            match readme(repo.owner.login.clone(), repo.name.clone()).await {
                Ok(html) => {
                    // 异步响应可能已过期，避免覆盖新选中项。
                    if ctx.selected_id.get_untracked() == Some(id) {
                        readme_html.set(html);
                    }
                }
                Err(err) => leptos::logging::warn!("加载 README 失败: {err}"),
            }
            loading.set(false);
        });
    });

    Effect::new(move |_| {
        // 切换仓库后滚动回顶部。
        let _ = selected.get();
        if let Some(article) = article_ref.get() {
            article.set_scroll_top(0);
        }
    });

    view! {
        <div class="relative flex min-w-0 flex-auto flex-col bg-surface">
            <Show when=move || selected.get().is_some()>
                <header class="flex h-14 flex-none items-center gap-2 border-b border-line bg-surface px-5">
                    {move || {
                        selected.get().map(|repo| {
                            let href = format!(
                                "https://github.com/{}/{}",
                                repo.owner.login, repo.name,
                            );
                            view! {
                                <a
                                    href=href
                                    class="group inline-flex min-w-0 items-center gap-2"
                                    rel="noopener noreferrer"
                                >
                                    <Icon
                                        name="github"
                                        class="text-lg text-muted transition group-hover:text-primary"
                                    />
                                    <h2 class="truncate text-sm font-semibold text-ink transition group-hover:text-primary">
                                        {repo.owner.login}
                                        <span class="mx-1 text-muted">"/"</span>
                                        {repo.name}
                                    </h2>
                                    <Icon name="share" class="text-xs text-muted"/>
                                </a>
                            }
                        })
                    }}
                </header>
            </Show>

            <Show when=move || readme_html.get().is_empty()>
                <div class="grid min-h-0 flex-auto place-items-center p-8">
                    <div class="flex flex-col items-center gap-4 text-center">
                        <div class="grid h-20 w-20 place-items-center rounded-2xl border border-line bg-surface-2">
                            <Icon name="hand-left" class="text-3xl text-faint"/>
                        </div>
                        <p class="font-brand text-2xl font-bold tracking-wide text-faint">
                            "README.md"
                        </p>
                        <Show when=move || selected.get().is_none()>
                            <p class="text-sm text-muted">{move || ctx.lang.get().readme_tip()}</p>
                        </Show>
                        <Show when=move || loading.get()>
                            <Icon name="loading" class="animate-spin text-2xl text-primary/60"/>
                        </Show>
                    </div>
                </div>
            </Show>

            <Show when=move || !readme_html.get().is_empty()>
                <article
                    node_ref=article_ref
                    class="markdown-body mx-auto min-h-0 w-full max-w-3xl flex-auto overflow-y-auto px-8 py-6 text-sm"
                    inner_html=move || readme_html.get()
                ></article>
            </Show>
        </div>
    }
}
