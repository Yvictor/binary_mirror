use binary_mirror::BinaryMirror;

#[repr(C)]
#[derive(Debug, BinaryMirror)]
struct TestStruct {
    #[bm(type = "str")]
    name: [u8; 10],
    #[bm(type = "i32")]
    value: [u8; 4],
}

#[test]
fn test_struct_derivation() {
    let test = TestStruct {
        name: *b"Hello\0\0\0\0\0",
        value: *b"123\0",
    };
    
    assert_eq!(test.name(), "Hello");
    assert_eq!(test.value(), Some(123));
}

#[test]
fn test_invalid_number() {
    let test = TestStruct {
        name: *b"Test\0\0\0\0\0\0",
        value: *b"abc\0",
    };
    
    assert_eq!(test.name(), "Test");
    assert_eq!(test.value(), None);
} 