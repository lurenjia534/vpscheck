use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use crate::metrics::snapshot;

#[get("/metrics")]
async fn metrics() -> impl Responder {
    match snapshot().await {
        Ok(m)  => HttpResponse::Ok().json(m),          // application/json
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("error: {e}")),
    }
}

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(metrics))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
