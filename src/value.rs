use std::{collections::HashMap, fmt};

use kdl::KdlValue;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// A newtype wrapper for [`KdlValue`] with Serde support.
///
/// ```rust
/// # use kdl::KdlValue;
/// # use serde_kdl::Value;
/// # use serde_json::json;
/// let value = Value(KdlValue::String("hello world".to_string()));
/// let json = serde_json::to_value(value).unwrap();
/// assert_eq!(json, json!("hello world"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Value(pub KdlValue);

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            KdlValue::Null => serializer.serialize_unit(),
            KdlValue::Boolean(b) => serializer.serialize_bool(b),
            KdlValue::Int(i) => i.serialize(serializer),
            KdlValue::Float(f) => f.serialize(serializer),
            KdlValue::String(ref s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid KDL value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value(KdlValue::Boolean(value)))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value(KdlValue::Int(value)))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Value, E> {
                Ok(Value(KdlValue::Int(i64::from(value))))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value(KdlValue::Float(value)))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value(KdlValue::String(value)))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value(KdlValue::Null))
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value(KdlValue::Null))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

/// Maps `Vec<`[`KdlValue`]`>` to `Vec<`[`Value`]`>`.
pub fn from_kdl_vec(v: Vec<KdlValue>) -> Vec<Value> {
    v.into_iter().map(Value).collect()
}

/// Maps `Vec<`[`Value`]`>` to `Vec<`[`KdlValue`]`>`.
pub fn into_kdl_vec(v: Vec<Value>) -> Vec<KdlValue> {
    v.into_iter().map(|Value(v)| v).collect()
}

/// Maps `HashMap<String, `[`KdlValue`]`>` to `HashMap<String, `[`Value`]`>`.
pub fn from_kdl_map(v: HashMap<String, KdlValue>) -> HashMap<String, Value> {
    v.into_iter().map(|(k, v)| (k, Value(v))).collect()
}

/// Maps `HashMap<String, `[`Value`]`>` to `HashMap<String, `[`KdlValue`]`>`.
pub fn into_kdl_map(v: HashMap<String, Value>) -> HashMap<String, KdlValue> {
    v.into_iter().map(|(k, Value(v))| (k, v)).collect()
}
