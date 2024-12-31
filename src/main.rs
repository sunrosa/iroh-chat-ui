use std::{
    any::{type_name, type_name_of_val, Any},
    str::FromStr,
};

use dioxus::prelude::*;

use iroh::{endpoint::Connection, Endpoint, NodeAddr, PublicKey};
use iroh_chat::Event;

#[derive(Clone, PartialEq, Eq, Default)]
struct Context {
    chatlog: Signal<Vec<String>>,
}

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
        .block_on(async_main())
        .unwrap();
}

async fn async_main() -> anyhow::Result<()> {
    let public_key = include_str!("public_key").trim();

    let addr = NodeAddr::new(PublicKey::from_str(public_key)?);
    let ep = Endpoint::builder().discovery_n0().bind().await?;

    // Keep retrying connection until it's made
    let conn = loop {
        match ep.connect(addr.clone(), b"my-alpn").await {
            Ok(o) => break o,
            Err(_) => continue,
        }
    };

    // Ctrl-C handler to close connection
    tokio::task::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        ep.close().await.unwrap();
        std::process::exit(0);
    });

    {
        let conn = conn.clone();
        tokio::task::spawn(async move {
            receiver(conn)
                .await
                .unwrap_or_else(|e| panic!("{e}\n{e:?}"));
        });
    }

    let mut send = conn.open_uni().await?;
    send.write_all(&serde_json::to_vec(&Event::Connected(
        conn.stable_id().to_string(),
    ))?)
    .await?;
    send.finish()?;

    // loop {
    //     let mut buf = String::new();
    //     std::io::stdin().read_line(&mut buf)?;
    //     let buf = buf.trim();

    //     let event: Event = Event::Chat(conn.stable_id().to_string(), buf.into());

    //     let mut send = conn.open_uni().await?;
    //     send.write_all(&serde_json::to_vec(&event)?).await?;
    //     send.finish()?;
    // }

    Ok(())
}

async fn receiver(conn: Connection) -> anyhow::Result<()> {
    loop {
        let mut recv = conn.accept_uni().await?;

        let received = recv.read_to_end(8192).await?;
        let message: Event = serde_json::from_slice(&received)?;

        match message {
            Event::Chat(sender, content) => println!("{sender}: {content}"),
            Event::Connected(c) => println!("{c} connected."),
            Event::Disconnected(d) => println!("{d} disconnected."),
        }
    }
}

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
    // The documentation that says to use_context_provider is WRONG. This is how you initially provide a context without it throwing a panic. Jesus fucking christ. I spent 6 hours trying to figure this out.
    provide_context(Context::default());

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
    let chatlog = use_context::<Context>().chatlog;

    rsx! {
        div {
            h2 { "chat" }
            for x in chatlog() {
                p { "{x}" }
            }
        }
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Input() -> Element {
    let mut input = use_signal(String::new);
    let mut chatlog = use_context::<Context>().chatlog;

    rsx! {
        div { id: "echo",
            input {
                placeholder: "Send message...",
                value: "{input()}",
                oninput: move |event| async move {
                    let data = event.value();
                    input.set(data);
                },
                onkeypress: move |event| async move {
                    if event.key() == Key::Enter {
                        chatlog.write().push(input());
                        input.set(String::new());
                    }
                },
            }
        }
    }
}
