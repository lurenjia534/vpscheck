use actix_web::{App, HttpServer};

pub mod routes;

pub use routes::{metrics, ws};

pub async fn run() -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p: String| p.parse().ok())
        .unwrap_or(8080);
    let server = HttpServer::new(|| App::new().service(metrics).service(ws))
        .bind(("0.0.0.0", port))?;
    for addr in server.addrs() {
        println!("Web 服务器已启动，监听地址：http://{}", addr);
        println!("WebSocket 服务器已启动，监听地址：ws://{}", addr);
    }
    server.run().await
}
