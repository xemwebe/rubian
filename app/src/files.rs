use crate::error_template::AppError;
use leptos::*;
use log::error;
use once_cell::sync::Lazy;
use rubilib::{
    binary::Binary,
    blob::{BinaryType, Blob},
};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::{path::PathBuf, sync::RwLock};
use thiserror::Error;
use web_sys::{wasm_bindgen::JsCast, FormData, HtmlFormElement, SubmitEvent};

// Global instance of binary storage
pub static BINARY_STORE: Lazy<RwLock<Binary>> = Lazy::new(|| RwLock::new(Binary::default()));

#[component]
pub fn FileUpload() -> impl IntoView {
    /// Upload functions for files to be analyzed.
    #[server( 
     input = MultipartFormData, 
 )]
    pub async fn file_length(data: MultipartData) -> Result<usize, ServerFnError> {
        // `.into_inner()` returns the inner `multer` stream
        // it is `None` if we call this on the client, but always `Some(_)` on the server, so is safe to
        // unwrap match binary {
        let mut data = data.into_inner().unwrap();

        let mut bin_data = Vec::new();
        while let Ok(Some(mut field)) = data.next_field().await {
            println!("\n[NEXT FIELD]\n");
            let name = field.name().unwrap_or_default().to_string();
            println!("  [NAME] {name}");
            while let Ok(Some(chunk)) = field.chunk().await {
                bin_data.extend_from_slice(&chunk);
            }
        }
        let len = bin_data.len();
        BINARY_STORE.write().unwrap().update(Blob::new(bin_data)?)?;
        Ok(len)
    }

    let upload_action = create_action(|data: &FormData| {
        let data = data.clone();
        // `MultipartData` implements `From<FormData>`
        file_length(data.into())
    });

    view! {
        <h3>Load Binary File</h3>
        <p>Select File to upload and analyze.</p>
        <form on:submit=move |ev: SubmitEvent| {
            ev.prevent_default();
            let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
            let form_data = FormData::new_with_form(&target).unwrap();
            upload_action.dispatch(form_data);
        }>
            <input type="file" name="file_to_upload"/>
            <input type="submit"/>
        </form>
        <p>
            {move || if upload_action.input().get().is_none() && upload_action.value().get().is_none() {
                "Upload a file.".to_string()
            } else if upload_action.pending().get() {
                "Uploading...".to_string()
            } else if let Some(Ok(value)) = upload_action.value().get() {
                value.to_string()
            } else {
                format!("{:?}", upload_action.value().get())
            }}
        </p>

    }
}
