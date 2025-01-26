use crate::error_template::{AppError, ErrorTemplate};

use leptos::{either::EitherOf3, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    StaticSegment,
};
use log::info;

pub mod error_template;
mod file_info;
mod files;

use file_info::FileInfo;
use files::FileUpload;

use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    info!("where to I run?");

    view! {
        <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css"/>

        // sets the document title
        <Title text="Rubian - binary file analyzer"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! { move || <ErrorTemplate outside_errors/> }.into_view()
                }>
                    <ParentRoute path=StaticSegment("") view=HomePage>
                        <Route path=StaticSegment("elf") view=ElfPage/>
                        <Route path=StaticSegment("pe") view=PePage/>
                        <Route path=StaticSegment("unknown") view=UnknownPage/>
                         <Route path=StaticSegment("") view=|| view!{
                             <p>"Load a file to start analyzing"</p>} />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    info!("HomePage");
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
    info!("display elf table");
    let (tab, set_tab) = signal(ElfTable::SectionHeaders);
    let table = Resource::new(tab, |tab| async move { fetch_elf_table(tab).await });

    view! {
        <h2>"Analyzing ELF file"</h2>
        <FileInfo/>
        <span class="tabs">
            <button
                on:click=move |_| set_tab(ElfTable::SectionHeaders)
                class:selected=move || tab() == ElfTable::SectionHeaders
                class="tab"
            >
                "Section Headers"
            </button>
            <button
                on:click=move |_| set_tab(ElfTable::Symbols)
                class:selected=move || tab() == ElfTable::Symbols
                class="tab"
            >
                "Symbols"
            </button>
            <button
                on:click=move |_| set_tab(ElfTable::DynSymbols)
                class:selected=move || tab() == ElfTable::DynSymbols
                class="tab"
            >
                "Dynamic Symbols"
            </button>
        </span>
        <Transition
            fallback=move || view! { <p>"Select table to show"</p> }
        >
            <Table table/>
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
fn Table(table: Resource<Result<rubilib::table::Table, ServerFnError>>) -> impl IntoView {
    info!("Try to display table");
    let table = table.get();
    if let Some(Ok(table)) = table {
        EitherOf3::A(view! {
            <p>
            <div style="overflow-x:auto; overflow-y:auto;">
            <table>
            <tr><th>bla1</th><th>bla2</th></tr>
            <tr><td>dat1</td><td>dat2</td></tr>
            //     <tr>
            //         {table.headline.iter().map(|header| view! { <th>{header}</th> }).collect::<Vec<_>>() }
            //     </tr>
            //     {table.rows.iter().map(|row| view! {
            //         <tr>
            //             {row.content.iter().map(|cell| view! { <td>{cell}</td> }).collect::<Vec<_>>() }
            //         </tr>
            //     }).collect::<Vec<_>>() }
            </table>
            </div>
            </p>
        })
    } else if let Some(Err(e)) = table {
        EitherOf3::B(view! {
            <p>{format!("Error: {}", e)}</p>
        })
    } else {
        EitherOf3::C(view! {
            <p>"Loading table..."</p>
        })
    }
}
