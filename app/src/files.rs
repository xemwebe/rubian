use leptos::*;
use once_cell::sync::Lazy;
use rubilib::{binary::Binary, blob::Blob};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::sync::RwLock;
use web_sys::{wasm_bindgen::JsCast, FormData, HtmlFormElement, SubmitEvent};

// Global instance of binary storage
pub static BINARY_STORE: Lazy<RwLock<Binary>> = Lazy::new(|| RwLock::new(Binary::default()));

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FileStats {
    pub file_name: String,
    pub len: usize,
}

#[component]
pub fn FileUpload() -> impl IntoView {
    /// Upload functions for files to be analyzed.
    #[server(
     input = MultipartFormData,
 )]
    pub async fn file_length(data: MultipartData) -> Result<FileStats, ServerFnError> {
        // `.into_inner()` returns the inner `multer` stream
        // it is `None` if we call this on the client, but always `Some(_)` on the server, so is safe to
        // unwrap match binary {
        let mut data = data.into_inner().unwrap();

        let mut bin_data = Vec::new();
        let mut file_name = String::new();
        while let Ok(Some(mut field)) = data.next_field().await {
            if let Some(name) = field.file_name() {
                if !file_name.is_empty() && file_name != name {
                    log::error!("Try to upload multiple files!");
                    break;
                }
                log::info!("File name is {}", name);
                file_name = name.to_owned();
            }

            while let Ok(Some(chunk)) = field.chunk().await {
                bin_data.extend_from_slice(&chunk);
            }
        }
        let len = bin_data.len();
        BINARY_STORE.write().unwrap().update(Blob::new(bin_data)?)?;
        let file_type = BINARY_STORE.read().unwrap().file_type();
        leptos_axum::redirect(&format!("/{file_type}"));
        Ok(FileStats { file_name, len })
    }

    let upload_action = create_action(|data: &FormData| {
        let data = data.clone();
        // `MultipartData` implements `From<FormData>`
        file_length(data.into())
    });

    view! {
        <p>Select File to upload and analyze.</p>
        <form on:submit=move |ev: SubmitEvent| {
            ev.prevent_default();
            let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
            let form_data = FormData::new_with_form(&target).unwrap();
            upload_action.dispatch(form_data);
        }>
            <input type="file" name="file_to_upload" id="file_to_upload" class="file-input" oninput="this.form.requestSubmit()" />
            <label for="file_to_upload" class="custom-button">Choose File</label>
        </form>
        <p>
            {move || if upload_action.input().get().is_none() && upload_action.value().get().is_none() {
                "Upload a file.".to_string()
            } else if upload_action.pending().get() {
                "Uploading...".to_string()
            } else if let Some(Ok(value)) = upload_action.value().get() {
                format!("File name: {} with size: {} (0x{:016x})", value.file_name, value.len, value.len)
            } else {
                format!("Error: {:?}", upload_action.value().get())
            }}
        </p>

    }
}
