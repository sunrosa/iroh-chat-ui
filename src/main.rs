use dioxus::prelude::*;

use iroh_chat::Event;

static CHATLOG: GlobalSignal<Vec<String>> = Signal::global(Vec::new);

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main());
}

async fn async_main() {}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Input {}
        Chatbox {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
        }

        Outlet::<Route> {}
    }
}

#[component]
fn Chatbox() -> Element {
    rsx! {
        div {
            h2 { "chat" }
            for x in CHATLOG() {
                p { "{x}" }
            }
        }
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Input() -> Element {
    let mut input = use_signal(String::new);

    rsx! {
        div { id: "echo",
            input {
                placeholder: "Send message...",
                value: "{input()}",
                oninput: move |event| async move {
                    let data = input_text(event.value()).await.unwrap();
                    input.set(data);
                },
                onkeypress: move |event| async move {
                    if event.key() == Key::Enter {
                        CHATLOG.write().push(input());
                        input.set(String::new());
                    }
                },
            }
        }
    }
}

/// Echo the user input on the server.
#[server]
async fn input_text(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
