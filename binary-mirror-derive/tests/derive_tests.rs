use binary_mirror_derive::{BinaryEnum, BinaryMirror};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinaryEnum, Serialize, Deserialize)]
enum OrderSide {
    #[bv(value = b"B")]
    Buy,
    #[bv(value = b"S")]
    Sell,
}

#[derive(Debug, PartialEq, BinaryEnum, Serialize, Deserialize)]
enum Direction {
    Up,   // Will use b'U'
    Down, // Will use b'D'
}

#[repr(C)]
#[derive(BinaryMirror)]
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
    #[bm(
        type = "date",
        format = "%Y%m%d",
        datetime_with = "time",
        alias = "datetime",
        skip = true
    )]
    date: [u8; 8],
    #[bm(type = "time", format = "%H%M%S", skip = true)]
    time: [u8; 6],
    #[bm(type = "enum", enum_type = "OrderSide")]
    side: [u8; 1],
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
        date: *b"20240101",
        time: *b"123456",
        side: *b"B",
    };
    println!("{:?}", test);

    assert_eq!(test.name(), "Hello");
    assert_eq!(test.value(), Some(123));
    assert_eq!(test.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(test.f32(), Some(123.4));
    assert_eq!(test.exchange(), "CME".to_string());
    assert_eq!(
        test.date(),
        Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
    );
    assert_eq!(
        test.time(),
        Some(NaiveTime::from_hms_opt(12, 34, 56).unwrap())
    );
    assert_eq!(
        test.datetime(),
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap()
        ))
    );
    assert_eq!(test.side(), Some(OrderSide::Buy));
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
        date: *b"xxxxxxxx",
        time: *b"xxxxxx",
        side: *b" ",
    };

    assert_eq!(test.name(), "Test");
    assert_eq!(test.value(), None);
    assert_eq!(test.decimal(), None);
    assert_eq!(test.f32(), None);
    assert_eq!(test.datetime(), None);
    assert_eq!(test.date(), None);
    assert_eq!(test.time(), None);
    assert_eq!(test.side(), None);
}

#[test]
fn test_debug_format() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"no_type",
        decimal: *b"000000123.4500000000",
        f32: *b"123.4",
        exh: *b"CME       ",
        date: *b"20240101",
        time: *b"123456",
        side: *b"B",
    };

    println!("{:#?}", test);
}

#[test]
fn test_display_format() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"no_type",
        decimal: *b"000000123.4500000000",
        f32: *b"123.4",
        exh: *b"CME       ",
        date: *b"20240101",
        time: *b"123456",
        side: *b"B",
    };

    // Will print:
    // TestStruct { name: Hello, value: 123, decimal: 123.45, f32: 123.4, exchange: CME, datetime: 2024-01-01T12:34:56 }
    assert_eq!(format!("{}", test), "TestStruct { name: Hello, value: 123, decimal: 123.45, f32: 123.4, exchange: CME, datetime: 2024-01-01T12:34:56, side: Buy }");

    let invalid = TestStruct {
        name: *b"Test      ",
        value: *b"abc\0",
        no_type: *b"no_type",
        decimal: *b"000000123.xxxxxxxxxx",
        f32: *b"123.x",
        exh: *b"CME       ",
        date: *b"xxxxxxxx",
        time: *b"xxxxxx",
        side: *b" ",
    };

    // Will print:
    // TestStruct { name: Test, value: <invalid>, decimal: <invalid>, f32: <invalid>, exchange: CME, date: <invalid>, datetime: <invalid>, time: <invalid> }
    println!("{}", invalid);
    assert_eq!(format!("{}", invalid), "TestStruct { name: Test, value: <invalid>, decimal: <invalid>, f32: <invalid>, exchange: CME, datetime: <invalid>, side: <invalid> }");
}

#[test]
fn test_binary_enum() {
    // Test custom byte values
    assert_eq!(OrderSide::from_bytes(b"B"), Some(OrderSide::Buy));
    assert_eq!(OrderSide::from_bytes(b"S"), Some(OrderSide::Sell));
    assert_eq!(OrderSide::from_bytes(b"X"), None);

    // Test default behavior
    assert_eq!(Direction::from_bytes(b"U"), Some(Direction::Up));
    assert_eq!(Direction::from_bytes(b"D"), Some(Direction::Down));
    assert_eq!(Direction::from_bytes(b"X"), None);
}

#[test]
fn test_struct_from_bytes() {
    let bytes = b"Hello     123 no_type000000123.4500000000123.4CME       20240101123456B";
    let test = TestStruct::from_bytes(bytes).expect("Should parse successfully");
    assert_eq!(test.name(), "Hello");
    assert_eq!(test.value(), Some(123));

    // Test wrong size
    let wrong_size = b"too short with some \xff special \x00 bytes";
    let err = TestStruct::from_bytes(wrong_size).unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "bytes size mismatch: expected {} bytes but got {} bytes, content: \"{}\"",
            TestStruct::size(),
            wrong_size.len(),
            "too short with some \\xff special \\x00 bytes"
        )
    );
}

#[test]
fn test_serde_serialization() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"no_type",
        decimal: *b"000000123.4500000000",
        f32: *b"123.4",
        exh: *b"CME       ",
        date: *b"20240101",
        time: *b"123456",
        side: *b"B",
    };
    let test_native = test.to_native();
    // Test JSON serialization
    let json = serde_json::to_string(&test_native).unwrap();
    assert_eq!(
        json,
        r#"{"name":"Hello","value":123,"decimal":"123.45","f32":123.4,"exchange":"CME","datetime":"2024-01-01T12:34:56","side":"Buy"}"#
    );

    // Test invalid values
    let invalid = TestStruct {
        name: *b"Test      ",
        value: *b"abc\0",
        no_type: *b"no_type",
        decimal: *b"000000123.xxxxxxxxxx",
        f32: *b"123.x",
        exh: *b"CME       ",
        date: *b"xxxxxxxx",
        time: *b"xxxxxx",
        side: *b" ",
    };
    let test_native = invalid.to_native();
    let json = serde_json::to_string(&test_native).unwrap();
    assert_eq!(
        json,
        r#"{"name":"Test","value":null,"decimal":null,"f32":null,"exchange":"CME","datetime":null,"side":null}"#
    );
}

#[test]
fn test_struct_to_bytes() {
    let original_bytes = b"Hello     123 no_type000000123.4500000000123.4CME       20240101123456B";
    let test = TestStruct::from_bytes(original_bytes).expect("Should parse successfully");

    // Convert back to bytes
    let bytes = test.to_bytes();
    assert_eq!(bytes, original_bytes);

    // Test that we can parse the bytes back
    let reparsed = TestStruct::from_bytes(test.to_bytes()).unwrap();
    assert_eq!(reparsed.name(), test.name());
    assert_eq!(reparsed.value(), test.value());
    assert_eq!(reparsed.decimal(), test.decimal());
    assert_eq!(reparsed.f32(), test.f32());
    assert_eq!(reparsed.exchange(), test.exchange());
    assert_eq!(reparsed.date(), test.date());
    assert_eq!(reparsed.time(), test.time());
    assert_eq!(reparsed.side(), test.side());
}

#[test]
fn test_struct_to_bytes_owned() {
    let test = TestStruct::from_bytes(
        b"Hello     123 no_type000000123.4500000000123.4CME       20240101123456B",
    )
    .unwrap();
    let bytes = test.to_bytes_owned();
    assert_eq!(
        bytes,
        b"Hello     123 no_type000000123.4500000000123.4CME       20240101123456B"
    );
}
