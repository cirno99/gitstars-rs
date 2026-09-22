use std::time::Duration;

use leptos::html::Input;
use leptos::prelude::*;

use crate::components::icon::Icon;
use crate::loaders::load_stars;
use crate::models::Repository;
use crate::state::{RepoSort, TagSrc, TagType, app_ctx};

/// 仓库搜索栏 + 排序 + 刷新。
#[component]
pub fn RepoSearch() -> impl IntoView {
    let ctx = app_ctx();
    let handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let input_ref = NodeRef::<Input>::new();

    let on_input = move |ev| {
        let next = event_target_value(&ev);
        if let Some(previous) = handle.get_value() {
            previous.clear();
        }
        let handle_new = set_timeout_with_handle(
            move || ctx.repo_filter.set(next),
            Duration::from_millis(300),
        );
        handle.set_value(handle_new.ok());
    };

    let clear = move |_| {
        ctx.repo_filter.set(String::new());
        if let Some(input) = input_ref.get() {
            input.set_value("");
        }
    };

    let toggle_sort = move |_| {
        ctx.repo_sort.update(|s| {
            *s = if *s == RepoSort::Time {
                RepoSort::Star
            } else {
                RepoSort::Time
            }
        });
    };

    let refresh = move |_| load_stars(ctx, true);

    view! {
        <div class="flex h-14 flex-none items-center gap-1 border-b border-line bg-surface px-3">
            <div class="relative flex-auto">
                <Icon name="search" class="absolute left-3 top-2.5 text-muted"/>
                <input
                    node_ref=input_ref
                    type="text"
                    placeholder=move || ctx.lang.get().repo_filter_tip()
                    class="h-9 w-full rounded-full border border-line bg-surface-2 pl-9 pr-8 text-sm text-ink outline-none transition placeholder:text-faint focus:border-primary/50 focus:bg-surface focus:ring-4 focus:ring-primary/10"
                    on:input=on_input
                />
                <Show when=move || !ctx.repo_filter.get().is_empty()>
                    <span
                        class="absolute right-3 top-1/2 -translate-y-1/2 cursor-pointer text-faint transition hover:text-ink"
                        on:click=clear
                    >
                        <Icon name="close"/>
                    </span>
                </Show>
            </div>

            <Show when=move || ctx.tag_src.get() == TagSrc::Star>
                <button
                    type="button"
                    class="icon-btn text-base"
                    title=move || {
                        let lang = ctx.lang.get();
                        match ctx.repo_sort.get() {
                            RepoSort::Star => lang.repo_sort_star(),
                            RepoSort::Time => lang.repo_sort_time(),
                        }
                    }
                    on:click=toggle_sort
                >
                    {move || {
                        let name = match ctx.repo_sort.get() {
                            RepoSort::Star => "star",
                            RepoSort::Time => "time",
                        };
                        view! { <Icon name=name/> }
                    }}
                </button>

                <button
                    type="button"
                    class="btn-ghost"
                    on:click=refresh
                >
                    <Icon name="refresh" class="text-base"/>
                    {move || ctx.lang.get().refresh()}
                </button>

                <Show when=move || ctx.stars_loading.get() && !ctx.stars.get().is_empty()>
                    <span class="ml-1.5 text-muted" title=move || ctx.lang.get().repo_updating()>
                        <Icon name="loading" class="animate-spin"/>
                    </span>
                </Show>
            </Show>
        </div>
    }
}

/// 单个仓库卡片（描述可展开）。
#[component]
pub fn RepoCard(repository: Repository) -> impl IntoView {
    let ctx = app_ctx();
    let id = repository.id;
    let repo = repository;

    let description = repo.description.clone().unwrap_or_default();
    let long_description = description.chars().count() > 90;
    let is_expanded = move || ctx.expanded.get() == Some(id);
    let topic_disabled = move || ctx.tag_src.get() != TagSrc::Star;

    let select_topic = move |topic: String| {
        if ctx.tag_src.get_untracked() != TagSrc::Star {
            return;
        }
        ctx.tag_nav.set(TagType::Topic);
        ctx.selected_tag_type.set(TagType::Topic);
        ctx.selected_tag.set(topic);
    };

    let medal = match repo.ranking {
        Some(1) => "🥇",
        Some(2) => "🥈",
        Some(3) => "🥉",
        _ => "",
    };

    let repo_url = format!("https://github.com/{}/{}", repo.owner.login, repo.name);

    let topics = repo
        .topics
        .iter()
        .map(|topic| {
            let value = topic.clone();
            let value_for_class = topic.clone();
            view! {
                <li
                    class="tag-topic rounded-full border border-line bg-canvas px-2 py-0.5 hover:border-primary/50 hover:text-primary"
                    class:selected-tag=move || {
                        !topic_disabled()
                            && ctx.selected_tag_type.get() == TagType::Topic
                            && ctx.selected_tag.get() == value_for_class
                    }
                    on:click=move |_| select_topic(value.clone())
                >
                    {topic.clone()}
                </li>
            }
        })
        .collect_view();

    view! {
        <div class="flex items-start justify-between gap-2">
            <h2 class="flex min-w-0 items-center gap-1.5 text-sm font-semibold text-ink">
                <span class="flex-none">{medal}</span>
                <a
                    href=repo_url
                    rel="noopener noreferrer"
                    class="truncate transition hover:text-primary"
                >
                    {format!("{} / {}", repo.owner.login, repo.name)}
                </a>
                <Icon name="share" class="flex-none text-[0.7rem] text-muted"/>
            </h2>
            {repo
                .homepage
                .clone()
                .filter(|h| !h.is_empty())
                .map(|homepage| {
                    view! {
                        <a
                            href=homepage
                            class="icon-btn icon-btn-sm flex-none text-sm"
                            rel="noopener noreferrer"
                        >
                            <Icon name="link"/>
                        </a>
                    }
                })}
        </div>

        <ul
            class="mt-2 flex flex-wrap gap-1.5 text-[0.7rem] text-muted"
            class:disabled=move || topic_disabled()
        >
            {topics}
        </ul>

        <div class="mt-2 text-xs leading-relaxed text-muted">
            <span class=move || {
                if is_expanded() { "" } else { "line-clamp-2" }
            }>{description.clone()}</span>
            <Show when=move || long_description>
                <button
                    type="button"
                    class="ml-1 cursor-pointer font-medium text-primary hover:underline"
                    on:click=move |_| {
                        if ctx.expanded.get_untracked() == Some(id) {
                            ctx.expanded.set(None);
                        } else {
                            ctx.expanded.set(Some(id));
                        }
                    }
                >
                    {move || {
                        if is_expanded() {
                            ctx.lang.get().collapse()
                        } else {
                            ctx.lang.get().expand()
                        }
                    }}
                </button>
            </Show>
        </div>

        <div class="mt-3 flex items-center justify-between gap-2 text-xs">
            <div class="flex min-w-0 items-center gap-3.5 font-medium text-muted">
                <span class="inline-flex items-center gap-1">
                    <Icon name="star-fill" class="text-secondary"/>
                    <span class="tabular-nums">{repo.stargazers_count}</span>
                </span>
                <span class="inline-flex items-center gap-1">
                    <Icon name="branch"/>
                    <span class="tabular-nums">{repo.forks_count}</span>
                </span>
            </div>
            {repo
                .language
                .clone()
                .filter(|language| !language.is_empty())
                .map(|language| view! { <span class="chip truncate">{language}</span> })}
        </div>
    }
}

/// 仓库列表容器。
#[component]
pub fn RepoList() -> impl IntoView {
    view! {
        <div class="relative flex w-96 flex-none flex-col border-r border-line bg-surface">
            <RepoSearch/>
            {list_body()}
        </div>
    }
}

/// 客户端使用动态高度虚拟列表；服务端仅渲染占位（列表本就依赖客户端鉴权）。
#[cfg(feature = "hydrate")]
fn list_body() -> impl IntoView {
    crate::components::virtual_repo::VirtualRepoList()
}

#[cfg(not(feature = "hydrate"))]
fn list_body() -> impl IntoView {
    view! { <div class="flex-auto"></div> }
}
