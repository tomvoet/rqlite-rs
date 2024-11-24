use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(untagged)]
pub enum RqliteArgumentValue<'a> {
    String(Cow<'a, str>),
    I64(i64),
    F64(f64),
    F32(f32),
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum RqliteArgument<'a> {
    Some(RqliteArgumentValue<'a>),
    None,
}

impl<'a> Serialize for RqliteArgument<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RqliteArgument::Some(arg) => arg.serialize(serializer),
            RqliteArgument::None => serializer.serialize_none(),
        }
    }
}

pub trait RqliteArgumentRaw<'a> {
    fn encode(&self) -> RqliteArgument<'a>;
}

impl<'a> RqliteArgumentRaw<'a> for RqliteArgumentValue<'a> {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(match self {
            // For String, just clone the ptr
            RqliteArgumentValue::String(s) => RqliteArgumentValue::String(s.clone()),
            RqliteArgumentValue::I64(val) => RqliteArgumentValue::I64(*val),
            RqliteArgumentValue::F64(val) => RqliteArgumentValue::F64(*val),
            RqliteArgumentValue::F32(val) => RqliteArgumentValue::F32(*val),
            RqliteArgumentValue::Bool(val) => RqliteArgumentValue::Bool(*val),
        })
    }
}

impl<'a> RqliteArgumentRaw<'a> for &'a RqliteArgumentValue<'a> {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(match self {
            RqliteArgumentValue::String(s) => RqliteArgumentValue::String(Cow::Borrowed(s)),
            RqliteArgumentValue::I64(val) => RqliteArgumentValue::I64(*val),
            RqliteArgumentValue::F64(val) => RqliteArgumentValue::F64(*val),
            RqliteArgumentValue::F32(val) => RqliteArgumentValue::F32(*val),
            RqliteArgumentValue::Bool(val) => RqliteArgumentValue::Bool(*val),
        })
    }
}

impl<'a> RqliteArgumentRaw<'a> for i32 {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::I64(*self as i64))
    }
}

impl<'a> RqliteArgumentRaw<'a> for i64 {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::I64(*self))
    }
}

impl<'a> RqliteArgumentRaw<'a> for usize {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::I64(*self as i64))
    }
}

impl<'a> RqliteArgumentRaw<'a> for f64 {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::F64(*self))
    }
}

impl<'a> RqliteArgumentRaw<'a> for f32 {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::F32(*self))
    }
}

impl<'a> RqliteArgumentRaw<'a> for bool {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::Bool(*self))
    }
}

impl<'a> RqliteArgumentRaw<'a> for &'a str {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::String(Cow::Borrowed(*self)))
    }
}

impl<'a> RqliteArgumentRaw<'a> for String {
    fn encode(&self) -> RqliteArgument<'a> {
        RqliteArgument::Some(RqliteArgumentValue::String(Cow::Owned(self.clone())))
    }
}

impl<'a, T> RqliteArgumentRaw<'a> for Option<T>
where
    T: RqliteArgumentRaw<'a>,
{
    fn encode(&self) -> RqliteArgument<'a> {
        match self {
            Some(value) => value.encode(),
            None => RqliteArgument::None,
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
        let arg = arg!(1);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::I64(1)));

        let arg = arg!(1i32);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::I64(1)));

        let arg = arg!(1usize);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::I64(1)));

        let arg = arg!(1i64);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::I64(1)));

        let arg = arg!(1.0);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::F64(1.0)));

        let arg = arg!(1.0f32);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::F32(1.0)));

        let arg = arg!(true);
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::Bool(true)));

        let arg = arg!("hello");
        assert_eq!(
            arg,
            RqliteArgument::Some(RqliteArgumentValue::String(Cow::Borrowed("hello")))
        );

        let arg = arg!("hello".to_string());
        assert_eq!(
            arg,
            RqliteArgument::Some(RqliteArgumentValue::String(Cow::Owned("hello".to_string())))
        );
    }

    #[test]
    fn unit_rqlite_argument_option() {
        let arg = arg!(Some(RqliteArgumentValue::I64(1)));
        assert_eq!(arg, RqliteArgument::Some(RqliteArgumentValue::I64(1)));

        let arg = arg!(None::<RqliteArgumentValue>);
        assert_eq!(arg, RqliteArgument::None);
    }

    #[test]
    fn unit_rqlite_serialize_option() {
        let arg = RqliteArgument::Some(RqliteArgumentValue::I64(1));
        let serialized = serde_json::to_string(&arg).unwrap();
        assert_eq!(serialized, "1");

        let arg = RqliteArgument::None;
        let serialized = serde_json::to_string(&arg).unwrap();
        assert_eq!(serialized, "null");
    }

    #[test]
    fn unit_rqlite_argument_raw_owned() {
        let arg = RqliteArgumentValue::String(Cow::Owned("hello".to_string()));
        let encoded = arg.encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::String(Cow::Owned("hello".to_string())))
        );

        let arg = RqliteArgumentValue::I64(123);
        let encoded = arg.encode();
        assert_eq!(encoded, RqliteArgument::Some(RqliteArgumentValue::I64(123)));

        let arg = RqliteArgumentValue::F64(3.2);
        let encoded = arg.encode();
        assert_eq!(encoded, RqliteArgument::Some(RqliteArgumentValue::F64(3.2)));

        let arg = RqliteArgumentValue::F32(2.71);
        let encoded = arg.encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::F32(2.71))
        );

        let arg = RqliteArgumentValue::Bool(true);
        let encoded = arg.encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::Bool(true))
        );
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn unit_rqlite_argument_raw_borrowed() {
        let arg = RqliteArgumentValue::String(Cow::Owned("hello".to_string()));
        let encoded = (&arg).encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::String(Cow::Borrowed("hello")))
        );

        let arg = RqliteArgumentValue::I64(123);
        let encoded = (&arg).encode();
        assert_eq!(encoded, RqliteArgument::Some(RqliteArgumentValue::I64(123)));

        let arg = RqliteArgumentValue::F64(3.2);
        let encoded = (&arg).encode();
        assert_eq!(encoded, RqliteArgument::Some(RqliteArgumentValue::F64(3.2)));

        let arg = RqliteArgumentValue::F32(2.71);
        let encoded = (&arg).encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::F32(2.71))
        );

        let arg = RqliteArgumentValue::Bool(true);

        let encoded = (&arg).encode();
        assert_eq!(
            encoded,
            RqliteArgument::Some(RqliteArgumentValue::Bool(true))
        );
    }
}
