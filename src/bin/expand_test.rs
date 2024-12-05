use binary_mirror::BinaryMirror;

#[repr(C)]
#[derive(Debug, BinaryMirror)]
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

fn main() {
    let payload = SomePayload {
        company: *b"COMPANY",
        exh: *b"EXCHANGE",
        stkprc1: *b"1234.56             ",
        ordqty: *b"1234",
    };
    
    println!("{:?}", payload);
} 