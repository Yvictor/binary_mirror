use binary_mirror_derive::BinaryMirror;

#[repr(C)]
#[derive(BinaryMirror)]
pub struct SomePayload {
    #[bm(type = "str")]
    company: [u8; 7],
    #[bm(type = "str")]
    exh: [u8; 8],
    #[bm(type = "str")]
    stkprc1: [u8; 20],
    #[bm(type = "i32")]
    ordqty: [u8; 4],
}

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
    let payload = SomePayload {
        company: *b"COMPANY",
        exh: *b"EXCHANGE",
        stkprc1: *b"1234.56             ",
        ordqty: *b"1234",
    };
    
    println!("{:?}", payload);
}
