use actix_web::{App, HttpServer};

pub mod routes;

pub use routes::{metrics, ws};

pub async fn run() -> std::io::Result<()> {
    let server = HttpServer::new(|| App::new().service(metrics).service(ws))
        .bind(("0.0.0.0", 8080))?;
    for addr in server.addrs() {
        println!("Web 服务器已启动，监听地址：http://{}", addr);
        println!("WebSocket 服务器已启动，监听地址：ws://{}", addr);
    }
    server.run().await
}
