use std::{collections::HashMap, fmt};
use serde::{de::Visitor, Deserialize, Deserializer};

enum OneValue {
    Boolean(bool),
    Number(f64),
    String(String)
}

struct OneValueVisitor;

impl<'de> Visitor<'de> for OneValueVisitor {
    type Value = OneValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a OneValue")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(OneValue::Boolean(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(OneValue::Number(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(OneValue::Number(v as f64))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(OneValue::Number(v as f64))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(OneValue::String(v.to_string()))
    }
}

impl<'de> Deserialize<'de> for OneValue {
    fn deserialize<D>(deserializer: D) -> Result<OneValue, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_any(OneValueVisitor)
    }
}

#[derive(Clone)]
pub(crate) enum DataValue {
    Empty,
    Boolean(Vec<bool>),
    Number(Vec<f64>),
    String(Vec<String>)
}

impl DataValue {
    pub(crate) fn first_string(&self) -> Option<String> {
        match self {
            DataValue::String(s) if s.len() > 0 => Some(s[0].to_string()),
            _ => None
        }
    }
}

fn downcast_bool(input: &[OneValue]) -> DataValue {
    let mut out = vec![];
    for v in input {
        match v {
            OneValue::Boolean(b) => out.push(*b),
            _ => {}
        }
    }
    DataValue::Boolean(out)
}

fn downcast_number(input: &[OneValue]) -> DataValue {
    let mut out = vec![];
    for v in input {
        match v {
            OneValue::Number(n) => out.push(*n),
            _ => {}
        }
    }
    DataValue::Number(out)
}


fn downcast_string(input: &[OneValue]) -> DataValue {
    let mut out = vec![];
    for v in input {
        match v {
            OneValue::String(s) => out.push(s.to_string()),
            _ => {}
        }
    }
    DataValue::String(out)
}

struct DataValueVisitor;

impl<'de> Visitor<'de> for DataValueVisitor {
    type Value = DataValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a DataValue")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let mut values = vec![];
        while let Some(el) = seq.next_element::<OneValue>()? {
            values.push(el);
        }
        if values.len() == 0 {
            Ok(DataValue::Empty)
        } else {
            Ok(match &values[0] {
                OneValue::Boolean(_) => downcast_bool(&values),
                OneValue::Number(_) => downcast_number(&values),
                OneValue::String(_) => downcast_string(&values)
            })
        }
    }
}

impl<'de> Deserialize<'de> for DataValue {
    fn deserialize<D>(deserializer: D) -> Result<DataValue, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(DataValueVisitor)
    }
}

#[derive(serde_derive::Deserialize,Clone)]
pub(crate) struct Response(HashMap<String,DataValue>);

impl Response {
    pub(crate) fn get(&self, key: &str) -> Result<&DataValue,String> {
        self.0.get(key).ok_or_else(||
            format!("no such stream {}",key)
        )
    }
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct EndpointResponse(HashMap<String,Response>);

#[derive(serde_derive::Deserialize)]
pub struct StubResponses(HashMap<String,EndpointResponse>);

impl StubResponses {
    pub fn empty() -> StubResponses {
        StubResponses(HashMap::new())
    }

    fn try_get(&self, backend: &str, endpoint: &str) -> Option<&Response> {
        self.0.get(backend)?.0.get(endpoint)
    }

    pub(crate) fn get(&self, backend: &str, endpoint: &str) -> Result<&Response,String> {    
        self.try_get(backend,endpoint).ok_or_else(|| format!("No stubbed response data for {} {}",backend,endpoint))
    }

    pub(crate) fn get_setting(&self, key: &str, path: &[String]) -> Result<&DataValue,String> {
        let settings = self.0.get("__settings").ok_or_else(|| {
            format!("no settings key in stub but settings used by program")
        })?;
        Ok(settings.0.get(key).and_then(|r| {
            r.get(&path.join("/")).ok()
        }).unwrap_or(&DataValue::Empty))
    }

    pub(crate) fn get_small_value(&self, namespace: &str, column: &str, key: &str) -> Option<String> {
        self.0.get("__small-values").and_then(|settings| {
            settings.0.get(&format!("{}/{}",namespace,column)).and_then(|r| {
                r.get(key).ok()?.first_string()
            })
        })
    }

    pub(crate) fn get_request(&self, key: &str, part: &str) -> Result<&DataValue,String> {
        let settings = self.0.get("__request").ok_or_else(|| {
            format!("no request key in stub but request settings used by program")
        })?;
        Ok(settings.0.get(key).and_then(|r| {
            r.get(part).ok()
        }).unwrap_or(&DataValue::Empty))
    }
}
