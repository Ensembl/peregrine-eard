use std::convert::Infallible;

use minicbor::{Encoder, encode::{Error}};

use crate::model::{CompiledCode};

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

    pub fn serialize(&self) -> Result<Vec<u8>,String> {
        self.encode().map_err(|e| format!("cannot serialise: {}",e))
    }
}
