use minicbor::{Decoder, decode::Error, Decode, data::Type};

use super::objectcode::cbor_map;

enum CborVariety {
    Integer,
    Float,
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
            Type::Int => CborVariety::Integer,
            Type::F16 | Type::F32 | Type::F64 => CborVariety::Float,
            Type::String | Type::StringIndef => CborVariety::String,
            Type::Array | Type::ArrayIndef => CborVariety::Array,
            Type::Map | Type::MapIndef => CborVariety::Map,
            _ => { 
                return Err(Error::message("bad constant"));
            }
        })
    }
}

macro_rules! force {
    ($name:ident,$mname:ident,$arm:tt,$out:ty,false) => {
        pub fn $name(&self) -> Result<&$out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(format!("unexpected type"))
            }
        }

        pub fn $mname(&mut self) -> Result<&mut $out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(format!("unexpected type"))
            }
        }
    };

    ($name:ident,$mname:ident,$arm:tt,$out:ty,true) => {
        pub fn $name(&self) -> Result<$out,String> {
            match self {
                Value::$arm(v) => Ok(*v),
                _ => Err(format!("unexpected type"))
            }
        }

        pub fn $mname(&mut self) -> Result<&mut $out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(format!("unexpected type"))
            }
        }
    };
}

#[derive(Clone,Debug)] // but don't use Clone except for trivial values: it's expensive!
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    FiniteBoolean(Vec<bool>),
    FiniteNumber(Vec<f64>),
    FiniteString(Vec<String>),
    InfiniteBoolean(bool),
    InfiniteNumber(f64),
    InfiniteString(String),
}

impl Value {
    pub fn is_finite(&self) -> bool {
        match self {
            Value::InfiniteBoolean(_) => false,
            Value::InfiniteNumber(_) => false,
            Value::InfiniteString(_) => false,
            _ => true
        }
    }

    force!(force_boolean,force_boolean_mut,Boolean,bool,true);
    force!(force_number,force_number_mut,Number,f64,true);
    force!(force_string,force_string_mut,String,str,false);
    force!(force_finite_boolean,force_finite_boolean_mut,FiniteBoolean,Vec<bool>,false);
    force!(force_finite_number,force_finite_number_mut,FiniteNumber,Vec<f64>,false);
    force!(force_finite_string,force_finite_string_mut,FiniteString,Vec<String>,false);
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
            CborVariety::Float => Value::Number(d.f64()?),
            CborVariety::Integer => Value::Number(d.u32()? as f64),
            CborVariety::String => Value::String(d.str()?.to_string()),
            CborVariety::Boolean => Value::Boolean(d.bool()?),
            CborVariety::Array => {
                let mut p = d.probe();
                p.array().ok();
                match CborVariety::peek(&mut p)? {
                    CborVariety::Float => Value::FiniteNumber(from_array(d)?),
                    CborVariety::Integer => {
                        let v = from_array::<u32>(d)?;
                        Value::FiniteNumber(v.iter().map(|x| *x as f64).collect())
                    }
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
                            CborVariety::Float => Value::InfiniteNumber(d.f64()?),
                            CborVariety::Integer => {
                                let v = from_array::<u32>(d)?;
                                Value::FiniteNumber(v.iter().map(|x| *x as f64).collect())
                            },
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
