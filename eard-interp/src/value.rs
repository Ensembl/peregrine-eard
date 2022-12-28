use minicbor::{Decoder, decode::Error, Decode, data::Type};
use crate::objectcode::cbor_map;

enum CborVariety {
    Index,
    Number,
    String,
    Boolean,
    Array,
    Map
}

impl CborVariety {
    fn peek(d: &mut Decoder) -> Result<CborVariety, Error> {
        Ok(match d.probe().datatype()? {
            Type::Bool => CborVariety::Boolean,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 |
            Type::I8 | Type::I16 | Type::I32 | Type::I64 |
            Type::Int => CborVariety::Index,
            Type::F16 |
            Type::F32 |
            Type::F64 => CborVariety::Number,
            Type::String | Type::StringIndef => CborVariety::String,
            Type::Array | Type::ArrayIndef => CborVariety::Array,
            Type::Map | Type::MapIndef => CborVariety::Map,
            _ => { 
                return Err(Error::message("bad constant"));
            }
        })
    }
}

#[derive(Clone,Debug)] // but don't use Clone except for trivial values: it's expensive!
pub enum Value {
    Boolean(bool),
    Number(f64),
    Index(usize),
    String(String),
    FiniteBoolean(Vec<bool>),
    FiniteNumber(Vec<f64>),
    FiniteIndex(Vec<usize>),
    FiniteString(Vec<String>),
    InfiniteBoolean(bool),
    InfiniteNumber(f64),
    InfiniteIndex(usize),
    InfiniteString(String),
}

impl Default for Value {
    fn default() -> Self { Value::Boolean(false) }
}

fn from_array<'a,T: Decode<'a,()>>(d: &mut Decoder<'a>) -> Result<Vec<T>,Error> {
    let mut out = vec![];
    for item in d.array_iter()? {
        out.push(item?);
    }
    Ok(out)
}

impl<'b> Decode<'b,()> for Value {
    fn decode(d: &mut Decoder<'b>, ctx: &mut ()) -> Result<Self, Error> {
        Ok(match CborVariety::peek(d)? {
            CborVariety::Index => Value::Index(d.u32()? as usize),
            CborVariety::Number => Value::Number(d.f64()?),
            CborVariety::String => Value::String(d.str()?.to_string()),
            CborVariety::Boolean => Value::Boolean(d.bool()?),
            CborVariety::Array => {
                let mut p = d.probe();
                p.array().ok();
                match CborVariety::peek(&mut p)? {
                    CborVariety::Index => Value::FiniteIndex(from_array(d)?),
                    CborVariety::Number => Value::FiniteNumber(from_array(d)?),
                    CborVariety::String => Value::FiniteString(from_array(d)?),
                    CborVariety::Boolean => Value::FiniteBoolean(from_array(d)?),
                    _ => { return Err(Error::message("bad constant")); }
                }
            },
            CborVariety::Map => {
                let mut out = None;
                cbor_map(d,&mut out,|key,out,d| {
                    if key == "" {
                        let mut p = d.probe();
                        p.array().ok();
                        *out = Some(match CborVariety::peek(&mut p)? {
                            CborVariety::Index => Value::InfiniteIndex(d.u32()? as usize),
                            CborVariety::Number => Value::InfiniteNumber(d.f64()?),
                            CborVariety::String => Value::InfiniteString(d.str()?.to_string()),
                            CborVariety::Boolean => Value::InfiniteBoolean(d.bool()?),
                            _ => { return Err(Error::message("bad constant")); }
                        });
                    } else {
                        d.skip()?;
                    }
                    Ok(())
                })?;
                out.unwrap()
            }
        })
    }
}
