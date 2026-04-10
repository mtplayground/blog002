use leptos::prelude::*;

#[component]
pub fn DeleteConfirmModal(
    is_open: Signal<bool>,
    item_label: Signal<String>,
    is_processing: Signal<bool>,
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || is_open.get() fallback=|| view! { <></> }>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4">
                <div class="w-full max-w-md space-y-4 rounded-xl border border-slate-700 bg-slate-900 p-5 shadow-xl">
                    <h2 class="text-lg font-semibold text-white">"Confirm deletion"</h2>

                    <p class="text-sm text-slate-300">
                        {move || format!("Delete \"{}\"? This action cannot be undone.", item_label.get())}
                    </p>

                    <div class="flex justify-end gap-2">
                        <button
                            type="button"
                            class="rounded-md border border-slate-600 px-3 py-2 text-sm text-slate-200 transition hover:border-slate-400 hover:text-white disabled:cursor-not-allowed disabled:opacity-70"
                            disabled=move || is_processing.get()
                            on:click=move |_| on_cancel.run(())
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="rounded-md bg-rose-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-rose-500 disabled:cursor-not-allowed disabled:opacity-70"
                            disabled=move || is_processing.get()
                            on:click=move |_| on_confirm.run(())
                        >
                            {move || if is_processing.get() { "Deleting..." } else { "Delete" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
