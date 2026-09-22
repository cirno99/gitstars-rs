use leptos::prelude::*;

/// 内联 SVG sprite（由 `assets/icons.sprite.svg` 生成）。
pub const SPRITE: &str = include_str!("../../assets/icons.sprite.svg");

/// 把 sprite 注入页面（隐藏），全局只需一次。
#[component]
pub fn IconSprite() -> impl IntoView {
    view! { <svg style="display:none" inner_html=SPRITE></svg> }
}

/// 单个图标。
#[component]
pub fn Icon(#[prop(into)] name: String, #[prop(into, optional)] class: String) -> impl IntoView {
    let href = format!("#icon-{name}");
    let class = format!("svg-icon {class}");
    view! {
        <svg class=class aria-hidden="true">
            <use href=href></use>
        </svg>
    }
}
