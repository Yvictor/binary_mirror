# binary-mirror

A derive macro for parsing fixed-length binary data structures. This crate provides a convenient way to work with fixed-length binary data formats, commonly used in financial market data and legacy systems.

## Features

- Parse fixed-length binary data into Rust structs
- Support for various data types:
  - Strings (`str`)
  - Numbers (`i32`, `i64`, `u32`, `u64`, `f32`, `f64`)
  - Decimals
  - Dates and Times
  - Custom Enums
- Debug and Display implementations
- Zero-copy parsing
- Custom field aliases
- Skip fields in display output

## Installation
```
cargo add binary-mirror binary-mirror-derive
```

## Usage Examples

### Basic Structure

``` rust
use binary_mirror_derive::BinaryMirror;

#[repr(C)]
#[derive(BinaryMirror)]
struct Trade {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32")]
    value: [u8; 4],
    #[bm(type = "f32")]
    qty: [u8; 5],
}

let trade = Trade {
    name: b"AAPL ",
    value: b"123 ",
    qty: b"123.4",
};
assert_eq!(trade.name(), "AAPL");
assert_eq!(trade.value(), Some(123));
assert_eq!(trade.qty(), Some(123.4));
```


### Custom Enums

``` rust
use binary_mirror_derive::{BinaryMirror, BinaryEnum};

#[derive(Debug, PartialEq, BinaryEnum)]
enum OrderSide {
    #[bv(value = b"B")]
    Buy,
    #[bv(value = b"S")]
    Sell,
}
// Default first character behavior
#[derive(Debug, PartialEq, BinaryEnum)]
enum Direction {
    Up, // Will use b'U'
    Down, // Will use b'D'
}

#[repr(C)]
#[derive(BinaryMirror)]
struct Order {
    #[bm(type = "enum", enum_type = "OrderSide")]
    side: [u8; 1],
}
let order = Order { side: b"B" };

assert_eq!(order.side(), Some(OrderSide::Buy));
```

### Date and Time Handling

``` rust
#[repr(C)]
#[derive(BinaryMirror)]
struct MarketData {
    #[bm(type = "date", format = "%Y%m%d")]
    date: [u8; 8],
    #[bm(type = "time", format = "%H%M%S")]
    time: [u8; 6],
    // Combine date and time into a datetime
    #[bm(type = "date", format = "%Y%m%d", datetime_with = "time", alias = "datetime")]
    trade_date: [u8; 8],
}
let data = MarketData {
    date: b"20240101",
    time: b"123456",
    trade_date: b"20240101",
};
assert_eq!(data.trade_date(), Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
assert_eq!(data.time(), Some(NaiveTime::from_hms_opt(12, 34, 56).unwrap()));
assert_eq!(
    data.datetime(),
    Some(NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 34, 56).unwrap()
    ))
);

```

### Field Aliases and Skip
``` rust
#[repr(C)]
#[derive(BinaryMirror)]
struct Quote {
    #[bm(type = "str", alias = "exchange")]
    exh: [u8; 4],
    #[bm(type = "str", skip = true)] // Skip in Display output
    internal_code: [u8; 10],
}

let quote = Quote {
    exh: b"NYSE",
    internal_code: b"SECRET ",
};
assert_eq!(quote.exchange(), "NYSE");
```

### Parse from Bytes

``` rust
#[repr(C)]
#[derive(BinaryMirror)]
struct Data {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32")]
    value: [u8; 4],
}

let bytes = b"Hello 123 ";
let data = Data::from_bytes(bytes).expect("Invalid data");
assert_eq!(data.name(), "Hello");
assert_eq!(data.value(), Some(123));

// Size mismatch error handling
let wrong_size = b"too short";
let err = Data::from_bytes(wrong_size).unwrap_err();
println!("{}", err); // Will show size mismatch and content
```


## Safety

The `from_bytes` method uses unsafe code to create a reference to the struct. It's safe when:
1. The struct is marked with `#[repr(C)]`
2. The input bytes match the exact size of the struct
3. The bytes represent a valid instance of the struct