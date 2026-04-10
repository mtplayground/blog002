use leptos::{ev, prelude::*};

#[component]
pub fn AdminTaxonomyForm(
    title: &'static str,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    is_submitting: ReadSignal<bool>,
    error_message: ReadSignal<String>,
    submit_label: Signal<String>,
    on_submit: Callback<ev::SubmitEvent>,
    on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let show_cancel = on_cancel.is_some();
    let cancel_action = on_cancel;

    view! {
        <div class="space-y-4 rounded-xl border border-slate-800/70 bg-slate-950/40 p-4">
            <h2 class="text-lg font-semibold text-white">{title}</h2>

            <form
                class="space-y-3"
                on:submit=move |ev| on_submit.run(ev)
            >
                <label class="block space-y-1 text-sm text-slate-300">
                    <span>"Name"</span>
                    <input
                        type="text"
                        class="w-full rounded-lg border border-slate-700 bg-slate-950/70 px-3 py-2 text-slate-100 outline-none ring-cyan-400/50 transition focus:ring"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                </label>

                <label class="block space-y-1 text-sm text-slate-300">
                    <span>"Slug"</span>
                    <input
                        type="text"
                        class="w-full rounded-lg border border-slate-700 bg-slate-950/70 px-3 py-2 text-slate-100 outline-none ring-cyan-400/50 transition focus:ring"
                        prop:value=move || slug.get()
                        on:input=move |ev| slug.set(event_target_value(&ev))
                    />
                </label>

                <div class="flex flex-wrap gap-2">
                    <button
                        type="submit"
                        class="inline-flex items-center justify-center rounded-lg bg-cyan-500 px-4 py-2 font-medium text-slate-950 transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-70"
                        disabled=move || is_submitting.get()
                    >
                        {move || submit_label.get()}
                    </button>

                    <Show
                        when=move || show_cancel
                        fallback=|| view! { <div></div> }
                    >
                        <button
                            type="button"
                            class="inline-flex items-center justify-center rounded-lg border border-slate-600 px-4 py-2 text-sm text-slate-200 transition hover:border-slate-400 hover:text-white"
                            on:click=move |_| {
                                if let Some(cancel) = cancel_action {
                                    cancel.run(());
                                }
                            }
                        >
                            "Cancel"
                        </button>
                    </Show>
                </div>
            </form>

            <Show
                when=move || !error_message.get().is_empty()
                fallback=|| view! { <div></div> }
            >
                <p class="rounded-lg border border-rose-600/40 bg-rose-500/10 px-3 py-2 text-sm text-rose-200">
                    {move || error_message.get()}
                </p>
            </Show>
        </div>
    }
}
