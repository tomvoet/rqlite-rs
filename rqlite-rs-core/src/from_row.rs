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
