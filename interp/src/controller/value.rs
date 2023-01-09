use minicbor::{Decoder, decode::Error, Decode, data::Type};

use super::objectcode::{cbor_map, cbor_array};

enum CborVariety {
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
            Type::Int | Type::F16 | Type::F32 | Type::F64 => CborVariety::Number,
            Type::String | Type::StringIndef => CborVariety::String,
            Type::Array | Type::ArrayIndef => CborVariety::Array,
            Type::Map | Type::MapIndef => CborVariety::Map,
            x => { 
                return Err(Error::message(&format!("bad constant {}",x)));
            }
        })
    }
}

fn unexpected_type(got: &Value) -> String {
    format!("unexpected type got={:?}",got)
}

macro_rules! force {
    ($name:ident,$mname:ident,$arm:tt,$out:ty,false) => {
        pub fn $name(&self) -> Result<&$out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(unexpected_type(self))
            }
        }

        pub fn $mname(&mut self) -> Result<&mut $out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(unexpected_type(self))
            }
        }
    };

    ($name:ident,$mname:ident,$arm:tt,$out:ty,true) => {
        pub fn $name(&self) -> Result<$out,String> {
            match self {
                Value::$arm(v) => Ok(*v),
                _ => Err(unexpected_type(self))
            }
        }

        pub fn $mname(&mut self) -> Result<&mut $out,String> {
            match self {
                Value::$arm(v) => Ok(v),
                _ => Err(unexpected_type(self))
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

fn merge2_s<T,F>(a: &mut Vec<T>,b: &T, mut cb: F) where F: FnMut(&mut T,&T) -> bool {
    for v in a.iter_mut() {
        cb(v,b);
    }
}

fn merge2_ss<T,F>(a: &mut Vec<T>,b: &Vec<T>, mut cb: F) where F: FnMut(&mut T,&T) -> bool {
    let mut bv = b.iter();
    let mut bvv = bv.next();
    for v in a.iter_mut() {
        if let Some(value) = bvv {
            if cb(v,value) {
                bvv = bv.next();
            }
        }
    }
}

impl Value {
    pub fn merge2<F,G,H>(&mut self, other: &Value, mut cb_b: F, mut cb_n: G, mut cb_s: H) -> Result<(),String>
            where F: FnMut(&mut bool,&bool) -> bool, 
                  G: FnMut(&mut f64,&f64) -> bool,
                  H: FnMut(&mut String,&String) -> bool {
        match (self,other) {
            (Value::Boolean(a), Value::Boolean(b)) => { cb_b(a,b); },
            (Value::Boolean(a), Value::InfiniteBoolean(b)) => {cb_b(a,b); },
            (Value::Number(a), Value::Number(b)) => { cb_n(a,b); },
            (Value::Number(a), Value::InfiniteNumber(b)) => { cb_n(a,b); },
            (Value::String(a), Value::String(b)) => { cb_s(a,b); },
            (Value::String(a), Value::InfiniteString(b)) => { cb_s(a,b); },
            (Value::FiniteBoolean(a), Value::FiniteBoolean(b)) => merge2_ss(a,b,cb_b),
            (Value::FiniteBoolean(a), Value::InfiniteBoolean(b)) => merge2_s(a,b,cb_b),
            (Value::FiniteNumber(a), Value::FiniteNumber(b)) => merge2_ss(a,b,cb_n),
            (Value::FiniteNumber(a), Value::InfiniteNumber(b)) => merge2_s(a,b,cb_n),
            (Value::FiniteString(a), Value::FiniteString(b)) => merge2_ss(a,b,cb_s),
            (Value::FiniteString(a), Value::InfiniteString(b)) => merge2_s(a,b,cb_s),
            (Value::InfiniteBoolean(a), Value::Boolean(b)) => { cb_b(a,b); },
            (Value::InfiniteBoolean(a), Value::InfiniteBoolean(b)) => { cb_b(a,b); },
            (Value::InfiniteNumber(a), Value::Number(b)) => { cb_n(a,b); },
            (Value::InfiniteNumber(a), Value::InfiniteNumber(b)) => { cb_n(a,b); },
            (Value::InfiniteString(a), Value::String(b)) => { cb_s(a,b); },
            (Value::InfiniteString(a), Value::InfiniteString(b)) => { cb_s(a,b); },
            (a,b) => { return Err(format!("invalid type combination {:?} and {:?}",a,b)); }
        }
        Ok(())
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Value::InfiniteBoolean(_) => false,
            Value::InfiniteNumber(_) => false,
            Value::InfiniteString(_) => false,
            _ => true
        }
    }

    pub fn is_atomic(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            Value::Number(_) => true,
            Value::String(_) => true,
            _ => false
        }
    }

    force!(force_boolean,force_boolean_mut,Boolean,bool,true);
    force!(force_number,force_number_mut,Number,f64,true);
    force!(force_string,force_string_mut,String,str,false);
    force!(force_infinite_boolean,force_infinite_boolean_mut,InfiniteBoolean,bool,true);
    force!(force_infinite_number,force_infinite_number_mut,InfiniteNumber,f64,true);
    force!(force_infinite_string,force_infinite_string_mut,InfiniteString,str,false);
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

fn number(d: &mut Decoder) -> Result<f64,Error> {
    Ok(if d.probe().i64().is_ok() { d.i64()? as f64 } else { d.f64()? })
}

fn number_from_array(d: &mut Decoder) -> Result<Vec<f64>,Error> {
    let mut out = vec![];
    cbor_array(d,&mut out,|_,out,d| {
        out.push(number(d)?);
        Ok(())
    })?;
    Ok(out)
}

impl<'b> Decode<'b,()> for Value {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut ()) -> Result<Self, Error> {
        Ok(match CborVariety::peek(d)? {
            CborVariety::Number => Value::Number(number(d)?),
            CborVariety::String => Value::String(d.str()?.to_string()),
            CborVariety::Boolean => Value::Boolean(d.bool()?),
            CborVariety::Array => {
                let mut p = d.probe();
                p.array().ok();
                match CborVariety::peek(&mut p)? {
                    CborVariety::Number => Value::FiniteNumber(number_from_array(d)?),
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
                        *out = Some(match CborVariety::peek(&mut p)? {
                            CborVariety::Number => Value::InfiniteNumber(number(d)?),
                            CborVariety::String => Value::InfiniteString(d.str()?.to_string()),
                            CborVariety::Boolean => Value::InfiniteBoolean(d.bool()?),
                            _ => { return Err(Error::message("bad constant")); }
                        });
                    } else if key == "e" {
                        let v = d.str()?;
                        *out = match v.chars().next() {
                            Some('b') => Some(Value::FiniteBoolean(vec![])),
                            Some('n') => Some(Value::FiniteNumber(vec![])),
                            Some('s') => Some(Value::FiniteString(vec![])),
                            Some('h') => Some(Value::FiniteNumber(vec![])),
                            _ => { return Err(Error::message("bad constant")); }

                        };
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
