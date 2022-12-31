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

    fn to_index(&self) -> Option<u32> {
        match self {
            Constant::Number(n) => {
                if n.0.fract() == 0. && n.0 > 0. && n.0 < 2_000_000_000. {
                    return Some(n.0 as u32);
                }
            },
            _ => {}
        }
        None
    }

    fn vec_to_index(values: &[Self]) -> Option<Vec<u32>> {
        values.iter().map(|v| v.to_index()).collect::<Option<Vec<_>>>()
    }

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
    pub(crate) fn is_empty_list(&self) -> bool {
        match self {
            FullConstant::Finite(f) => f.len() == 0,
            _ => false
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

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum OperationConstant {
    Constant(FullConstant),
    EmptyNumberSeq,
    EmptyBooleanSeq,
    EmptyStringSeq,
    EmptyHandleSeq(String)
}

impl OperationConstant {
    pub(crate) fn to_full_constant(&self) -> FullConstant {
        match self {
            Self::Constant(c) => c.clone(),
            _ => FullConstant::Finite(vec![])
        }
    }

    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        match self {
            OperationConstant::Constant(FullConstant::Atomic(c)) => {
                if let Some(value) = c.to_index() {
                    encoder.u32(value as u32)?;
                } else {
                    c.encode(encoder)?;
                }
            },
            OperationConstant::Constant(FullConstant::Finite(seq)) => {
                encoder.array(seq.len() as u64)?;
                if let Some(value) = Constant::vec_to_index(seq) {
                    for v in value {
                        encoder.u32(v)?;
                    }
                } else {
                    for c in seq {
                        c.encode(encoder)?;
                    }
                }
            },
            OperationConstant::Constant(FullConstant::Infinite(c)) => {
                encoder.begin_map()?.str("")?;
                if let Some(value) = c.to_index() {
                    encoder.u32(value as u32)?;
                } else {
                    c.encode(encoder)?;
                }
                encoder.end()?;
            },
            OperationConstant::EmptyBooleanSeq => {
                encoder.begin_map()?.str("e")?;
                encoder.str("b")?;
                encoder.end()?;
            },
            OperationConstant::EmptyNumberSeq => {
                encoder.begin_map()?.str("e")?;
                encoder.str("n")?;
                encoder.end()?;
            },
            OperationConstant::EmptyStringSeq => {
                encoder.begin_map()?.str("e")?;
                encoder.str("s")?;
                encoder.end()?;
            },
            OperationConstant::EmptyHandleSeq(h) => {
                encoder.begin_map()?.str("e")?;
                encoder.str(&format!("h{}",h))?;
                encoder.end()?;
            }
        }
        Ok(())
    }

    pub(crate) fn encode_json(&self) -> JsonValue {
        match self {
            OperationConstant::Constant(FullConstant::Atomic(c)) => c.encode_json(),
            OperationConstant::Constant(FullConstant::Finite(s)) => {
                JsonValue::Array(s.iter().map(|c| c.encode_json()).collect())
            },
            OperationConstant::Constant(FullConstant::Infinite(c)) => {
                let mut obj = Object::new();
                obj.insert("",c.encode_json());
                JsonValue::Object(obj)
            },
            OperationConstant::EmptyBooleanSeq => {
                let mut obj = Object::new();
                obj.insert("e","b".into());
                JsonValue::Object(obj)
            },
            OperationConstant::EmptyNumberSeq => {
                let mut obj = Object::new();
                obj.insert("e","n".into());
                JsonValue::Object(obj)
            },
            OperationConstant::EmptyStringSeq => {
                let mut obj = Object::new();
                obj.insert("e","s".into());
                JsonValue::Object(obj)
            },
            OperationConstant::EmptyHandleSeq(h) => {
                let mut obj = Object::new();
                obj.insert("e",format!("h{}",h).into());
                JsonValue::Object(obj)
            },
        }
    }
}

impl fmt::Debug for OperationConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(c) => write!(f,"{:?}",c),
            Self::EmptyNumberSeq => write!(f, "[]n"),
            Self::EmptyBooleanSeq => write!(f, "[]b"),
            Self::EmptyStringSeq => write!(f, "[]s"),
            Self::EmptyHandleSeq(h) => write!(f,"[]h({})",h)
        }
    }
}
