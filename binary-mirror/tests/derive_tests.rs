use binary_mirror::BinaryMirror;

#[repr(C)]
#[derive(Debug, BinaryMirror)]
struct TestStruct {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32")]
    value: [u8; 4],
    no_type: [u8; 7],
}

#[test]
fn test_struct_derivation() {
    let test = TestStruct {
        name: *b"Hello     ",
        value: *b"123 ",
        no_type: *b"no_type",
    };
    
    assert_eq!(test.name(), "Hello");
    assert_eq!(test.value(), Some(123));
}

#[test]
fn test_invalid_number() {
    let test = TestStruct {
        name: *b"Test      ",
        value: *b"abc\0",
        no_type: *b"no_type",
    };
    
    assert_eq!(test.name(), "Test");
    assert_eq!(test.value(), None);
} 