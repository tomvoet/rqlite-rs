/// Decodes a base64 encoded string into a blob-like type.
///
/// # Errors
///
/// This function will return an error if the input string is not valid base64.
#[cfg(feature = "blob")]
pub fn decode_blob<T>(blob: &str) -> Result<T, base64::DecodeError>
where
    T: From<Vec<u8>>,
{
    use base64::{engine::general_purpose, Engine};

    general_purpose::STANDARD.decode(blob).map(T::from)
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "blob")]
    fn unit_decode_blob() {
        use super::*;

        let blob = "SGVsbG8gV29ybGQ=";
        let decoded = decode_blob::<Vec<u8>>(blob).unwrap();
        assert_eq!(decoded, b"Hello World");
    }
}
