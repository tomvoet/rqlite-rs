use crate::{row::Row, IntoTypedError};

pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self, IntoTypedError>;
}
