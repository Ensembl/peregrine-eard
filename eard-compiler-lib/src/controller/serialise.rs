use std::{convert::Infallible, collections::HashMap};
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

#[derive(Clone,Debug)]
pub(crate) struct OpcodeVersion {
    versions: HashMap<String,(u32,u32)>
}

impl OpcodeVersion {
    pub(crate) fn new() -> OpcodeVersion {
        let mut versions = HashMap::new();
        versions.insert("core".to_string(),(0,0));
        OpcodeVersion { versions }
    }

    pub(crate) fn add(&mut self, name: &str, version: (u32,u32)) {
        self.versions.insert(name.to_string(),version);
    }

    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_map()?;
        for (name,version) in &self.versions {
            encoder.str(name)?;
            encoder.array(2)?;
            encoder.u32(version.0)?;
            encoder.u32(version.1)?;
        }
        encoder.end()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EardSerializeCode {
    code: Vec<CompiledCode>
}

impl EardSerializeCode {
    pub fn new() -> EardSerializeCode {
        EardSerializeCode { code: vec![] }
    }

    pub fn add(&mut self, code: CompiledCode) {
        self.code.push(code);
    }

    fn encode(&self) -> Result<Vec<u8>,Error<Infallible>> {
        let mut buffer = vec![];
        let mut encoder = Encoder::new(&mut buffer);
        encoder.begin_array()?;
        for code in &self.code {
            code.encode(&mut encoder)?;
        }
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
