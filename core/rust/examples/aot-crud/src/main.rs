#[tokio::main]
async fn main() {
    let app = clrinf_aot_crud::build_app();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8085")
        .await
        .expect("bind port 8085");
    println!("clrinf-aot-crud listening on http://127.0.0.1:8085");
    axum::serve(listener, app).await.expect("serve app");
}
