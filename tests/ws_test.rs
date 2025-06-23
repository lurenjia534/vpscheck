use actix_web::{App};
use actix_test::start;
use awc::ws::Frame;
use futures_util::StreamExt as _;
use std::time::Duration;
use tokio::time::timeout;

#[actix_web::test]
async fn websocket_streams_metrics() {
    let mut srv = start(|| App::new().service(vpscheck::web::ws));
    let mut framed = srv.ws_at("/ws").await.expect("connect");

    let frame = timeout(Duration::from_secs(3), framed.next())
        .await
        .expect("timed out")
        .expect("stream closed")
        .expect("frame error");

    match frame {
        Frame::Text(bytes) => {
            let _: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        }
        other => panic!("unexpected frame: {:?}", other),
    }
}
