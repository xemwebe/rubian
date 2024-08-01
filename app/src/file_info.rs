use crate::error_template::AppError;
use crate::files::BINARY_STORE;
use leptos::*;
use log::error;
use rubilib::{
    binary::Binary,
    blob::{BinaryType, BlobError},
};
use std::{path::PathBuf, sync::RwLock};
use thiserror::Error;

#[component]
pub fn FileInfo() -> impl IntoView {
    #[server]
    // Update the file info
    pub async fn update_file_info() -> Result<String, ServerFnError> {
        match BINARY_STORE.write() {
            Ok(binary_lock) => {
                let file_info = match &*binary_lock {
                    Binary::Elf(elf) => elf.header_info(),
                    Binary::Pe => "File is Windows binary".to_string(),
                    Binary::Unknown => "unknown file type".to_string(),
                };
                // send event to update the view
                return Ok(file_info);
            }
            Err(e) => {
                error!("Failed to get binary lock: {:#?}", e);
                Err(ServerFnError::from(e))
            }
        }
    }

    let file_info: Resource<(), _> =
        create_resource(|| (), |_| async move { update_file_info().await });
    view! {
        <h3>File Info</h3>
        <p>Basic file info of loaded binary.</p>
        <button on:click=move |_| { file_info.refetch(); }>"Analyze File"</button>
        <Suspense fallback=|| view!{ <p>"Loading..."</p> } >
       <pre>{move || file_info.get()}</pre>
       </Suspense>
    }
}
