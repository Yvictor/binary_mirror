use binary_mirror::BinaryMirror;
use rust_decimal::prelude::*;

#[repr(C)]
#[derive(Debug, BinaryMirror)]
struct TestStruct {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32")]
    value: [u8; 4],
    no_type: [u8; 7],
    #[bm(type = "decimal")]
    decimal: [u8; 20],
    #[bm(type = "f32")]
    f32: [u8; 5],
    #[bm(type = "str", alias = "exchange")]
    exh: [u8; 10],
}

#[test]
fn test_struct_derivation() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"no_type",
        decimal: *b"000000123.4500000000",
        f32: *b"123.4",
        exh: *b"CME       ",
    };
    
    assert_eq!(test.name(), "Hello");
    assert_eq!(test.value(), Some(123));
    assert_eq!(test.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(test.f32(), Some(123.4));
    assert_eq!(test.exchange(), "CME".to_string());
}

#[test]
fn test_invalid_number() {
    let test = TestStruct {
        name: *b"Test      ",
        value: *b"abc\0",
        no_type: *b"no_type",
        decimal: *b"000000123.xxxxxxxxxx",
        f32: *b"123.x",
        exh: *b"CME       ",
    };
    
    assert_eq!(test.name(), "Test");
    assert_eq!(test.value(), None);
    assert_eq!(test.decimal(), None);
    assert_eq!(test.f32(), None);
} 
