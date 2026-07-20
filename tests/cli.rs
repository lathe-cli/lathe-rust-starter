use std::process::Output;

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    process::Command,
    task::JoinHandle,
};

async fn start_server() -> (String, String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let hostname = format!("http://{address}");
    let handle = tokio::spawn(async move {
        axum::serve(listener, cli_first_rust_app::app())
            .await
            .unwrap();
    });
    (hostname, address, handle)
}

async fn run_cli(hostname: &str, args: &[&str]) -> Output {
    Command::new("./bin/appctl")
        .arg("--hostname")
        .arg(hostname)
        .args(args)
        .arg("-o")
        .arg("json")
        .output()
        .await
        .unwrap()
}

async fn cli(hostname: &str, args: &[&str]) -> Option<Value> {
    let output = run_cli(hostname, args).await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (!output.stdout.is_empty()).then(|| serde_json::from_slice(&output.stdout).unwrap())
}

async fn cli_failure(hostname: &str, args: &[&str]) -> String {
    let output = run_cli(hostname, args).await;
    assert!(!output.status.success());
    serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"]["message"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn generated_cli_is_the_application_acceptance_surface() {
    let (hostname, _, server) = start_server().await;
    assert_eq!(
        cli(&hostname, &["health", "get"]).await,
        Some(json!({ "status": "ok" }))
    );

    let created = cli(
        &hostname,
        &["tasks", "create", "--set", "title=Ship from the CLI"],
    )
    .await
    .unwrap();
    assert_eq!(
        created,
        json!({ "id": "1", "title": "Ship from the CLI", "completed": false })
    );
    assert_eq!(
        cli(&hostname, &["tasks", "list"]).await,
        Some(json!([created]))
    );
    assert_eq!(
        cli(&hostname, &["tasks", "get", "--id", "1"]).await,
        Some(created.clone())
    );
    let updated = cli(
        &hostname,
        &["tasks", "update", "--id", "1", "--set", "completed=true"],
    )
    .await;
    assert_eq!(
        updated,
        Some(json!({ "id": "1", "title": "Ship from the CLI", "completed": true }))
    );
    cli(&hostname, &["tasks", "delete", "--id", "1"]).await;
    assert_eq!(cli(&hostname, &["tasks", "list"]).await, Some(json!([])));
    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn generated_cli_surfaces_api_errors() {
    let (hostname, _, server) = start_server().await;
    assert!(
        cli_failure(&hostname, &["tasks", "create", "--set-str", "title="])
            .await
            .contains("HTTP 400")
    );
    assert!(
        cli_failure(&hostname, &["tasks", "get", "--id", "missing"])
            .await
            .contains("HTTP 404")
    );
    cli(
        &hostname,
        &["tasks", "create", "--set", "title=Keep the contract honest"],
    )
    .await;
    assert!(
        cli_failure(
            &hostname,
            &["tasks", "update", "--id", "1", "--file", "test/empty.json"],
        )
        .await
        .contains("title or completed is required")
    );
    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn http_boundary_rejects_unsupported_input() {
    let (_, address, server) = start_server().await;
    let (status, headers, _) = request(&address, "PUT", "/tasks", None, "").await;
    assert_eq!(status, 405);
    let allow = headers.to_ascii_lowercase();
    assert!(allow.contains("get"));
    assert!(allow.contains("post"));

    let (status, _, body) = request(
        &address,
        "POST",
        "/tasks",
        Some("text/plain"),
        r#"{"title":"wrong media type"}"#,
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body, r#"{"error":"content-type must be application/json"}"#);

    let oversized = format!(r#"{{"title":"{}"}}"#, "x".repeat(1_000_001));
    let (status, _, body) = request(
        &address,
        "POST",
        "/tasks",
        Some("application/json"),
        &oversized,
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body, r#"{"error":"request body exceeds 1 MB"}"#);
    server.abort();
}

async fn request(
    address: &str,
    method: &str,
    path: &str,
    content_type: Option<&str>,
    body: &str,
) -> (u16, String, String) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    let content_type = content_type
        .map(|value| format!("Content-Type: {value}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n{content_type}Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    let (headers, body) = response.split_once("\r\n\r\n").unwrap();
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, headers.to_owned(), body.to_owned())
}
