use leptos::{html, prelude::*, task::spawn_local};

use crate::{
    api::auth::{self, LoginRequest},
    components::layout::BaseLayout,
    state::auth::use_or_provide_auth_context,
};

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let auth = use_or_provide_auth_context();
    let error_message = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);

    let email_ref = NodeRef::<html::Input>::new();
    let password_ref = NodeRef::<html::Input>::new();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let Some(email_input) = email_ref.get() else {
            error_message.set("email field is unavailable".to_string());
            return;
        };

        let Some(password_input) = password_ref.get() else {
            error_message.set("password field is unavailable".to_string());
            return;
        };

        let email = email_input.value();
        let password = password_input.value();

        if email.trim().is_empty() || password.is_empty() {
            error_message.set("email and password are required".to_string());
            return;
        }

        error_message.set(String::new());
        is_submitting.set(true);

        let auth_ctx = auth;
        let error_signal = error_message;
        let submitting_signal = is_submitting;

        spawn_local(async move {
            let payload = LoginRequest {
                email: email.trim().to_string(),
                password,
            };

            match auth::login(&payload).await {
                Ok(response) => {
                    auth_ctx.set_token(response.token);
                    redirect_to("/admin");
                }
                Err(err) => {
                    error_signal.set(format!("login failed: {err}"));
                }
            }

            submitting_signal.set(false);
        });
    };

    view! {
        <BaseLayout>
            <div class="mx-auto max-w-md space-y-6">
                <header class="space-y-2">
                    <p class="text-xs uppercase tracking-[0.16em] text-cyan-300">"Admin Access"</p>
                    <h1 class="text-2xl font-semibold text-white">"Sign in to dashboard"</h1>
                    <p class="text-sm text-slate-300">"Use your admin email and password to continue."</p>
                </header>

                <form class="space-y-4" on:submit=on_submit>
                    <label class="block space-y-1 text-sm text-slate-300">
                        <span>"Email"</span>
                        <input
                            node_ref=email_ref
                            type="email"
                            name="email"
                            autocomplete="email"
                            class="w-full rounded-lg border border-slate-700 bg-slate-950/70 px-3 py-2 text-slate-100 outline-none ring-cyan-400/50 transition focus:ring"
                        />
                    </label>

                    <label class="block space-y-1 text-sm text-slate-300">
                        <span>"Password"</span>
                        <input
                            node_ref=password_ref
                            type="password"
                            name="password"
                            autocomplete="current-password"
                            class="w-full rounded-lg border border-slate-700 bg-slate-950/70 px-3 py-2 text-slate-100 outline-none ring-cyan-400/50 transition focus:ring"
                        />
                    </label>

                    <button
                        type="submit"
                        class="inline-flex w-full items-center justify-center rounded-lg bg-cyan-500 px-4 py-2 font-medium text-slate-950 transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-70"
                        disabled=move || is_submitting.get()
                    >
                        {move || if is_submitting.get() { "Signing in..." } else { "Sign in" }}
                    </button>
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
        </BaseLayout>
    }
}

fn redirect_to(path: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(path);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
    }
}
