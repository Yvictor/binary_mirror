// #[tokio::main]
// async 
fn main() {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .init();

//     // 同時運行伺服器和客戶端
//     tokio::join!(
//         server::run_query_server(),
//         server::run_order_server(),
//         client::run_query_client(),
//         client::run_order_client()
//     );
}
