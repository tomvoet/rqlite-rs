use crate::row::Row;

pub trait FromRow: Sized {
    fn from_row(row: Row) -> anyhow::Result<Self>;
}
