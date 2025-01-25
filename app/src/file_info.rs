use crate::files::BINARY_STORE;
use leptos::prelude::*;
use log::error;

#[component]
pub fn FileInfo() -> impl IntoView {
    #[server]
    // Update the file info
    pub async fn update_file_info() -> Result<Vec<(String, String)>, ServerFnError> {
        match BINARY_STORE.write() {
            Ok(binary_lock) => {
                let file_info = binary_lock.file_info();
                // send event to update the view
                return Ok(file_info);
            }
            Err(e) => {
                error!("Failed to get binary lock: {:#?}", e);
                Err(ServerFnError::from(e))
            }
        }
    }

    let file_info = Resource::new(|| (), |_| async move { update_file_info().await });
    file_info.refetch();

    view! {
        <Suspense fallback=|| view!{ <p>"Loading..."</p> } >
            <div>
                <table>
                    <tr><th><strong>Field</strong></th><th><strong>Value</strong></th></tr>
                    {move || {
                        file_info.get().map(
                            |info| {
                                {info.unwrap().into_iter()
                                    .map(|(key, value)| view! { <tr><td>{key}</td><td>{value}</td></tr>})
                                    .collect::<Vec<_>>()}
                            }
                        )
                    }}
                </table>
            </div>
       </Suspense>
    }
}
