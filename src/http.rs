use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt as _;
use crate::metrics::snapshot;
use actix_ws::{Message, handle};
use tokio::time::{interval, Duration};

#[get("/metrics")]
async fn metrics() -> impl Responder {
    match snapshot().await {
        Ok(m)  => HttpResponse::Ok().json(m),          // application/json
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("error: {e}")),
    }
}

#[get("/ws")]
pub async fn ws(req: HttpRequest, body: web::Payload) -> Result<HttpResponse, Error> {
    let (response, mut session, mut stream) = handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match snapshot().await {
                        Ok(m) => {
                            if session.text(serde_json::to_string(&m).unwrap()).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            let _ = session.close(None).await;
                            break;
                        }
                    }
                }
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(reason))) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(_)) | None => {
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(response)
}

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(metrics).service(ws))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
