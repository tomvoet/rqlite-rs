#![warn(clippy::pedantic, clippy::all)]
#![allow(clippy::module_name_repetitions)]

mod column;
pub mod decode;
mod error;
mod from_row;
mod into_typed_rows;
mod row;
pub use column::Column;
pub use error::IntoTypedError;
pub use from_row::FromRow;
pub use into_typed_rows::IntoTypedRows;
pub use row::Row;

mod prelude {
    #![allow(unused_imports)]
    pub use crate::column::Column;
    pub use crate::from_row::FromRow;
    pub use crate::into_typed_rows::IntoTypedRows;
    pub use crate::row::Row;
    pub use crate::IntoTypedError;
}
