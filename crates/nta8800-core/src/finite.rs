//! Safety net for results: the path of the first non-finite number
//! (NaN or ±∞) in a serializable value.
//!
//! serde_json writes a non-finite `f64` as `null`, so a broken result would
//! otherwise reach the report as a blank. The assessments call
//! [`first_non_finite`] before they return and refuse the result instead.

use serde::ser::{self, Serialize};
use std::fmt;

/// Path (`a.b[2].c`) of the first non-finite `f32`/`f64` in `value`.
pub fn first_non_finite<T: Serialize + ?Sized>(value: &T) -> Option<String> {
    let mut finder = Finder::default();
    match value.serialize(&mut finder) {
        Err(Found) => Some(finder.path.join("")),
        Ok(()) => None,
    }
}

/// The walk stops at the first non-finite number by returning this error.
#[derive(Debug)]
struct Found;

impl fmt::Display for Found {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("non-finite number")
    }
}

impl std::error::Error for Found {}

impl ser::Error for Found {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Found
    }
}

#[derive(Default)]
struct Finder {
    path: Vec<String>,
}

impl Finder {
    fn number(&mut self, value: f64) -> Result<(), Found> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(Found)
        }
    }

    fn nested<T: Serialize + ?Sized>(&mut self, segment: String, value: &T) -> Result<(), Found> {
        self.path.push(segment);
        value.serialize(&mut *self)?;
        self.path.pop();
        Ok(())
    }
}

fn field(key: &str, first: bool) -> String {
    if first {
        key.to_string()
    } else {
        format!(".{key}")
    }
}

/// Sequence, map and struct walker.
struct Walk<'a> {
    finder: &'a mut Finder,
    index: usize,
    key: String,
}

impl<'a> ser::Serializer for &'a mut Finder {
    type Ok = ();
    type Error = Found;
    type SerializeSeq = Walk<'a>;
    type SerializeTuple = Walk<'a>;
    type SerializeTupleStruct = Walk<'a>;
    type SerializeTupleVariant = Walk<'a>;
    type SerializeMap = Walk<'a>;
    type SerializeStruct = Walk<'a>;
    type SerializeStructVariant = Walk<'a>;

    fn serialize_bool(self, _: bool) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_f32(self, value: f32) -> Result<(), Found> {
        self.number(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> Result<(), Found> {
        self.number(value)
    }
    fn serialize_char(self, _: char) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Found> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Found> {
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Found> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Found> {
        let segment = field(variant, self.path.is_empty());
        self.nested(segment, value)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_tuple(self, _: usize) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Walk<'a>, Found> {
        Ok(Walk {
            finder: self,
            index: 0,
            key: String::new(),
        })
    }
}

impl Walk<'_> {
    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        let segment = format!("[{}]", self.index);
        self.index += 1;
        self.finder.nested(segment, value)
    }

    fn member<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) -> Result<(), Found> {
        let segment = field(key, self.finder.path.is_empty());
        self.finder.nested(segment, value)
    }
}

impl ser::SerializeSeq for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        self.element(value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeTuple for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        self.element(value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        self.element(value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        self.element(value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeMap for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Found> {
        // Map keys are strings or numbers in the kernel output.
        self.key = serde_json::to_value(key)
            .map(|value| match value {
                serde_json::Value::String(text) => text,
                other => other.to_string(),
            })
            .unwrap_or_default();
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Found> {
        let key = std::mem::take(&mut self.key);
        self.member(&key, value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeStruct for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Found> {
        self.member(key, value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for Walk<'_> {
    type Ok = ();
    type Error = Found;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Found> {
        self.member(key, value)
    }
    fn end(self) -> Result<(), Found> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::first_non_finite;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Inner {
        mean_u: f64,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Outer {
        status: &'static str,
        values: Vec<f64>,
        items: Vec<Inner>,
        named: BTreeMap<String, Option<f64>>,
    }

    #[test]
    fn finds_the_first_non_finite_number_with_its_path() {
        let mut value = Outer {
            status: "calculated",
            values: vec![1.0, 2.0],
            items: vec![Inner { mean_u: 0.2 }, Inner { mean_u: 0.3 }],
            named: BTreeMap::from([("a".to_string(), Some(1.0)), ("b".to_string(), None)]),
        };
        assert_eq!(first_non_finite(&value), None);
        value.items[1].mean_u = f64::INFINITY;
        assert_eq!(first_non_finite(&value).as_deref(), Some("items[1].meanU"));
        value.values[0] = f64::NAN;
        assert_eq!(first_non_finite(&value).as_deref(), Some("values[0]"));
        value.values[0] = 1.0;
        value.items[1].mean_u = 0.3;
        value.named.insert("c".into(), Some(f64::NEG_INFINITY));
        assert_eq!(first_non_finite(&value).as_deref(), Some("named.c"));
    }
}
