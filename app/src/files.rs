use leptos::prelude::*;
use log::info;
use once_cell::sync::Lazy;
use rubilib::{binary::Binary, blob::Blob};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::sync::RwLock;
use web_sys::{wasm_bindgen::JsCast, FormData, HtmlFormElement, HtmlInputElement, SubmitEvent};

// Global instance of binary storage
pub static BINARY_STORE: Lazy<RwLock<Binary>> = Lazy::new(|| RwLock::new(Binary::default()));

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FileStats {
    pub file_name: String,
    pub len: usize,
    pub file_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UploadedFile {
    pub data: Vec<u8>,
    pub file_name: String,
}

/// Upload functions for files to be analyzed.
#[server]
pub async fn upload_file(data: UploadedFile) -> Result<FileStats, ServerFnError> {
    let len = data.data.len();
    BINARY_STORE
        .write()
        .unwrap()
        .update(Blob::new(data.data)?)?;
    let file_type = BINARY_STORE.read().unwrap().file_type();
    leptos_axum::redirect(&format!("/{file_type}"));
    Ok(FileStats {
        file_name: data.file_name,
        len,
        file_type,
    })
}

#[component]
pub fn FileUpload() -> impl IntoView {
    let upload_file = ServerAction::<UploadFile>::new();
    let value = upload_file.value();

    view! {
        <p>Select File to upload and analyze.</p>
            <ActionForm action=upload_file>
            <input type="file" name="file_to_upload" id="file_to_upload" class="file-input" on:change=|ev| {
                let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                info!("elem: {:?}", elem);
                let files = elem.files();
                info!("files: {:?}", files);
            } />
            <label for="file_to_upload" class="custom-button">Choose File</label>
        </ActionForm>
        <p>
            {move || if upload_file.input().get().is_none() && upload_file.value().get().is_none() {
                "Upload a file.".to_string()
            } else if upload_file.pending().get() {
                "Uploading...".to_string()
            } else if let Some(Ok(value)) = upload_file.value().get() {
                format!("File name: {} with size: {} (0x{:016x})", value.file_name, value.len, value.len)
            } else {
                format!("Error: {:?}", upload_file.value().get())
            }}
        </p>
    }
}
