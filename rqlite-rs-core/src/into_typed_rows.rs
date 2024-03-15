use crate::{FromRow, Row};

pub trait IntoTypedRows {
    fn into_typed<T: FromRow>(self) -> anyhow::Result<Vec<T>>;
}

impl IntoTypedRows for Vec<Row> {
    fn into_typed<T: FromRow>(self) -> anyhow::Result<Vec<T>> {
        self.into_iter().map(T::from_row).collect()
    }
}
