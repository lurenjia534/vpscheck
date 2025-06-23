use vpscheck::web;      // ← 库 crate 的名字就是 package name

#[tokio::main]
async fn main() -> std::io::Result<()> {
    web::run().await
}
