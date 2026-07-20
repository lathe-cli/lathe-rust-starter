use std::env;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let port: u16 = port
        .parse()
        .ok()
        .filter(|port| *port > 0)
        .expect("PORT must be an integer between 1 and 65535");
    let address = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("bind application server");
    println!("API listening on http://{address}");
    axum::serve(listener, cli_first_rust_app::app())
        .await
        .expect("serve application");
}
