use leptos::prelude::*;

const AUTH_TOKEN_KEY: &str = "blog002_admin_jwt";

#[derive(Clone, Copy)]
pub struct AuthContext {
    token: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn token(&self) -> Option<String> {
        self.token.get()
    }

    pub fn set_token(&self, value: String) {
        self.token.set(Some(value.clone()));
        persist_token(Some(value));
    }

    pub fn clear_token(&self) {
        self.token.set(None);
        persist_token(None);
    }
}

pub fn provide_auth_context() -> AuthContext {
    let initial = load_token();
    let token = RwSignal::new(initial);
    let context = AuthContext { token };
    provide_context(context);
    context
}

pub fn use_or_provide_auth_context() -> AuthContext {
    if let Some(context) = use_context::<AuthContext>() {
        context
    } else {
        provide_auth_context()
    }
}

#[cfg(target_arch = "wasm32")]
fn load_token() -> Option<String> {
    use web_sys::window;

    let storage = window()?.local_storage().ok().flatten()?;
    storage.get_item(AUTH_TOKEN_KEY).ok().flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_token() -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
fn persist_token(token: Option<String>) {
    use web_sys::window;

    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        match token {
            Some(value) => {
                let _ = storage.set_item(AUTH_TOKEN_KEY, &value);
            }
            None => {
                let _ = storage.remove_item(AUTH_TOKEN_KEY);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_token(_token: Option<String>) {}
