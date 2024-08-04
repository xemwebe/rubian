use crate::error_template::{AppError, ErrorTemplate};

use html::Table;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

pub mod error_template;
mod file_info;
mod files;

use file_info::FileInfo;
use files::FileUpload;

use rubilib::elf;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css"/>

        // sets the document title
        <Title text="Rubian - binary file analyzer"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage>
                        <Route path="/elf" view=ElfPage/>
                        <Route path="/pe" view=PePage/>
                        <Route path="/unknown" view=UnknownPage/>
                        <Route path="" view=|| view!{
                            <p>"Select a file to start analyzing"</p>} />
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Rubian"</h1>
        <FileUpload/>
        <Outlet/>
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum ElfTable {
    Symbols,
    DynSymbols,
    SectionHeaders,
}

#[server]
pub async fn fetch_elf_table(table_type: ElfTable) -> Result<rubilib::table::Table, ServerFnError> {
    let mut binary = files::BINARY_STORE.write().unwrap();
    match binary.deref_mut() {
        rubilib::binary::Binary::Elf(elf_binary) => {
            let table = match table_type {
                ElfTable::SectionHeaders => elf_binary.section_headers_table()?,
                ElfTable::Symbols => elf_binary.symbols_table()?,
                ElfTable::DynSymbols => elf_binary.dyn_symbols_table()?,
            };
            Ok(table)
        }
        _ => {
            log::error!("Binary has invalid type");
            Err(ServerFnError::from(AppError::NotFound))
        }
    }
}

/// Renders the home page of your application.
#[component]
fn ElfPage() -> impl IntoView {
    let (tab, set_tab) = create_signal(ElfTable::SectionHeaders);
    let table =
        create_resource(tab,  |tab| async move { fetch_elf_table(tab).await });

    view! {
        <h2>"Analyzing ELF file"</h2>
        <FileInfo/>
        <span class="tabs">
            <button
                on:click=move |_| set_tab(ElfTable::SectionHeaders)
                class:selected=move || tab() == ElfTable::SectionHeaders
            >
                "Section Headers"
            </button>
            <button
                on:click=move |_| set_tab(ElfTable::Symbols)
                class:selected=move || tab() == ElfTable::Symbols
            >
                "Symbols"
            </button>
            <button
                on:click=move |_| set_tab(ElfTable::DynSymbols)
                class:selected=move || tab() == ElfTable::DynSymbols
            >
                "Dynamic Symbols"
            </button>
        </span>
        <Transition
            fallback=move || view! { <p>"Select table to show"</p> }
        >
            <Table table=table.get() />
        </Transition>
    }
}

/// Renders the home page of your application.
#[component]
fn PePage() -> impl IntoView {
    view! {
        <h2>"Analyzing PE file"</h2>
        <FileInfo/>
    }
}

/// Renders the home page of your application.
#[component]
fn UnknownPage() -> impl IntoView {
    view! {
        <p>"The file format is not supported"</p>
    }
}

/// Display a table view
#[component]
fn Table(table: Option<Result<rubilib::table::Table, ServerFnError>>) -> impl IntoView {
    if let Some(Ok(table)) = table {
        view! {
            <p>
            <table>
                <tr>
                    {table.headline.iter().map(|header| view! { <th>{header}</th> }).collect::<Vec<_>>() }
                </tr>
                {table.rows.iter().map(|row| view! {
                    <tr>
                        {row.content.iter().map(|cell| view! { <td>{cell}</td> }).collect::<Vec<_>>() }
                    </tr>
                }).collect::<Vec<_>>() }
            </table>
            </p>
        }
    } else if let Some(Err(e)) = table {
        view! {
            <p>{format!("Error: {}", e)}</p>
        }
    } else {
    view! {
            <p>"Loading table..."</p>
        }
    }
}