use leptos::prelude::*;

use crate::components::header::ThemeToggle;
use crate::components::icon::Icon;
use crate::constants::BRAND;
use crate::state::app_ctx;

/// 未登录落地页。
#[component]
pub fn Unauth() -> impl IntoView {
    let ctx = app_ctx();

    view! {
        <div class="unauth relative h-full w-full overflow-hidden">
            <div class="wrapper pointer-events-none absolute inset-0"></div>
            <div class="pointer-events-none absolute -left-24 top-1/4 h-72 w-72 rounded-full bg-white/10 blur-3xl"></div>
            <div class="pointer-events-none absolute -right-16 bottom-1/4 h-80 w-80 rounded-full bg-white/10 blur-3xl"></div>

            <div class="absolute right-5 top-5 z-10">
                <ThemeToggle class="text-white/80 hover:bg-white/15 hover:text-white"/>
            </div>

            <div class="relative grid h-full place-items-center p-6">
                <div class="animate-rise w-full max-w-lg rounded-[1.75rem] border border-white/25 bg-white/10 p-10 text-center shadow-pop backdrop-blur-xl">
                    <div class="mx-auto mb-6 grid h-16 w-16 place-items-center rounded-2xl border border-white/25 bg-white/15 shadow-soft">
                        <Icon name="logo" class="text-4xl text-white"/>
                    </div>

                    <h1 class="font-brand text-5xl font-bold uppercase tracking-[0.2em] text-white drop-shadow-sm">
                        {BRAND}
                    </h1>

                    <p class="mx-auto mt-5 max-w-sm text-sm leading-relaxed text-white/85">
                        {move || ctx.lang.get().tagline()}
                    </p>

                    <a
                        class="group mx-auto mt-9 flex w-72 items-center justify-center gap-2 rounded-full bg-white px-6 py-3.5 text-base font-semibold text-ink shadow-pop transition duration-200 hover:-translate-y-0.5 hover:shadow-lg active:translate-y-0"
                        href=move || ctx.login_url.get()
                        target="_self"
                    >
                        <Icon
                            name="github"
                            class="text-xl text-ink transition group-hover:text-primary-strong"
                        />
                        <span>{move || ctx.lang.get().login_tip()}</span>
                    </a>

                    <p class="mx-auto mt-8 max-w-xs text-[0.7rem] leading-relaxed text-white/60">
                        {move || ctx.lang.get().disclaimer()}
                    </p>
                </div>
            </div>
        </div>
    }
}
