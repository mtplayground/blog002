use leptos::{prelude::*, task::spawn_local};

use crate::{
    api::posts::{self, ListPostsQuery, PaginatedPostsDto, PostDto},
    components::{delete_confirm_modal::DeleteConfirmModal, layout::BaseLayout},
    state::auth::use_or_provide_auth_context,
};

const PAGE_SIZE: u32 = 10;

#[component]
pub fn AdminPostsPage() -> impl IntoView {
    let auth = use_or_provide_auth_context();

    let posts = RwSignal::new(Vec::<PostDto>::new());
    let page = RwSignal::new(1u32);
    let per_page = RwSignal::new(PAGE_SIZE);
    let total = RwSignal::new(0i64);
    let is_loading = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let error_message = RwSignal::new(String::new());
    let delete_target = RwSignal::new(None::<PostDto>);

    let load_posts = Callback::new(move |_| {
        let Some(token) = auth.token() else {
            error_message.set("sign in required to manage posts".to_string());
            return;
        };

        let page_value = page.get();
        let per_page_value = per_page.get();
        let posts_signal = posts;
        let total_signal = total;
        let loading_signal = is_loading;
        let error_signal = error_message;
        let page_signal = page;

        spawn_local(async move {
            loading_signal.set(true);
            error_signal.set(String::new());

            let query = ListPostsQuery {
                page: page_value,
                per_page: per_page_value,
            };

            match posts::list_posts(&token, &query).await {
                Ok(PaginatedPostsDto {
                    items,
                    page: current_page,
                    per_page: _,
                    total: count,
                }) => {
                    posts_signal.set(items);
                    total_signal.set(count);
                    page_signal.set(current_page);
                }
                Err(err) => {
                    error_signal.set(format!("failed to load posts: {err}"));
                    posts_signal.set(Vec::new());
                }
            }

            loading_signal.set(false);
        });
    });

    load_posts.run(());

    let total_pages = Signal::derive(move || {
        let count = total.get();
        let size = per_page.get().max(1);

        if count <= 0 {
            1
        } else {
            ((count as u32).saturating_add(size - 1)) / size
        }
    });

    let on_prev_page = {
        let load_posts = load_posts;
        move |_| {
            let current = page.get();
            if current > 1 {
                page.set(current - 1);
                load_posts.run(());
            }
        }
    };

    let on_next_page = {
        let load_posts = load_posts;
        move |_| {
            let current = page.get();
            let max_page = total_pages.get();
            if current < max_page {
                page.set(current + 1);
                load_posts.run(());
            }
        }
    };

    let on_open_delete_modal = move |post: PostDto| {
        delete_target.set(Some(post));
    };

    let on_cancel_delete = Callback::new(move |_| {
        if !is_deleting.get() {
            delete_target.set(None);
        }
    });

    let on_confirm_delete = {
        let load_posts = load_posts;
        Callback::new(move |_| {
            let Some(token) = auth.token() else {
                error_message.set("sign in required to manage posts".to_string());
                return;
            };

            let Some(target) = delete_target.get() else {
                return;
            };

            is_deleting.set(true);
            error_message.set(String::new());

            let id = target.id;
            let deleting_signal = is_deleting;
            let error_signal = error_message;
            let delete_target_signal = delete_target;

            spawn_local(async move {
                match posts::delete_post(&token, &id).await {
                    Ok(()) => {
                        delete_target_signal.set(None);
                        load_posts.run(());
                    }
                    Err(err) => error_signal.set(format!("failed to delete post: {err}")),
                }

                deleting_signal.set(false);
            });
        })
    };

    let delete_label = Signal::derive(move || {
        delete_target
            .get()
            .map(|post| post.title)
            .unwrap_or_else(|| "post".to_string())
    });

    view! {
        <BaseLayout>
            <div class="space-y-6">
                <header class="space-y-2">
                    <p class="text-xs uppercase tracking-[0.16em] text-cyan-300">"Admin / Posts"</p>
                    <h1 class="text-2xl font-semibold text-white">"Post management"</h1>
                    <p class="text-sm text-slate-300">
                        "All draft and published posts are listed here."
                    </p>
                </header>

                <Show
                    when=move || !error_message.get().is_empty()
                    fallback=|| view! { <></> }
                >
                    <p class="rounded-lg border border-rose-600/40 bg-rose-500/10 px-3 py-2 text-sm text-rose-200">
                        {move || error_message.get()}
                    </p>
                </Show>

                <section class="space-y-3">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <h2 class="text-lg font-semibold text-white">"Posts"</h2>
                        <p class="text-xs text-slate-400">
                            {move || format!("Total: {}", total.get())}
                        </p>
                    </div>

                    <Show
                        when=move || is_loading.get()
                        fallback=move || {
                            view! {
                                <ul class="space-y-3">
                                    <For
                                        each=move || posts.get()
                                        key=|post| post.id.clone()
                                        children=move |post| {
                                            let delete_item = post.clone();
                                            let title = post.title.clone();
                                            let slug = post.slug.clone();
                                            let category_name = post.category.name.clone();
                                            let category_slug = post.category.slug.clone();
                                            let tags_line = render_tags(&post);
                                            let status = post.status.clone();
                                            let status_label = status.clone();
                                            view! {
                                                <li class="rounded-xl border border-slate-800 bg-slate-950/30 p-4">
                                                    <div class="flex flex-wrap items-start justify-between gap-3">
                                                        <div class="space-y-1">
                                                            <h3 class="text-base font-semibold text-white">{title}</h3>
                                                            <p class="text-xs text-slate-400">{format!("slug: {}", slug)}</p>
                                                            <p class="text-xs text-slate-400">
                                                                {format!("category: {} ({})", category_name, category_slug)}
                                                            </p>
                                                            <p class="text-xs text-slate-400">
                                                                {format!("tags: {}", tags_line)}
                                                            </p>
                                                        </div>

                                                        <div class="flex flex-col items-end gap-2">
                                                            <span class=move || status_badge_class(&status)>
                                                                {status_label}
                                                            </span>
                                                            <button
                                                                type="button"
                                                                class="rounded-md border border-rose-700/70 px-3 py-1 text-xs text-rose-200 transition hover:border-rose-500 hover:text-white"
                                                                on:click=move |_| on_open_delete_modal(delete_item.clone())
                                                            >
                                                                "Delete"
                                                            </button>
                                                        </div>
                                                    </div>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            }
                        }
                    >
                        <p class="text-sm text-slate-300">"Loading posts..."</p>
                    </Show>
                </section>

                <div class="flex items-center justify-between gap-2 rounded-lg border border-slate-800 bg-slate-950/30 px-3 py-2">
                    <button
                        type="button"
                        class="rounded-md border border-slate-700 px-3 py-1 text-sm text-slate-200 transition hover:border-slate-500 hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                        disabled=move || page.get() <= 1 || is_loading.get()
                        on:click=on_prev_page
                    >
                        "Previous"
                    </button>

                    <p class="text-xs text-slate-400">
                        {move || format!("Page {} of {}", page.get(), total_pages.get())}
                    </p>

                    <button
                        type="button"
                        class="rounded-md border border-slate-700 px-3 py-1 text-sm text-slate-200 transition hover:border-slate-500 hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                        disabled=move || page.get() >= total_pages.get() || is_loading.get()
                        on:click=on_next_page
                    >
                        "Next"
                    </button>
                </div>
            </div>

            <DeleteConfirmModal
                is_open=Signal::derive(move || delete_target.get().is_some())
                item_label=delete_label
                is_processing=Signal::derive(move || is_deleting.get())
                on_cancel=on_cancel_delete
                on_confirm=on_confirm_delete
            />
        </BaseLayout>
    }
}

fn status_badge_class(status: &str) -> String {
    match status {
        "published" => {
            "inline-flex rounded-full border border-emerald-500/40 bg-emerald-500/10 px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-emerald-200".to_string()
        }
        "draft" => {
            "inline-flex rounded-full border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-amber-200".to_string()
        }
        "archived" => {
            "inline-flex rounded-full border border-slate-500/40 bg-slate-500/10 px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-slate-200".to_string()
        }
        _ => {
            "inline-flex rounded-full border border-cyan-500/40 bg-cyan-500/10 px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-cyan-200".to_string()
        }
    }
}

fn render_tags(post: &PostDto) -> String {
    if post.tags.is_empty() {
        "none".to_string()
    } else {
        post.tags
            .iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
