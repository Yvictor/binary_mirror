use std::fmt;

#[derive(Debug)]
pub struct BytesSizeError {
    pub(crate) expected: usize,
    pub(crate) actual: usize,
    pub(crate) bytes: String,
}

impl fmt::Display for BytesSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bytes size mismatch: expected {} bytes but got {} bytes, content: \"{}\"", 
            self.expected, 
            self.actual,
            self.bytes
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