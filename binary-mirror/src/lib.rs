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

// pub mod strp {

//     pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let s = String::from_utf8_lossy(value).trim().to_string();
//         serializer.serialize_str(&s)
//     }

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 10], D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let s: &[u8] = serde::Deserialize::deserialize(deserializer)?;
//         Ok(s.try_into().unwrap())
//     }
// }
