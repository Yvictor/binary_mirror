use binary_mirror_derive::{BinaryEnum, BinaryMirror};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use binary_mirror::{FromBytes, ToBytes, ToNative, FromNative, NativeStructCode};

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

#[derive(Debug, PartialEq, BinaryEnum, Serialize, Deserialize)]
enum OrderType {
    #[bv(value = b"MKT")]
    Market,
    #[bv(value = b"LMT")]
    Limit,
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

#[repr(C)]
#[derive(BinaryMirror)]
struct WithDateTime {
    #[bm(type = "datetime", format = "%Y%m%d%H%M%S")]
    dt: [u8; 14],
}

#[test]
fn test_with_datetime() {
    let dt = WithDateTime {
        dt: *b"20240101123456",
    };
    println!("{:?}", dt);
    assert_eq!(
        dt.dt(),
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap()
        ))
    );
}

#[derive(BinaryMirror)]
struct WithDefaultByte {
    #[bm(type = "str", default_byte = b'0')]
    zero: [u8; 10],
}

#[test]
fn test_with_default_byte() {
    let default_ = WithDefaultByteNative {
        zero: String::from("11"),
    };
    println!("{:?}", default_);
    let raw = WithDefaultByte::from_native(&default_);
    assert_eq!(raw.zero(), "1100000000");
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithDefaultBytes {
    #[bm(type = "str", default_byte = b'\x08')] // 0x08 hex
    hex: [u8; 5],
    #[bm(type = "str", default_byte = b'\x00')] // Null in hex
    null: [u8; 5],
    #[bm(type = "str", default_byte = b'0')] // Regular byte literal
    zero: [u8; 5],
}

#[test]
fn test_default_bytes() {
    let native = WithDefaultBytesNative::default()
        .with_hex("abc")
        .with_null("abc")
        .with_zero("abc");

    let binary = WithDefaultBytes::from_native(&native);

    assert_eq!(&binary.hex, b"abc\x08\x08"); // Padded with spaces (0x08)
    assert_eq!(&binary.null, b"abc\x00\x00"); // Padded with nulls (0x00)
    assert_eq!(&binary.zero, b"abc00"); // Padded with '0' chars
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
    assert_eq!(format!("{}", test), "TestStruct { name: Hello, value: 123, decimal: 123.45, f32: 123.4, exchange: CME, datetime: 2024-01-01 12:34:56, side: Buy }");

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
    //TestStruct { name: Test, value: Error<bytes: "abc\x00">, decimal: Error<bytes: "000000123.xxxxxxxxxx">, f32: Error<bytes: "123.x">, exchange: CME, datetime: Error<bytes: "xxxxxxxx">, side: Error<bytes: " "> }
    println!("{}", invalid);
    assert_eq!(format!("{}", invalid), "TestStruct { name: Test, value: Error<bytes: \"abc\\x00\">, decimal: Error<bytes: \"000000123.xxxxxxxxxx\">, f32: Error<bytes: \"123.x\">, exchange: CME, datetime: Error<bytes: \"xxxxxxxx\">, side: Error<bytes: \" \"> }");
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
fn test_to_native() {
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
    let native = test.to_native();
    assert_eq!(native.name, test.name());
}

#[test]
fn test_struct_from_native() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"       ",
        // no_type: *b"no_type",
        decimal: *b"123.45              ",
        // decimal: *b"000000123.4500000000",
        f32: *b"123.4",
        exh: *b"CME       ",
        date: *b"20240101",
        time: *b"123456",
        side: *b"B",
    };
    let native = test.to_native();
    let parsed = TestStruct::from_native(&native);
    assert_eq!(parsed.name(), test.name());
    assert_eq!(parsed.value(), test.value());
    assert_eq!(parsed.decimal(), test.decimal());
    assert_eq!(parsed.f32(), test.f32());
    assert_eq!(parsed.exchange(), test.exchange());
    assert_eq!(parsed.datetime(), test.datetime());
    assert_eq!(parsed.side(), test.side());

    let bytes = test.to_bytes();
    let parsed_bytes = parsed.to_bytes();
    println!("{:?}", test);
    println!("{:?}", parsed);
    assert_eq!(bytes, parsed_bytes);
}

#[test]
fn test_native_default() {
    let native = TestStructNative::default();
    assert_eq!(native.name, "");
    assert_eq!(native.value, None);
    assert_eq!(native.decimal, None);
    assert_eq!(native.f32, None);
    assert_eq!(native.exchange, "");
    assert_eq!(native.datetime, None);
    assert_eq!(native.side, None);
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
    assert_eq!(reparsed.datetime(), test.datetime());
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

#[test]
fn test_binary_enum_roundtrip() {
    // Test custom byte values
    assert_eq!(OrderSide::Buy.as_bytes(), b"B");
    assert_eq!(OrderSide::Sell.as_bytes(), b"S");

    // Test roundtrip
    let buy = OrderSide::from_bytes(b"B").unwrap();
    assert_eq!(OrderSide::from_bytes(buy.as_bytes()), Some(OrderSide::Buy));

    let sell = OrderSide::from_bytes(b"S").unwrap();
    assert_eq!(
        OrderSide::from_bytes(sell.as_bytes()),
        Some(OrderSide::Sell)
    );

    // Test default behavior
    assert_eq!(Direction::Up.as_bytes(), b"U");
    assert_eq!(Direction::Down.as_bytes(), b"D");

    // Test roundtrip with default behavior
    let up = Direction::from_bytes(b"U").unwrap();
    assert_eq!(Direction::from_bytes(up.as_bytes()), Some(Direction::Up));

    let down = Direction::from_bytes(b"D").unwrap();
    assert_eq!(
        Direction::from_bytes(down.as_bytes()),
        Some(Direction::Down)
    );
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithIgnoreWarn {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32", ignore_warn = true)]
    value: [u8; 4],
}

#[test]
fn test_ignore_warn() {
    let test = WithIgnoreWarn {
        name: *b"Hello     ",
        value: *b"abc ", // Invalid value
    };

    let native = test.to_native();
    // No warning will be logged for value field
    assert_eq!(native.value, None);
}

#[test]
fn test_native_builder() {
    let native = TestStructNative::default()
        .with_name("AAPL")
        .with_value(123)
        .with_decimal(Decimal::from_str("123.45").unwrap())
        .with_f32(123.4)
        .with_exchange("NYSE")
        .with_datetime(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
        ))
        .with_side(OrderSide::Buy);

    assert_eq!(native.name, "AAPL");
    assert_eq!(native.value, Some(123));
    assert_eq!(native.decimal, Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(native.f32, Some(123.4));
    assert_eq!(native.exchange, "NYSE");
    assert_eq!(
        native.datetime,
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap()
        ))
    );
    assert_eq!(native.side, Some(OrderSide::Buy));

    // Convert to binary format
    let binary = TestStruct::from_native(&native);
    assert_eq!(binary.name(), "AAPL");
    assert_eq!(binary.value(), Some(123));
    assert_eq!(binary.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(binary.f32(), Some(123.4));
    assert_eq!(binary.exchange(), "NYSE");
    assert_eq!(
        binary.datetime(),
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap()
        ))
    );
    assert_eq!(binary.side(), Some(OrderSide::Buy));
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithNumberFormat {
    #[bm(type = "i32", format = "{:04}")] // Zero-padded 4 digits
    value: [u8; 4],
    #[bm(type = "f32", format = "{:09.3}")] // 2 decimal places
    price: [u8; 9],
    #[bm(type = "f32", format = "{:010.3}")]
    nagtive: [u8; 10],
    #[bm(type = "decimal", format = "{:010.3}")]
    decimal: [u8; 10],
    #[bm(type = "decimal", format = "{:010.3}")]
    decimal_with_neg: [u8; 10],
}

#[test]
fn test_number_format() {
    let native = WithNumberFormatNative::default()
        .with_value(42)
        .with_price(123.45)
        .with_nagtive(-123.45)
        .with_decimal(Decimal::from_str("123.45").unwrap())
        .with_decimal_with_neg(Decimal::from_str("-123.45").unwrap());

    let binary = WithNumberFormat::from_native(&native);
    assert_eq!(binary.value(), Some(42));
    assert_eq!(binary.price(), Some(123.45)); // Rounded to 2 decimals
    assert_eq!(binary.nagtive(), Some(-123.45));
    assert_eq!(binary.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(
        binary.decimal_with_neg(),
        Some(Decimal::from_str("-123.45").unwrap())
    );
    // Check raw bytes format
    println!("{:?}", binary);
    assert_eq!(&binary.value, b"0042");
    assert_eq!(&binary.price, b"00123.450");
    assert_eq!(&binary.nagtive, b"-00123.450");
    assert_eq!(&binary.decimal, b"000123.450");
    assert_eq!(&binary.decimal_with_neg, b"-00123.450");
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithBytes {
    #[bm(type = "bytes")]
    raw: [u8; 10],
    #[bm(type = "bytes", default_byte = b'0')]
    padded: [u8; 5],
}

#[test]
fn test_bytes() {
    let native = WithBytesNative::default()
        .with_raw([1, 2, 3, 4, 5, b' ', b' ', b' ', b' ', b' '])
        .with_padded([0xFF, 0xFE, b'0', b'0', b'0']);

    let binary = WithBytes::from_native(&native);
    assert_eq!(binary.raw(), [1, 2, 3, 4, 5, b' ', b' ', b' ', b' ', b' ']);
    assert_eq!(binary.padded(), [0xFF, 0xFE, b'0', b'0', b'0']);

    // Test raw bytes format
    assert_eq!(&binary.raw, &[1, 2, 3, 4, 5, b' ', b' ', b' ', b' ', b' ']);
    assert_eq!(&binary.padded, &[0xFF, 0xFE, b'0', b'0', b'0']);
}

#[test]
fn test_bytes_serde() {
    let native = WithBytesNative {
        raw: [1, 2, 3, b' ', b' ', b' ', b' ', b' ', b' ', b' '],
        padded: [0xFF, 0xFE, b'0', b'0', b'0'],
    };

    let json = serde_json::to_string(&native).unwrap();
    assert_eq!(
        json,
        r#"{"raw":[1,2,3,32,32,32,32,32,32,32],"padded":[255,254,48,48,48]}"#
    );

    let parsed: WithBytesNative = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed.raw,
        [1, 2, 3, b' ', b' ', b' ', b' ', b' ', b' ', b' ']
    );
    assert_eq!(parsed.padded, [0xFF, 0xFE, b'0', b'0', b'0']);
}

#[test]
fn test_field_specs() {
    let name_spec = TestStruct::name_spec();
    assert_eq!(name_spec.offset, 0);
    assert_eq!(name_spec.limit, 10);
    assert_eq!(name_spec.size, 10);

    let value_spec = TestStruct::value_spec();
    assert_eq!(value_spec.offset, 10);
    assert_eq!(value_spec.limit, 14);
    assert_eq!(value_spec.size, 4);

    // Test field access using spec
    let bytes = b"Hello     123 no_type000000123.4500000000123.4CME       20240101123456B";
    let _test = TestStruct::from_bytes(bytes).unwrap();

    // Get field slices using spec
    let spec = TestStruct::name_spec();
    assert_eq!(&bytes[spec.offset..spec.limit], b"Hello     ");

    let spec = TestStruct::decimal_spec();
    assert_eq!(&bytes[spec.offset..spec.limit], b"000000123.4500000000");

    // Test field boundaries
    assert_eq!(
        TestStruct::name_spec().limit,
        TestStruct::value_spec().offset
    );
    assert_eq!(
        TestStruct::value_spec().limit,
        TestStruct::no_type_spec().offset
    );
}

#[test]
fn test_multi_byte_enum() {
    // Test multi-byte values
    assert_eq!(OrderType::from_bytes(b"MKT"), Some(OrderType::Market));
    assert_eq!(OrderType::from_bytes(b"LMT"), Some(OrderType::Limit));
    assert_eq!(OrderType::from_bytes(b"XXX"), None);

    // Test roundtrip
    let market = OrderType::Market;
    assert_eq!(market.as_bytes(), b"MKT");
    assert_eq!(
        OrderType::from_bytes(market.as_bytes()),
        Some(OrderType::Market)
    );

    let limit = OrderType::Limit;
    assert_eq!(limit.as_bytes(), b"LMT");
    assert_eq!(
        OrderType::from_bytes(limit.as_bytes()),
        Some(OrderType::Limit)
    );

    // Test in struct
    #[repr(C)]
    #[derive(BinaryMirror)]
    struct Order {
        #[bm(type = "enum", enum_type = "OrderType")]
        order_type: [u8; 3],
    }

    let order = Order {
        order_type: *b"MKT",
    };

    assert_eq!(order.order_type(), Some(OrderType::Market));
    let native = order.to_native();
    assert_eq!(native.order_type, Some(OrderType::Market));
}

fn default_i32() -> i32 {
    42
}

fn default_str() -> String {
    "UNKNOWN".to_string()
}

fn now() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_utc().with_nanosecond(0).unwrap()
}

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn now_time() -> chrono::NaiveTime {
    chrono::Local::now().time().with_nanosecond(0).unwrap()
}

fn order_type_default() -> OrderType {
    OrderType::Limit
}

fn decimal_default() -> Decimal {
    Decimal::from_str("123.45").unwrap()
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithDefaults {
    #[bm(type = "str", default_func = "default_str")]
    name: [u8; 10],
    #[bm(type = "i32", default_func = "default_i32")]
    value: [u8; 4],
    #[bm(type = "datetime", default_func = "now", format = "%Y%m%d%H%M%S")]
    timestamp: [u8; 14],
    #[bm(type = "date", default_func = "today", format = "%Y%m%d")]
    today: [u8; 8],
    #[bm(type = "time", default_func = "now_time", format = "%H%M%S")]
    time: [u8; 6],
    #[bm(
        type = "date",
        default_func = "now",
        format = "%Y-%m-%d",
        datetime_with = "tt",
        alias = "concate_datetime"
    )]
    date: [u8; 10],
    #[bm(type = "time", format = "%H:%M:%S")]
    tt: [u8; 8],
    #[bm(
        type = "enum",
        enum_type = "OrderType",
        default_func = "order_type_default"
    )]
    order_type: [u8; 3],
    #[bm(type = "decimal", default_func = "decimal_default")]
    decimal: [u8; 10],
}

#[test]
fn test_defaults() {
    let native = WithDefaultsNative::default();
    println!("{:?}", native);
    let binary = WithDefaults::from_native(&native);
    println!("{:?}", binary);
    assert_eq!(binary.name(), "UNKNOWN");
    assert_eq!(binary.value(), Some(42));
    assert_eq!(binary.timestamp(), Some(now()));
    assert_eq!(binary.today(), Some(today()));
    assert_eq!(binary.time(), Some(now_time()));
    assert_eq!(binary.concate_datetime(), Some(now()));
    assert_eq!(binary.order_type(), Some(OrderType::Limit));
    assert_eq!(binary.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    // Test overriding defaults
    let native = WithDefaultsNative::default()
        .with_name("TEST")
        .with_value(100)
        .with_decimal(Decimal::from_str("999.99").unwrap())
        .with_order_type(OrderType::Market);

    let binary = WithDefaults::from_native(&native);
    assert_eq!(binary.name(), "TEST");
    assert_eq!(binary.value(), Some(100));
    assert_eq!(binary.decimal(), Some(Decimal::from_str("999.99").unwrap()));
    assert_eq!(binary.order_type(), Some(OrderType::Market));
}

#[test]
fn test_native_to_raw() {
    // Create native struct with builder pattern
    let native = TestStructNative::default()
        .with_name("AAPL")
        .with_value(123)
        .with_decimal(Decimal::from_str("123.45").unwrap())
        .with_f32(123.4)
        .with_exchange("NYSE")
        .with_datetime(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 34, 56).unwrap(),
        ))
        .with_side(OrderSide::Buy);

    // Convert directly to raw binary struct
    let raw = native.to_raw();

    // Verify values
    assert_eq!(raw.name(), "AAPL");
    assert_eq!(raw.value(), Some(123));
    assert_eq!(raw.decimal(), Some(Decimal::from_str("123.45").unwrap()));
    assert_eq!(raw.f32(), Some(123.4));
    assert_eq!(raw.exchange(), "NYSE");
    assert_eq!(raw.datetime(), Some(NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 34, 56).unwrap()
    )));
    assert_eq!(raw.side(), Some(OrderSide::Buy));

    // Test roundtrip
    let native2 = raw.to_native();
    let raw2 = native2.to_raw();
    assert_eq!(raw.to_bytes(), raw2.to_bytes());
}

#[test]
fn test_native_struct_code() {
    let code = TestStruct::native_struct_code();
    assert_eq!(
        code,
        r#"pub struct TestStructNative {
    pub name: String,
    pub value: Option<i32>,
    pub decimal: Option<rust_decimal::Decimal>,
    pub f32: Option<f32>,
    pub exchange: String,
    pub datetime: Option<chrono::NaiveDateTime>,
    pub side: Option<OrderSide>,
}"#
    );
}


#[repr(C)]
#[derive(BinaryMirror)]
#[bm(derive(Debug, Clone))]
struct CustomDerives {
    #[bm(type = "str")]
    name: [u8; 10],
}

#[test]
fn test_custom_native_derives() {
    let code = CustomDerives::native_struct_code();
    assert_eq!(
        code,
        r#"pub struct CustomDerivesNative {
    pub name: String,
}"#
    );

    // Test that the actual struct has the same derives
    let native = CustomDerivesNative::default();
    let _cloned = native.clone(); // Should compile because we have Clone
    let _debug = format!("{:?}", native); // Should compile because we have Debug
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithSkippedFields {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32", skip = true, default_byte = b'\x33')]
    skipped_value: [u8; 4],
    #[bm(type = "str")]
    description: [u8; 20],
}

#[test]
fn test_skipped_fields() {
    let native = WithSkippedFieldsNative::default()
        .with_name("TEST")
        .with_description("Description");

    let raw = native.to_raw();
    let bytes = raw.to_bytes();
    let parsed = WithSkippedFields::from_bytes(&bytes).unwrap();

    // Verify values
    assert_eq!(raw.name(), "TEST");
    assert_eq!(raw.description(), "Description");
    assert_eq!(parsed.name(), "TEST");
    assert_eq!(parsed.description(), "Description");

    // Verify native struct code doesn't include skipped field
    let code = WithSkippedFields::native_struct_code();
    assert_eq!(
        code,
        r#"pub struct WithSkippedFieldsNative {
    pub name: String,
    pub description: String,
}"#
    );

    // Test roundtrip
    let native2 = raw.to_native();
    let raw2 = native2.to_raw();
    assert_eq!(raw.to_bytes(), raw2.to_bytes());
    assert_eq!(WithSkippedFields::size(), 34);
    assert_eq!(WithSkippedFields::SIZE, 34);
}

#[repr(C)]
#[derive(BinaryMirror)]
struct WithSkipNativeStruct {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32", skip_native = true)]
    raw_value: [u8; 4],
    #[bm(type = "str")]
    description: [u8; 5],
}

#[test]
fn test_skip_native() {
    let raw = WithSkipNativeStruct {
        name: *b"TEST      ",
        raw_value: *b"1234",
        description: *b"Descr",
    };

    // Field should be accessible in raw struct
    assert_eq!(raw.raw_value(), Some(1234));

    // Convert to native
    let native = raw.to_native();
    
    // Field should not appear in native struct
    let code = WithSkipNativeStruct::native_struct_code();
    assert_eq!(
        code,
        r#"pub struct WithSkipNativeStructNative {
    pub name: String,
    pub description: String,
}"#
    );

    // Roundtrip should preserve the raw value
    let raw2 = native.to_raw();
    assert_eq!(raw2.raw_value(), None);
}
