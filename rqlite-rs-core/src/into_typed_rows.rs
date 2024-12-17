use crate::{FromRow, IntoTypedError, Row};

pub trait IntoTypedRows {
    /// Convert a `Vec<Row>` into a `Vec<T>`, where `T` implements `FromRow`
    ///
    /// # Errors
    /// If any of the values are not found or cannot be converted, returns an `IntoTypedError`
    fn into_typed<T: FromRow>(self) -> Result<Vec<T>, IntoTypedError>;
}

impl IntoTypedRows for Vec<Row> {
    fn into_typed<T: FromRow>(self) -> Result<Vec<T>, IntoTypedError> {
        self.into_iter().map(T::from_row).collect()
    }
}
