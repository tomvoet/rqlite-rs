use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(untagged)]
pub enum RqliteArgument {
    String(String),
    I64(i64),
    F64(f64),
    F32(f32),
    Bool(bool),
    Null,
}

impl RqliteArgument {}

pub trait RqliteArgumentRaw {
    fn encode(&self) -> RqliteArgument;
}

impl RqliteArgumentRaw for i32 {
    fn encode(&self) -> RqliteArgument {
        #[allow(clippy::cast_lossless)]
        RqliteArgument::I64(*self as i64)
    }
}

impl RqliteArgumentRaw for i64 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::I64(self.to_owned())
    }
}

impl RqliteArgumentRaw for usize {
    fn encode(&self) -> RqliteArgument {
        #[allow(clippy::cast_possible_wrap)]
        RqliteArgument::I64(*self as i64)
    }
}

impl RqliteArgumentRaw for f64 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::F64(self.to_owned())
    }
}

impl RqliteArgumentRaw for f32 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::F32(self.to_owned())
    }
}

impl RqliteArgumentRaw for bool {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::Bool(self.to_owned())
    }
}

impl RqliteArgumentRaw for &str {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::String((*self).to_string())
    }
}

impl RqliteArgumentRaw for String {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::String(self.to_string())
    }
}

impl<T> RqliteArgumentRaw for Option<T>
where
    T: RqliteArgumentRaw,
{
    fn encode(&self) -> RqliteArgument {
        match self {
            Some(v) => v.encode(),
            None => RqliteArgument::Null,
        }
    }
}

#[macro_export]
macro_rules! arg {
    ($e:expr) => {
        $crate::query::arguments::RqliteArgumentRaw::encode(&$e)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_rqlite_argument() {
        let arg = arg!(1i32);
        assert_eq!(arg, RqliteArgument::I64(1));

        let arg = arg!(1i64);
        assert_eq!(arg, RqliteArgument::I64(1));

        let arg = arg!(1.0);
        assert_eq!(arg, RqliteArgument::F64(1.0));

        let arg = arg!(1.0f32);
        assert_eq!(arg, RqliteArgument::F32(1.0));

        let arg = arg!(true);
        assert_eq!(arg, RqliteArgument::Bool(true));

        let arg = arg!("hello");
        assert_eq!(arg, RqliteArgument::String("hello".to_string()));

        let arg = arg!("hello".to_string());
        assert_eq!(arg, RqliteArgument::String("hello".to_string()));

        let arg = arg!(Some(1i64));
        assert_eq!(arg, RqliteArgument::I64(1));

        let arg = arg!(None::<i64>);
        assert_eq!(arg, RqliteArgument::Null);
    }
}
