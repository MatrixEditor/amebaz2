pub mod pt;

#[macro_export]
macro_rules! expect_length {
    ($value:expr, $expected:literal) => {
        if $value.len() != $expected {
            return Err(crate::error::Error::InvalidState(format!(
                "Expected {} characters in hex string, got {}",
                $expected,
                $value.len()
            )));
        }
    };
}
