use crate::{row::Row, IntoTypedError};

pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self, IntoTypedError>;
}

macro_rules! impl_from_row_for_tuple {
    ($( ($idx:tt) - $T:ident );+;) => {
        impl<$($T,)+> FromRow for ($($T,)+)
        where
            $($T: serde::de::DeserializeOwned,)+
        {
            #[inline]
            fn from_row(row: Row) -> Result<Self, IntoTypedError> {
                let columns = row.columns().len();
                // weird, hacky way to calculate the expected number of columns
                // can't overflow, because the number of columns is always 0 < n < 17
                #[allow(arithmetic_overflow)]
                let expected: i32 = 0 $(+ $idx - ($idx - 1))+;

                let expected = expected as usize;

                if columns != expected {
                    return Err(IntoTypedError::ColumnCountMismatch(expected, columns));
                }

                Ok(($(
                    row.get_by_index::<$T>($idx)?,
                )+))
            }
        }
    };
}

impl_from_row_for_tuple!(
    (0) - T0;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
    (11) - T11;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
    (11) - T11;
    (12) - T12;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
    (11) - T11;
    (12) - T12;
    (13) - T13;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
    (11) - T11;
    (12) - T12;
    (13) - T13;
    (14) - T14;
);

impl_from_row_for_tuple!(
    (0) - T0;
    (1) - T1;
    (2) - T2;
    (3) - T3;
    (4) - T4;
    (5) - T5;
    (6) - T6;
    (7) - T7;
    (8) - T8;
    (9) - T9;
    (10) - T10;
    (11) - T11;
    (12) - T12;
    (13) - T13;
    (14) - T14;
    (15) - T15;
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::Column;
    use std::{collections::HashMap, sync::Arc};

    #[test]
    fn unit_from_row_tuple() {
        let mut column_names: HashMap<String, usize> = HashMap::new();
        let columns = vec![
            Column::new("id".to_string(), 0, "integer".to_string()),
            Column::new("name".to_string(), 1, "text".to_string()),
        ];

        columns.iter().enumerate().for_each(|(i, c)| {
            column_names.insert(c.name().to_string(), i);
        });

        let columns = Arc::new(columns);
        let column_names = Arc::new(column_names);

        let row = Row::new(
            &columns,
            &column_names,
            vec![serde_json::json!(1), serde_json::json!("test")].into_boxed_slice(),
        );

        let (id, name) = <(i32, String)>::from_row(row).unwrap();

        assert_eq!(id, 1);
        assert_eq!(name, "test");

        let row = Row::new(
            &columns,
            &column_names,
            vec![serde_json::json!(1), serde_json::json!("test")].into_boxed_slice(),
        );

        let foo = <(i32, String, i32)>::from_row(row).unwrap_err();
        assert!(matches!(foo, IntoTypedError::ColumnCountMismatch(3, 2)));
    }
}
