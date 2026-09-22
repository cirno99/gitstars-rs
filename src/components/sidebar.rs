use std::time::Duration;

use leptos::html::Input;
use leptos::prelude::*;

use crate::components::icon::Icon;
use crate::constants::{BRAND, BRAND_URI};
use crate::loaders::load_ranking;
use crate::state::{self, AppCtx, SortOrder, TagSrc, TagType, app_ctx};

/// 单个 tag 行。
#[component]
pub fn TagItem(
    #[prop(into)] label: Signal<String>,
    #[prop(optional, into)] count: Signal<Option<usize>>,
    #[prop(optional, into)] icon: String,
    #[prop(into)] selected: Signal<bool>,
    on_click: Callback<()>,
) -> impl IntoView {
    let icon_name = if icon.is_empty() {
        "tag".to_string()
    } else {
        icon
    };

    view! {
        <div
            class="tag-item mx-2 flex cursor-pointer items-center justify-between rounded-lg px-3 text-xs"
            class:selected=move || selected.get()
            on:click=move |_| on_click.run(())
        >
            <div class="flex min-w-0 items-center gap-2">
                <Icon name=icon_name class="text-sm opacity-80"/>
                <span class="truncate">{move || label.get()}</span>
            </div>
            {move || {
                count
                    .get()
                    .filter(|c| *c > 0)
                    .map(|c| {
                        view! {
                            <span class="rounded-full bg-white/10 px-2 py-0.5 text-[0.7rem] tabular-nums">
                                {c}
                            </span>
                        }
                    })
            }}
        </div>
    }
}

/// 构造一个 topic tag 行。
fn topic_tag_item(ctx: AppCtx, label: String, count: usize) -> impl IntoView {
    let name_for_selected = label.clone();
    let name_for_click = label.clone();
    let label_signal = Signal::derive(move || label.clone());
    view! {
        <TagItem
            label=label_signal
            count=Signal::derive(move || Some(count))
            selected=Signal::derive(move || {
                ctx.selected_tag.get() == name_for_selected
                    && ctx.selected_tag_type.get() == TagType::Topic
            })
            on_click=Callback::new(move |_| {
                ctx.selected_tag_type.set(TagType::Topic);
                ctx.selected_tag.set(name_for_click.clone());
            })
        />
    }
}

/// topic 列表：客户端用社区 crate `leptos_virtual_scroller` 虚拟滚动（固定行高）。
#[cfg(feature = "hydrate")]
fn render_topics(ctx: AppCtx, topics: Memo<Vec<(String, usize)>>) -> impl IntoView {
    use leptos_virtual_scroller::VirtualScroller;

    view! {
        <VirtualScroller
            each=topics
            key=|(_index, item): (usize, &(String, usize))| item.0.clone()
            header=()
            item_height=36
            children=move |(_index, item): (usize, &(String, usize))| {
                topic_tag_item(ctx, item.0.clone(), item.1)
            }
        />
    }
}

/// 服务端渲染下退化为普通列表。
#[cfg(not(feature = "hydrate"))]
fn render_topics(ctx: AppCtx, topics: Memo<Vec<(String, usize)>>) -> impl IntoView {
    view! {
        <ul>
            <For
                each=move || topics.get()
                key=|item| item.0.clone()
                children=move |(label, count)| topic_tag_item(ctx, label, count)
            />
        </ul>
    }
}

/// tag 搜索框（300ms 防抖），可带一个尾部插槽。
#[component]
pub fn TagSearch(
    value: RwSignal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let input_ref = NodeRef::<Input>::new();

    let on_input = move |ev| {
        let next = event_target_value(&ev);
        if let Some(previous) = handle.get_value() {
            previous.clear();
        }
        let handle_new =
            set_timeout_with_handle(move || value.set(next), Duration::from_millis(300));
        handle.set_value(handle_new.ok());
    };

    let clear = move |_| {
        value.set(String::new());
        if let Some(input) = input_ref.get() {
            input.set_value("");
        }
    };

    view! {
        <section class="flex h-12 flex-none items-center gap-2 border-t border-sidebar-line px-3 text-xs">
            <span class="flex-none text-sidebar-muted">
                <Icon name="search"/>
            </span>

            <span class="relative flex-auto">
                <input
                    node_ref=input_ref
                    type="text"
                    class="w-full rounded-lg border border-white/10 bg-white/5 py-1.5 pl-2.5 pr-7 text-white outline-none transition placeholder:text-sidebar-muted focus:border-primary/60 focus:bg-white/10"
                    on:input=on_input
                />
                <Show when=move || !value.get().is_empty()>
                    <span
                        class="absolute right-2 top-1/2 -translate-y-1/2 cursor-pointer text-sidebar-muted transition hover:text-white"
                        on:click=clear
                    >
                        on:click=clear
                    >
                        <Icon name="close"/>
                    </span>
                </Show>
            </span>

            {children.map(|children| children())}
        </section>
    }
}

/// 顶部来源页签：你的星标 / 排行榜。
#[component]
pub fn TagSrcTab() -> impl IntoView {
    let ctx = app_ctx();

    let switch = move |src: TagSrc| {
        ctx.tag_src.set(src);
        if src == TagSrc::Ranking {
            load_ranking(ctx, false);
        }
    };

    view! {
        <div class="flex-none px-3 pb-2 pt-3">
            <ul class="flex gap-1 rounded-xl bg-black/20 p-1 text-xs">
                <li
                    class="seg-tab flex h-7 flex-1 cursor-pointer items-center justify-center capitalize"
                    class:selected=move || ctx.tag_src.get() == TagSrc::Star
                    on:click=move |_| switch(TagSrc::Star)
                >
                    {move || ctx.lang.get().category_star()}
                </li>
                <li
                    class="seg-tab flex h-7 flex-1 cursor-pointer items-center justify-center capitalize"
                    class:selected=move || ctx.tag_src.get() == TagSrc::Ranking
                    on:click=move |_| switch(TagSrc::Ranking)
                >
                    {move || ctx.lang.get().category_ranking()}
                    <Show when=move || ctx.ranking_loading.get()>
                        <Icon name="loading" class="ml-1 animate-spin"/>
                    </Show>
                </li>
            </ul>
        </div>
    }
}

/// Topics / Languages 页签。
#[component]
pub fn TagTypeNav() -> impl IntoView {
    let ctx = app_ctx();
    view! {
        <div class="flex-none px-3 pb-2 pt-2">
            <ul class="flex gap-1 text-xs">
                <li
                    class="seg-tab flex h-7 flex-1 cursor-pointer items-center justify-center capitalize"
                    class:selected=move || ctx.tag_nav.get() == TagType::Language
                    on:click=move |_| ctx.tag_nav.set(TagType::Language)
                >
                    "languages"
                </li>
                <li
                    class="seg-tab flex h-7 flex-1 cursor-pointer items-center justify-center capitalize"
                    class:selected=move || ctx.tag_nav.get() == TagType::Topic
                    on:click=move |_| ctx.tag_nav.set(TagType::Topic)
                >
                    "topics"
                </li>
            </ul>
        </div>
    }
}

/// 「你的星标」侧栏：全部 + 搜索 + 排序 + Topics/Languages 列表。
#[component]
pub fn TagSrcSelf() -> impl IntoView {
    let ctx = app_ctx();

    let topic_map = Memo::new(move |_| state::topic_map(&ctx.stars.get()));
    let language_map = Memo::new(move |_| state::language_map(&ctx.stars.get()));
    let topics = Memo::new(move |_| {
        state::tag_list(&topic_map.get(), &ctx.tag_filter.get(), ctx.tag_sort.get())
    });
    let languages = Memo::new(move |_| {
        state::tag_list(
            &language_map.get(),
            &ctx.tag_filter.get(),
            ctx.tag_sort.get(),
        )
    });

    let toggle_sort = move |_| {
        ctx.tag_sort.update(|s| {
            *s = if *s == SortOrder::Descend {
                SortOrder::Ascend
            } else {
                SortOrder::Descend
            }
        });
    };

    view! {
        <div class="relative flex h-0 flex-auto flex-col">
            <TagItem
                label=Signal::derive(move || ctx.lang.get().all().to_string())
                count=Signal::derive(move || Some(ctx.stars.get().len()))
                icon="all-application"
                selected=Signal::derive(move || ctx.selected_tag.get().is_empty())
                on_click=Callback::new(move |_| ctx.selected_tag.set(String::new()))
            />

            <Show
                when=move || !ctx.stars.get().is_empty()
                fallback=|| view! {
                    <Icon name="loading" class="absolute left-1/2 top-1/3 -ml-3 animate-spin text-2xl text-sidebar-muted"/>
                }
            >
                <TagSearch value=ctx.tag_filter>
                    <button
                        type="button"
                        class="flex h-6 w-6 flex-none cursor-pointer items-center justify-center rounded-md text-white/50 transition hover:bg-white/10 hover:text-white"
                        title=move || {
                            let lang = ctx.lang.get();
                            match ctx.tag_sort.get() {
                                SortOrder::Ascend => lang.sort_ascend(),
                                SortOrder::Descend => lang.sort_descend(),
                            }
                        }
                        on:click=toggle_sort
                    >
                        {move || {
                            let name = match ctx.tag_sort.get() {
                                SortOrder::Ascend => "sort-ascend",
                                SortOrder::Descend => "sort-descend",
                            };
                            view! { <Icon name=name/> }
                        }}
                    </button>
                </TagSearch>

                <section class="flex-auto overflow-auto pb-2">
                    <Show when=move || ctx.tag_nav.get() == TagType::Topic>
                        {render_topics(ctx, topics)}
                    </Show>
                    <Show when=move || ctx.tag_nav.get() == TagType::Language>
                        <ul class="pb-2">
                            <For
                                each=move || languages.get()
                                key=|item| item.0.clone()
                                children=move |(label, count)| {
                                    let name = label.clone();
                                    let label_signal = Signal::derive({
                                        let label = label.clone();
                                        move || label.clone()
                                    });
                                    view! {
                                        <TagItem
                                            label=label_signal
                                            count=Signal::derive(move || Some(count))
                                            selected=Signal::derive({
                                                let name = name.clone();
                                                move || {
                                                    ctx.selected_tag.get() == name
                                                        && ctx.selected_tag_type.get()
                                                            == TagType::Language
                                                }
                                            })
                                            on_click=Callback::new(move |_| {
                                                ctx.selected_tag_type.set(TagType::Language);
                                                ctx.selected_tag.set(name.clone());
                                            })
                                        />
                                    }
                                }
                            />
                        </ul>
                    </Show>
                </section>

                <TagTypeNav/>
            </Show>
        </div>
    }
}

/// 「GitHub 排行榜」侧栏：全部 + 搜索 + 语言列表。
#[component]
pub fn TagSrcGithub() -> impl IntoView {
    let ctx = app_ctx();

    let languages = Memo::new(move |_| {
        let filter = ctx.ranking_filter.get().to_lowercase();
        ctx.ranking
            .get()
            .languages
            .into_iter()
            .filter(|name| filter.is_empty() || name.to_lowercase().contains(&filter))
            .collect::<Vec<_>>()
    });

    view! {
        <div class="flex h-0 flex-auto flex-col">
            <TagItem
                label=Signal::derive(move || ctx.lang.get().all().to_string())
                icon="all-application"
                selected=Signal::derive(move || ctx.selected_language.get().is_empty())
                on_click=Callback::new(move |_| ctx.selected_language.set(String::new()))
            />

            <TagSearch value=ctx.ranking_filter/>

            <ul class="flex-auto overflow-auto pb-2">
                <For
                    each=move || languages.get()
                    key=|name| name.clone()
                    children=move |name| {
                        let value = name.clone();
                        view! {
                            <TagItem
                                label=Signal::derive({
                                    let name = name.clone();
                                    move || name.clone()
                                })
                                selected=Signal::derive({
                                    let value = value.clone();
                                    move || ctx.selected_language.get() == value
                                })
                                on_click=Callback::new(move |_| {
                                    ctx.selected_language.set(value.clone());
                                })
                            />
                        }
                    }
                />
            </ul>
        </div>
    }
}

/// 左侧栏整体。
#[component]
pub fn Sidebar() -> impl IntoView {
    let ctx = app_ctx();

    view! {
        <div class="sidebar flex h-full w-72 flex-none flex-col border-r border-line bg-gradient-to-b from-sidebar to-sidebar-deep text-sidebar-ink shadow-pop">
            <a
                href=BRAND_URI
                class="group relative flex h-16 flex-none items-center justify-center gap-1.5"
            >
                <span class="pointer-events-none absolute inset-x-8 bottom-3 h-8 rounded-full bg-primary/25 opacity-0 blur-xl transition duration-300 group-hover:opacity-100"></span>
                <Icon
                    name="logo"
                    class="relative text-2xl text-primary transition duration-300 group-hover:rotate-12 group-hover:text-secondary"
                />
                <span class="brand-text relative font-brand text-2xl font-bold uppercase tracking-[0.18em]">
                    {BRAND}
                </span>
            </a>

            <TagSrcTab/>

            <Show when=move || ctx.tag_src.get() == TagSrc::Star>
                <TagSrcSelf/>
            </Show>
            <Show when=move || ctx.tag_src.get() == TagSrc::Ranking>
                <TagSrcGithub/>
            </Show>

            <footer class="flex h-9 flex-none items-center justify-center border-t border-sidebar-line px-4 text-xs">
                <a
                    href=move || ctx.user.get().map(|u| u.html_url).unwrap_or_default()
                    class="flex h-full min-w-0 items-center justify-center gap-1 text-sidebar-muted transition hover:text-primary"
                    rel="noopener noreferrer"
                >
                    <span class="truncate font-semibold">
                        {move || {
                            ctx.user
                                .get()
                                .and_then(|u| u.html_url.rsplit('/').next().map(str::to_owned))
                                .unwrap_or_default()
                        }}
                    </span>
                    <Icon name="share" class="flex-none text-secondary"/>
                </a>
            </footer>
        </div>
    }
}
