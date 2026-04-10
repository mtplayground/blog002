use leptos::prelude::*;

#[component]
pub fn BaseLayout(children: Children) -> impl IntoView {
    view! {
        <div class="shell">
            <header class="border-b border-slate-800/70 bg-slate-900/50">
                <div class="mx-auto flex max-w-6xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
                    <a class="text-base font-semibold tracking-wide text-cyan-300" href="/">
                        "blog002"
                    </a>
                    <nav class="flex items-center gap-4 text-sm text-slate-300">
                        <a class="transition hover:text-white" href="/">"Home"</a>
                        <a class="transition hover:text-white" href="/admin">"Admin"</a>
                    </nav>
                </div>
            </header>

            <main class="mx-auto w-full max-w-6xl px-4 py-10 sm:px-6 lg:px-8">
                <section class="content-card">{children()}</section>
            </main>
        </div>
    }
}
