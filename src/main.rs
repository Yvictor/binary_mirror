use binary_mirror_derive::{BinaryMirror, BinaryEnum};
use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, BinaryEnum, Serialize, Deserialize)]
enum OrderSide {
    #[bv(value = b"B")]
    Buy,
    #[bv(value = b"S")]
    Sell,
}

#[repr(C)]
#[derive(BinaryMirror)]
pub struct SomePayload {
    #[bm(type = "str")]
    company: [u8; 10],
    #[bm(type = "str", alias = "exchange")]
    exh: [u8; 8],
    #[bm(type = "decimal")]
    stkprc1: [u8; 20],
    #[bm(type = "i32")]
    ordqty: [u8; 4],
    #[bm(type = "enum", enum_type = "OrderSide")]
    side: [u8; 1],
    // #[bm(type = "date", format = "%Y%m%d")]
    // date: [u8; 8],
    // #[bm(type = "time", format = "%H%M%S")]
    // time: [u8; 6],
    #[bm(type = "date", format = "%Y%m%d", datetime_with = "time", alias = "datetime", skip = true)]
    date: [u8; 8],
    #[bm(type = "time", format = "%H%M%S", skip = true)]
    time: [u8; 6],
    #[bm(type = "i32")]
    err_case: [u8; 4],
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
        company: *b"COMPANY   ",
        exh: *b"EXCHANGE",
        stkprc1: *b"1234.56             ",
        ordqty: *b"1234",
        side: *b"B",
        date: *b"20240101",
        time: *b"123456",
        err_case: *b"12xx",
    };
    
    println!("{:?}", payload);
    println!("{}", payload);
    let native = payload.to_native();
    let json = serde_json::to_string(&native).unwrap();
    println!("{}", json);
    let parsed = serde_json::from_str::<SomePayloadNative>(&json).unwrap();
    println!("{:?}", parsed);
    let payload_from_native = SomePayload::from_native(&parsed);
    println!("{}", payload_from_native);
}
