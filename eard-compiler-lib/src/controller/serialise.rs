use std::convert::Infallible;
use json::JsonValue;
use minicbor::{Encoder, encode::{Error}};
use regex::Regex;

use crate::model::compiled::CompiledCode;

fn compactify(s: &str) -> String {
    let re1 = Regex::new(r"\n {12,}").unwrap();
    let re2 = Regex::new(r"\n {10,}([\]}])").unwrap();
    let s = re1.replace_all(s," ");
    let s = re2.replace_all(&s," $1").to_string();
    s
}

#[derive(Debug)]
pub struct EardSerializeCode {
    version: (u32,u32),
    code: Vec<CompiledCode>
}

impl EardSerializeCode {
    pub fn new() -> EardSerializeCode {
        EardSerializeCode { version: (0,0), code: vec![] }
    }

    pub fn add(&mut self, code: CompiledCode) {
        self.code.push(code);
    }

    fn encode(&self) -> Result<Vec<u8>,Error<Infallible>> {
        let mut buffer = vec![];
        let mut encoder = Encoder::new(&mut buffer);
        encoder.begin_map()?;
        encoder.str("version")?;
        encoder.begin_array()?;
        encoder.u32(self.version.0)?;
        encoder.u32(self.version.1)?;
        encoder.end()?;
        encoder.str("code")?;
        encoder.begin_array()?;
        for code in &self.code {
            code.encode(&mut encoder)?;
        }
        encoder.end()?;
        encoder.end()?;
        Ok(buffer)
    }

    fn encode_json(&self) -> JsonValue {
        JsonValue::Array(self.code.iter().map(|c| c.encode_json()).collect())
    }

    pub fn serialize(&self) -> Result<Vec<u8>,String> {
        self.encode().map_err(|e| format!("cannot serialise: {}",e))
    }

    pub fn serialize_json(&self) -> String {
        compactify(&self.encode_json().pretty(2))

    }
}
