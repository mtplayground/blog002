use leptos::{ev, html, prelude::*, task::spawn_local};

use crate::api::upload;

#[component]
pub fn ImageUpload(
    token: Signal<Option<String>>,
    on_uploaded: Callback<String>,
) -> impl IntoView {
    let is_dragging = RwSignal::new(false);
    let is_uploading = RwSignal::new(false);
    let progress_percent = RwSignal::new(0u32);
    let error_message = RwSignal::new(String::new());
    let uploaded_url = RwSignal::new(None::<String>);
    let input_ref = NodeRef::<html::Input>::new();

    let on_file_change = move |event: ev::Event| {
        event.prevent_default();

        let Some(jwt) = token.get() else {
            error_message.set("sign in required to upload images".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(file) = event_file(&event) {
                upload_selected_file(
                    file,
                    jwt,
                    is_uploading,
                    progress_percent,
                    error_message,
                    uploaded_url,
                    on_uploaded,
                );
            } else {
                error_message.set("select an image file to upload".to_string());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = event;
            error_message.set("image upload is only available in browser runtime".to_string());
        }
    };

    let on_drag_over = move |event: ev::DragEvent| {
        event.prevent_default();
        is_dragging.set(true);
    };

    let on_drag_leave = move |event: ev::DragEvent| {
        event.prevent_default();
        is_dragging.set(false);
    };

    let on_drop = move |event: ev::DragEvent| {
        event.prevent_default();
        is_dragging.set(false);

        let Some(jwt) = token.get() else {
            error_message.set("sign in required to upload images".to_string());
            return;
        };

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(file) = dropped_file(&event) {
                upload_selected_file(
                    file,
                    jwt,
                    is_uploading,
                    progress_percent,
                    error_message,
                    uploaded_url,
                    on_uploaded,
                );
            } else {
                error_message.set("drop an image file to upload".to_string());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = event;
            error_message.set("image upload is only available in browser runtime".to_string());
        }
    };

    let open_picker = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(input) = input_ref.get() {
                let _ = input.click();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = input_ref;
        }
    };

    view! {
        <section class="space-y-3 rounded-xl border border-slate-800 bg-slate-950/40 p-4">
            <h2 class="text-lg font-semibold text-white">"Image upload"</h2>

            <div
                class=move || {
                    if is_dragging.get() {
                        "rounded-lg border-2 border-cyan-400 bg-cyan-500/10 p-5 text-center transition"
                    } else {
                        "rounded-lg border-2 border-dashed border-slate-700 bg-slate-950/20 p-5 text-center transition"
                    }
                }
                on:dragover=on_drag_over
                on:dragleave=on_drag_leave
                on:drop=on_drop
            >
                <input
                    node_ref=input_ref
                    type="file"
                    accept="image/jpeg,image/png,image/gif,image/webp,image/svg+xml"
                    class="hidden"
                    on:change=on_file_change
                />

                <p class="text-sm text-slate-200">
                    "Drag and drop an image here, or use file picker."
                </p>

                <button
                    type="button"
                    class="mt-3 inline-flex items-center rounded-md bg-cyan-500 px-3 py-2 text-sm font-medium text-slate-950 transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-70"
                    disabled=move || is_uploading.get()
                    on:click=open_picker
                >
                    {move || if is_uploading.get() { "Uploading..." } else { "Choose image" }}
                </button>
            </div>

            <Show when=move || is_uploading.get() fallback=|| view! { <></> }>
                <div class="space-y-2">
                    <div class="h-2 w-full rounded-full bg-slate-800">
                        <div
                            class="h-2 rounded-full bg-cyan-400 transition-all duration-150"
                            style=move || format!("width: {}%;", progress_percent.get())
                        ></div>
                    </div>
                    <p class="text-xs text-slate-300">
                        {move || format!("{}%", progress_percent.get())}
                    </p>
                </div>
            </Show>

            <Show
                when=move || !error_message.get().is_empty()
                fallback=|| view! { <></> }
            >
                <p class="rounded-md border border-rose-600/40 bg-rose-500/10 px-3 py-2 text-sm text-rose-200">
                    {move || error_message.get()}
                </p>
            </Show>

            <Show
                when=move || uploaded_url.get().is_some()
                fallback=|| view! { <></> }
            >
                <div class="space-y-2 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-100">
                    <p class="text-xs uppercase tracking-wide text-emerald-200">"Uploaded image URL"</p>
                    <p class="break-all">{move || uploaded_url.get().unwrap_or_default()}</p>
                </div>
            </Show>
        </section>
    }
}

#[cfg(target_arch = "wasm32")]
fn upload_selected_file(
    file: web_sys::File,
    token: String,
    is_uploading: RwSignal<bool>,
    progress_percent: RwSignal<u32>,
    error_message: RwSignal<String>,
    uploaded_url: RwSignal<Option<String>>,
    on_uploaded: Callback<String>,
) {
    is_uploading.set(true);
    progress_percent.set(0);
    error_message.set(String::new());

    spawn_local(async move {
        let upload_result =
            upload::upload_image(&token, file, move |progress| progress_percent.set(progress)).await;

        match upload_result {
            Ok(response) => {
                uploaded_url.set(Some(response.url.clone()));
                on_uploaded.run(response.url);
            }
            Err(err) => error_message.set(format!("failed to upload image: {err}")),
        }

        is_uploading.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
fn event_file(event: &ev::Event) -> Option<web_sys::File> {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlInputElement;

    let target = event.target()?;
    let input: HtmlInputElement = target.dyn_into().ok()?;
    let files = input.files()?;
    files.get(0)
}

#[cfg(target_arch = "wasm32")]
fn dropped_file(event: &ev::DragEvent) -> Option<web_sys::File> {
    let data_transfer = event.data_transfer()?;
    let files = data_transfer.files()?;
    files.get(0)
}
