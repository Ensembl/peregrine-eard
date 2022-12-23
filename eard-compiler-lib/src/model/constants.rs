use std::{convert::Infallible, fmt};
use json::{JsonValue, object::Object};
use minicbor::{Encoder, encode::Error};
use ordered_float::OrderedFloat;
use crate::test::testutil::sepfmt;

use super::checkstypes::AtomicTypeSpec;

#[derive(PartialEq,PartialOrd,Eq,Ord,Clone)]
pub enum Constant {
    Number(OrderedFloat<f64>),
    String(String),
    Boolean(bool)
}

impl Constant {
    pub(crate) fn to_atomic_type(&self) -> AtomicTypeSpec {
        match self {
            Constant::Number(_) => AtomicTypeSpec::Number,
            Constant::String(_) => AtomicTypeSpec::String,
            Constant::Boolean(_) => AtomicTypeSpec::Boolean
        }
    }
}

impl Constant {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        match self {
            Constant::Number(n) => { 
                if n.0.fract() == 0. {
                    encoder.i64(n.0 as i64)?;
                } else {
                    encoder.f64(n.0)?; 
                }
            },
            Constant::String(s) => { encoder.str(s)?; },
            Constant::Boolean(b) => { encoder.bool(*b)?; }
        }
        Ok(())
    }

    fn encode_json(&self) -> JsonValue {
        match self {
            Constant::Number(n) => JsonValue::Number(n.0.into()),
            Constant::String(s) => JsonValue::String(s.into()),
            Constant::Boolean(b) => JsonValue::Boolean(*b)
        }
    }
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",match self {
            Constant::Number(n) => n.to_string(),
            Constant::String(s) => format!("{:?}",s),
            Constant::Boolean(b) => {
                (if *b { "true" } else { "false" }).to_string()
            }
        })
    }
}

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone)]
pub enum FullConstant {
    Atomic(Constant),
    Finite(Vec<Constant>),
    Infinite(Constant)
}

impl FullConstant {
    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        match self {
            FullConstant::Atomic(c) => {
                c.encode(encoder)?;
            },
            FullConstant::Finite(seq) => {
                encoder.array(seq.len() as u64)?;
                for c in seq {
                    c.encode(encoder)?;
                }
            },
            FullConstant::Infinite(c) => {
                encoder.begin_map()?.str("")?;
                c.encode(encoder)?;
                encoder.end()?;
            }
        }
        Ok(())
    }

    pub(crate) fn encode_json(&self) -> JsonValue {
        match self {
            FullConstant::Atomic(c) => c.encode_json(),
            FullConstant::Finite(s) => {
                JsonValue::Array(s.iter().map(|c| c.encode_json()).collect())
            },
            FullConstant::Infinite(c) => {
                let mut obj = Object::new();
                obj.insert("",c.encode_json());
                JsonValue::Object(obj)
            },
        }
    }
}

impl fmt::Debug for FullConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(a) => write!(f,"{:?}",a),
            Self::Finite(s) => write!(f,"[{}]",sepfmt(&mut s.iter(),",","")),
            Self::Infinite(a) => write!(f,"[{:?},...]",a),
        }
    }
}
