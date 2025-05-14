use lexicon_cid::Cid;
use libipld::cbor::encode::write_null;
use libipld::cbor::DagCborCodec;
use libipld::codec::Encode;
use serde::Deserializer;
use serde_cbor::Value as CborValue;
use serde_json::{Value as JsonValue, Value};
use std::collections::BTreeMap;
use thiserror::Error;

/// Ipld
#[derive(Debug, Clone, PartialEq)]
pub enum Ipld {
    /// Represents a Cid.
    Link(Cid),
    /// Represents a list.
    List(Vec<Ipld>),
    /// Represents a map of strings to objects.
    Map(BTreeMap<String, Ipld>),
    /// String
    String(String),
    /// Number
    Integer(i64),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
    /// Represents a Json Value
    Json(JsonValue),
}

#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Ipld {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            let __content = <_serde::__private::de::Content as _serde::Deserialize>::deserialize(
                __deserializer,
            )?;
            let __deserializer =
                _serde::__private::de::ContentRefDeserializer::<__D::Error>::new(&__content);
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <Cid as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::Link,
            ) {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <Vec<Ipld> as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::List,
            ) {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <BTreeMap<String, Ipld> as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::Map,
            ) {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <String as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::String,
            ) {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <i64 as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::Integer,
            ) {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = {
                _serde::__private::Result::map(
                    serde_bytes::deserialize(__deserializer),
                    |__wrap: (Vec<u8>)| Ipld::Bytes(__wrap),
                )
            } {
                return _serde::__private::Ok(__ok);
            }
            if let _serde::__private::Ok(__ok) = _serde::__private::Result::map(
                <JsonValue as _serde::Deserialize>::deserialize(__deserializer),
                Ipld::Json,
            ) {
                return _serde::__private::Ok(__ok);
            }
            _serde::__private::Err(_serde::de::Error::custom(
                "data did not match any variant of untagged enum Ipld",
            ))
        }
    }
};

impl serde::Serialize for Ipld {
    fn serialize<S>(&self, serializer: S) -> serde::__private::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Ipld::Link(ref field0) => serde::Serialize::serialize(field0, serializer),
            Ipld::List(ref field0) => serde::Serialize::serialize(field0, serializer),
            Ipld::Map(ref field0) => serde::Serialize::serialize(field0, serializer),
            Ipld::String(ref field0) => serde::Serialize::serialize(field0, serializer),
            Ipld::Integer(ref field0) => serializer.serialize_i64(field0.clone()),
            Ipld::Bytes(ref field0) => serde::Serialize::serialize(
                {
                    #[doc(hidden)]
                    struct SerializeWith<'a> {
                        values: (&'a Vec<u8>,),
                        phantom: serde::__private::PhantomData<Ipld>,
                    }
                    #[automatically_derived]
                    impl<'a> serde::Serialize for SerializeWith<'a> {
                        fn serialize<S>(&self, s: S) -> serde::__private::Result<S::Ok, S::Error>
                        where
                            S: serde::Serializer,
                        {
                            serde_bytes::serialize(self.values.0, s)
                        }
                    }
                    &SerializeWith {
                        values: (field0,),
                        phantom: serde::__private::PhantomData::<Ipld>,
                    }
                },
                serializer,
            ),
            Ipld::Json(ref field0) => match field0 {
                Value::Null => serde::Serialize::serialize(field0, serializer),
                Value::Bool(x) => serde::Serialize::serialize(x, serializer),
                Value::Number(x) => {
                    let value = x.as_i64().unwrap();
                    serializer.serialize_i64(value)
                }
                Value::String(x) => serde::Serialize::serialize(x, serializer),
                Value::Array(x) => serde::Serialize::serialize(x, serializer),
                Value::Object(x) => serde::Serialize::serialize(x, serializer),
            },
        }
    }
}

impl Encode<DagCborCodec> for Ipld {
    fn encode<W: Write>(&self, c: DagCborCodec, w: &mut W) -> Result<()> {
        match self {
            Self::Json(JsonValue::Null) => write_null(w),
            Self::Json(JsonValue::Bool(b)) => b.encode(c, w),
            Self::Json(JsonValue::Number(n)) => {
                if n.is_f64() {
                    n.as_f64().unwrap().encode(c, w)
                } else if n.is_u64() {
                    n.as_u64().unwrap().encode(c, w)
                } else {
                    n.as_i64().unwrap().encode(c, w)
                }
            }
            Self::Json(JsonValue::String(s)) => s.encode(c, w),
            Self::Json(JsonValue::Object(o)) => serde_json::to_vec(o)?.encode(c, w),
            Self::Json(JsonValue::Array(a)) => serde_json::to_vec(a)?.as_slice().encode(c, w),
            Self::Bytes(b) => b.as_slice().encode(c, w),
            Self::List(l) => l.encode(c, w),
            Self::Map(m) => m.encode(c, w),
            Self::Link(cid) => cid.encode(c, w),
            Self::String(s) => s.encode(c, w),
            Self::Integer(i) => i.encode(c, w),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ObjAndBytes {
    pub obj: CborValue,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CidAndRev {
    pub cid: Cid,
    pub rev: String,
}

#[derive(Error, Debug)]
pub enum RepoRootError {
    #[error("Repo root not found")]
    RepoRootNotFoundError,
}

pub mod memory_blockstore;
pub mod readable_blockstore;
pub mod sync_storage;
pub mod types;
