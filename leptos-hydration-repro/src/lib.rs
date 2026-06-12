//! Minimal repro for a tachys hydration marker desync (leptos 0.8.19 /
//! tachys 0.2.15): "expected an HTML <div> element, found a node of an
//! unexpected type" + panic at hydration.rs:184, on a page whose structure is
//!
//!   OnceResource::new_blocking → Signal::derive
//!     → {move || …closure…} returning a view! that BOTH
//!         - contains a nested dynamic {…collect_view()} block, and
//!         - renders components from the resource-derived list.
//!
//! `cargo leptos serve` → http://127.0.0.1:3111 → hard reload → console.

use leptos::hydration::HydrationScripts;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
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
    provide_meta_context();
    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=StaticSegment("") view=Page/>
            </Routes>
        </Router>
    }
}

#[component]
fn Page() -> impl IntoView {
    // Outer blocking resource + Suspense (the "team detail" layer).
    let detail = OnceResource::new_blocking(async move { "Repro Team".to_string() });

    view! {
        <div>
            <p>"Ride Groups"</p>
            <Suspense fallback=|| ()>
                {move || detail.get().map(|name| view! {
                    <h2>{name.clone()}</h2>
                    <Inner/>
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn Inner() -> impl IntoView {
    // A SECOND blocking resource, created inside the Suspense-rendered child —
    // exactly how the real page creates its groups list.
    let groups_once = OnceResource::new_blocking(async move {
        vec!["Group 1".to_string()]
    });
    view! {
        {move || {
            let list = groups_once.get().unwrap_or_default();
            list.into_iter()
                .map(|g| view! { <div><div>{g}</div><div>"body"</div></div> })
                .collect_view()
        }}
    }
}

#[component]
fn ZoneView(label: String) -> impl IntoView {
    view! {
        <div style="margin-bottom: 20px;">
            <div>{label.clone()}</div>
            <div>"body"</div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
