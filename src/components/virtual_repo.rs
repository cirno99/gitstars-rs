//! 自研动态高度虚拟列表（仅客户端）。
//!
//! 需求要求仓库卡片描述「可展开」，展开会改变行高，属于动态高度列表；
//! 社区 crate `leptos_virtual_scroller` 只支持固定行高，因此这里自行实现：
//! 逐项测量实际高度，维护前缀和偏移，只渲染可视区间。
//!
//! 高度测量不依赖 `ResizeObserver`，而是在「挂载 / 展开状态变化 / 窗口尺寸变化」
//! 时于微任务中重新测量，避免为每个列表项持有观察器带来的生命周期负担。

use std::collections::HashMap;

use leptos::html::Div;
use leptos::prelude::*;

use crate::components::repo_list::RepoCard;
use crate::state::{self, app_ctx};

/// 未测量项的预估高度（像素）。
const ESTIMATED_HEIGHT: f64 = 150.0;
/// 视口外预渲染的缓冲高度。
const OVERSCAN: f64 = 600.0;

/// 布局：每项偏移、总高度、按 id 查询的偏移。
#[derive(Clone, Default, PartialEq)]
struct Layout {
    offsets: Vec<f64>,
    total: f64,
    by_id: HashMap<i64, f64>,
}

#[component]
pub fn VirtualRepoList() -> impl IntoView {
    let ctx = app_ctx();
    let items = Memo::new(move |_| state::filtered_repositories(&ctx));
    let heights = RwSignal::new(HashMap::<i64, f64>::new());
    let measure_nonce = RwSignal::new(0_u32);
    let scroll_top = RwSignal::new(0.0_f64);
    let viewport = RwSignal::new(600.0_f64);
    let container_ref = NodeRef::<Div>::new();

    let layout = Memo::new(move |_| {
        let list = items.get();
        let measured = heights.get();
        let mut offsets = Vec::with_capacity(list.len());
        let mut by_id = HashMap::with_capacity(list.len());
        let mut total = 0.0;
        for repo in &list {
            offsets.push(total);
            by_id.insert(repo.id, total);
            total += measured.get(&repo.id).copied().unwrap_or(ESTIMATED_HEIGHT);
        }
        Layout {
            offsets,
            total,
            by_id,
        }
    });

    let total_height = Memo::new(move |_| layout.get().total);

    let visible = Memo::new(move |_| {
        let list = items.get();
        let current = layout.get();
        if list.is_empty() {
            return Vec::new();
        }
        let top = scroll_top.get();
        let height = viewport.get();
        let offsets = &current.offsets;
        let start = offsets.partition_point(|o| *o < (top - OVERSCAN).max(0.0));
        let end = offsets.partition_point(|o| *o < top + height + OVERSCAN);
        let start = start.min(list.len());
        let end = end.min(list.len());
        list[start..end].to_vec()
    });

    Effect::new(move |_| {
        if let Some(element) = container_ref.get() {
            viewport.set(element.client_height() as f64);
            scroll_top.set(element.scroll_top() as f64);
        }
    });

    // 窗口尺寸变化会改变文本换行，需清空已测高度重新测量。
    let _resize = window_event_listener(leptos::ev::resize, move |_| {
        if let Some(element) = container_ref.get() {
            viewport.set(element.client_height() as f64);
        }
        heights.set(HashMap::new());
        measure_nonce.update(|n| *n += 1);
    });

    // 列表内容（筛选/排序/切换来源）变化时滚动回顶部。
    Effect::new(move |_| {
        let _ = items.get();
        if let Some(element) = container_ref.get() {
            element.set_scroll_top(0);
            scroll_top.set(0.0);
        }
    });

    let busy = move || ctx.stars_loading.get() || ctx.ranking_loading.get();

    view! {
        <div
            node_ref=container_ref
            class="relative flex-auto overflow-y-auto overflow-x-hidden"
            on:scroll=move |_| {
                if let Some(element) = container_ref.get() {
                    scroll_top.set(element.scroll_top() as f64);
                }
            }
        >
            <div style=move || format!("position: relative; height: {}px", total_height.get())>
                <For
                    each=move || visible.get()
                    key=|repository| repository.id
                    children=move |repository| {
                        let id = repository.id;
                        let expanded = Signal::derive(move || ctx.expanded.get() == Some(id));
                        view! {
                            <div
                                style=move || {
                                    let top =
                                        layout.get().by_id.get(&id).copied().unwrap_or(0.0);
                                    format!("position:absolute;top:{top}px;left:0;right:0")
                                }
                            >
                                <Measured
                                    id=id
                                    expanded=expanded
                                    measure_nonce=measure_nonce
                                    heights=heights
                                >
                                    <div
                                        class="repo-item cursor-pointer border-b border-line px-4 py-3"
                                        class:selected=move || ctx.selected_id.get() == Some(id)
                                        on:click=move |_| ctx.selected_id.set(Some(id))
                                    >
                                        <RepoCard repository=repository/>
                                    </div>
                                </Measured>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || busy() && items.get().is_empty()>
                <div class="pointer-events-none absolute inset-x-0 top-0 space-y-2 p-4">
                    {(0..4)
                        .map(|_| view! { <div class="skeleton h-24 rounded-xl"></div> })
                        .collect_view()}
                </div>
            </Show>

            <Show when=move || !busy() && items.get().is_empty()>
                <div class="grid h-48 place-items-center px-6 text-center text-sm text-faint">
                    {move || ctx.lang.get().no_result()}
                </div>
            </Show>
        </div>
    }
}

/// 包裹单项并测量其高度，写入 `heights`。
#[component]
fn Measured(
    id: i64,
    expanded: Signal<bool>,
    measure_nonce: RwSignal<u32>,
    heights: RwSignal<HashMap<i64, f64>>,
    children: Children,
) -> impl IntoView {
    let node_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        // 订阅展开状态与全局重测信号。
        let _ = expanded.get();
        let _ = measure_nonce.get();
        let Some(element) = node_ref.get() else {
            return;
        };
        // 微任务中测量，确保 DOM 已完成本次更新。
        queue_microtask(move || apply_height(&element, id, heights));
    });

    view! { <div node_ref=node_ref>{children()}</div> }
}

fn apply_height(element: &web_sys::HtmlElement, id: i64, heights: RwSignal<HashMap<i64, f64>>) {
    let height = element.get_bounding_client_rect().height();
    if height <= 0.0 {
        return;
    }
    heights.update(|map| {
        let changed = map
            .get(&id)
            .map(|previous| (previous - height).abs() > 0.5)
            .unwrap_or(true);
        if changed {
            map.insert(id, height);
        }
    });
}
