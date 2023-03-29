use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let config = RustlsConfig::from_pem_file("localhost.pem", "localhost.key")
        .await
        .unwrap();

    // axum::Server::bind(&"0.0.0.0:80".parse().unwrap())
    axum_server::bind_rustls("0.0.0.0:8080".parse().unwrap(), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
