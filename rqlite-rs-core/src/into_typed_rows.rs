use crate::{FromRow, IntoTypedError, Row};

pub trait IntoTypedRows {
    fn into_typed<T: FromRow>(self) -> Result<Vec<T>, IntoTypedError>;
}

impl IntoTypedRows for Vec<Row> {
    fn into_typed<T: FromRow>(self) -> Result<Vec<T>, IntoTypedError> {
        self.into_iter().map(T::from_row).collect()
    }
}
