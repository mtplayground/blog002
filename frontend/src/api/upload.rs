use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UploadImageResponse {
    pub url: String,
    pub key: String,
    pub content_type: String,
    pub size: usize,
}

#[derive(Debug, Deserialize)]
struct ErrorPayload {
    error: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn upload_image(
    token: &str,
    file: web_sys::File,
    on_progress: impl Fn(u32) + 'static,
) -> Result<UploadImageResponse, String> {
    use std::rc::Rc;

    use js_sys::Promise;
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Event, FormData, ProgressEvent, XmlHttpRequest};

    let xhr = XmlHttpRequest::new().map_err(js_error_to_string)?;
    xhr.open_with_async("POST", "/api/admin/upload", true)
        .map_err(js_error_to_string)?;
    xhr.set_request_header("Authorization", &format!("Bearer {token}"))
        .map_err(js_error_to_string)?;

    let form_data = FormData::new().map_err(js_error_to_string)?;
    form_data
        .append_with_blob_and_filename("file", &file, &file.name())
        .map_err(js_error_to_string)?;

    let progress_callback = Rc::new(on_progress);
    let upload_target = xhr
        .upload()
        .ok_or_else(|| "upload progress target is unavailable".to_string())?;
    let progress_fn = progress_callback.clone();
    let on_progress = Closure::<dyn FnMut(ProgressEvent)>::new(move |event: ProgressEvent| {
        if event.length_computable() {
            let total = event.total();
            if total > 0.0 {
                let loaded = event.loaded();
                let percent = ((loaded / total) * 100.0).round().clamp(0.0, 100.0) as u32;
                progress_fn(percent);
            }
        }
    });
    upload_target.set_onprogress(Some(on_progress.as_ref().unchecked_ref()));
    on_progress.forget();

    let xhr_for_promise = xhr.clone();
    let completion = Promise::new(&mut move |resolve, reject| {
        let onload_xhr = xhr_for_promise.clone();
        let resolve_onload = resolve.clone();
        let reject_onload = reject.clone();
        let onload = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let status = onload_xhr.status().ok().unwrap_or_default();
            let body = onload_xhr
                .response_text()
                .ok()
                .flatten()
                .unwrap_or_default();

            if (200..300).contains(&status) {
                let _ = resolve_onload.call1(&JsValue::NULL, &JsValue::from_str(&body));
            } else {
                let _ = reject_onload.call1(
                    &JsValue::NULL,
                    &JsValue::from_str(&extract_error_message(&body, status)),
                );
            }
        });
        xhr_for_promise.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        let reject_onerror = reject.clone();
        let onerror = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let _ = reject_onerror.call1(
                &JsValue::NULL,
                &JsValue::from_str("upload request failed due to a network error"),
            );
        });
        xhr_for_promise.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        let reject_onabort = reject.clone();
        let onabort = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let _ = reject_onabort.call1(
                &JsValue::NULL,
                &JsValue::from_str("upload request was aborted"),
            );
        });
        xhr_for_promise.set_onabort(Some(onabort.as_ref().unchecked_ref()));
        onabort.forget();
    });

    xhr.send_with_opt_form_data(Some(&form_data))
        .map_err(js_error_to_string)?;

    let response_body = JsFuture::from(completion)
        .await
        .map_err(js_error_to_string)?;
    progress_callback(100);

    let response_text = response_body
        .as_string()
        .ok_or_else(|| "upload response was not valid text".to_string())?;

    serde_json::from_str::<UploadImageResponse>(&response_text)
        .map_err(|err| format!("failed to parse upload response: {err}"))
}

#[cfg(target_arch = "wasm32")]
fn js_error_to_string(value: wasm_bindgen::JsValue) -> String {
    value
        .as_string()
        .unwrap_or_else(|| "unexpected JavaScript runtime error".to_string())
}

#[cfg(target_arch = "wasm32")]
fn extract_error_message(body: &str, status: u16) -> String {
    if body.trim().is_empty() {
        return format!("upload failed with status {status}");
    }

    serde_json::from_str::<ErrorPayload>(body)
        .map(|payload| payload.error)
        .unwrap_or_else(|_| body.to_string())
}
