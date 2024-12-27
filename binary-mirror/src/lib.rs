use std::fmt;

#[derive(Debug)]
pub struct BytesSizeError {
    pub(crate) expected: usize,
    pub(crate) actual: usize,
    pub(crate) bytes: String,
}

impl fmt::Display for BytesSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bytes size mismatch: expected {} bytes but got {} bytes, content: \"{}\"",
            self.expected, self.actual, self.bytes
        )
    }
}

impl std::error::Error for BytesSizeError {}

impl BytesSizeError {
    pub fn new(expected: usize, actual: usize, bytes: String) -> Self {
        Self {
            expected,
            actual,
            bytes,
        }
    }
}

pub fn to_hex_repr(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("0x{:02x}", b)).collect::<Vec<_>>().join(", ")
}

pub fn to_bytes_repr(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| {
        match b {
            0x0A => "\\n".to_string(),
            0x0D => "\\r".to_string(),
            0x09 => "\\t".to_string(),
            0x20..=0x7E => (b as char).to_string(),
            _ => format!("\\x{:02x}", b),
        }
    }).collect::<Vec<String>>().join("")
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub offset: usize,
    pub limit: usize,
    pub size: usize,
}

pub trait FromBytes: Sized {
    /// Get the size of the struct in bytes
    const SIZE: usize;

    /// Create a new instance from bytes
    /// Returns Err if the bytes length doesn't match the struct size
    fn from_bytes(bytes: &[u8]) -> Result<&Self, BytesSizeError>;
}

pub trait ToBytes {
    /// Convert the struct to its binary representation
    fn to_bytes(&self) -> &[u8];
    
    /// Convert the struct to an owned Vec<u8>
    fn to_bytes_owned(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

pub trait ToNative {
    type Native;
    
    /// Convert to native type
    fn to_native(&self) -> Self::Native;
}

pub trait FromNative<T> {
    /// Create from native type
    fn from_native(native: &T) -> Self;
}

pub trait NativeStructCode {
    /// Get the native struct code as a string
    fn native_struct_code() -> String;
}