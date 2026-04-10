use leptos::{ev, prelude::*, task::spawn_local};

use crate::{
    api::categories::{self, CategoryDto, UpsertCategoryRequest},
    components::{admin_taxonomy_form::AdminTaxonomyForm, layout::BaseLayout},
    state::auth::use_or_provide_auth_context,
};

#[component]
pub fn AdminCategoriesPage() -> impl IntoView {
    let auth = use_or_provide_auth_context();

    let categories = RwSignal::new(Vec::<CategoryDto>::new());
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let editing_id = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(String::new());

    let load_categories = Callback::new(move |_| {
        if let Some(token) = auth.token() {
            let categories_signal = categories;
            let error_signal = error_message;
            let loading_signal = is_loading;

            spawn_local(async move {
                loading_signal.set(true);
                match categories::list_categories(&token).await {
                    Ok(list) => categories_signal.set(list),
                    Err(err) => error_signal.set(format!("failed to load categories: {err}")),
                }
                loading_signal.set(false);
            });
        } else {
            error_message.set("sign in required to manage categories".to_string());
        }
    });

    load_categories.run(());

    let on_submit = {
        let load_categories = load_categories;
        Callback::new(move |ev: ev::SubmitEvent| {
            ev.prevent_default();

            let Some(token) = auth.token() else {
                error_message.set("sign in required to manage categories".to_string());
                return;
            };

            if name.get().trim().is_empty() || slug.get().trim().is_empty() {
                error_message.set("name and slug are required".to_string());
                return;
            }

            let payload = UpsertCategoryRequest {
                name: name.get().trim().to_string(),
                slug: slug.get().trim().to_string(),
            };

            let current_editing = editing_id.get();
            let error_signal = error_message;
            let submitting_signal = is_submitting;
            let name_signal = name;
            let slug_signal = slug;
            let editing_signal = editing_id;

            submitting_signal.set(true);
            error_signal.set(String::new());

            spawn_local(async move {
                let result = if let Some(id) = current_editing {
                    categories::update_category(&token, &id, &payload).await.map(|_| ())
                } else {
                    categories::create_category(&token, &payload).await.map(|_| ())
                };

                match result {
                    Ok(()) => {
                        name_signal.set(String::new());
                        slug_signal.set(String::new());
                        editing_signal.set(None);
                        load_categories.run(());
                    }
                    Err(err) => error_signal.set(format!("failed to save category: {err}")),
                }

                submitting_signal.set(false);
            });
        })
    };

    let on_cancel = {
        Callback::new(move |_| {
            name.set(String::new());
            slug.set(String::new());
            editing_id.set(None);
            error_message.set(String::new());
        })
    };

    view! {
        <BaseLayout>
            <div class="space-y-6">
                <header class="space-y-2">
                    <p class="text-xs uppercase tracking-[0.16em] text-cyan-300">"Admin / Categories"</p>
                    <h1 class="text-2xl font-semibold text-white">"Category management"</h1>
                </header>

                <AdminTaxonomyForm
                    title="Category form"
                    name=name
                    slug=slug
                    is_submitting=is_submitting.read_only()
                    error_message=error_message.read_only()
                    submit_label=Signal::derive(move || {
                        if is_submitting.get() {
                            "Saving...".to_string()
                        } else if editing_id.get().is_some() {
                            "Update category".to_string()
                        } else {
                            "Create category".to_string()
                        }
                    })
                    on_submit=on_submit
                    on_cancel=Some(on_cancel)
                />

                <section class="space-y-3">
                    <h2 class="text-lg font-semibold text-white">"Categories"</h2>

                    <Show
                        when=move || is_loading.get()
                        fallback=move || {
                            view! {
                                <ul class="space-y-2">
                                    <For
                                        each=move || categories.get()
                                        key=|item| item.id.clone()
                                        children=move |item| {
                                            let item_id = item.id.clone();
                                            let item_name = item.name.clone();
                                            let item_slug = item.slug.clone();

                                            view! {
                                                <li class="rounded-lg border border-slate-800 bg-slate-950/30 p-3">
                                                    <div class="flex flex-wrap items-center justify-between gap-3">
                                                        <div>
                                                            <p class="font-medium text-white">{item_name.clone()}</p>
                                                            <p class="text-xs text-slate-400">{format!("slug: {}", item_slug.clone())}</p>
                                                        </div>

                                                        <div class="flex gap-2">
                                                            <button
                                                                class="rounded-md border border-slate-600 px-3 py-1 text-xs text-slate-200 transition hover:border-slate-400 hover:text-white"
                                                                on:click=move |_| {
                                                                    editing_id.set(Some(item_id.clone()));
                                                                    name.set(item_name.clone());
                                                                    slug.set(item_slug.clone());
                                                                    error_message.set(String::new());
                                                                }
                                                            >
                                                                "Edit"
                                                            </button>
                                                            <button
                                                                class="rounded-md border border-rose-700/70 px-3 py-1 text-xs text-rose-200 transition hover:border-rose-500 hover:text-white"
                                                                on:click=move |_| {
                                                                    if let Some(token) = auth.token() {
                                                                        let id = item_id.clone();
                                                                        let error_signal = error_message;
                                                                        let load = load_categories;
                                                                        spawn_local(async move {
                                                                            if let Err(err) = categories::delete_category(&token, &id).await {
                                                                                error_signal.set(format!("failed to delete category: {err}"));
                                                                            } else {
                                                                                load.run(());
                                                                            }
                                                                        });
                                                                    } else {
                                                                        error_message.set("sign in required to manage categories".to_string());
                                                                    }
                                                                }
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
                        <p class="text-sm text-slate-300">"Loading categories..."</p>
                    </Show>
                </section>
            </div>
        </BaseLayout>
    }
}
