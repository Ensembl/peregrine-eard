use std::{convert::Infallible, fmt, collections::HashMap};
use json::{JsonValue, object::Object};
use minicbor::{Encoder, encode::Error};
use crate::test::testutil::sepfmt;
use super::constants::FullConstant;

#[derive(Clone)]
pub(crate) struct Metadata {
    pub(crate) group: String,
    pub(crate) name: String,
    pub(crate) version: u32
}

impl Metadata {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_array()?.str(&self.group)?.str(&self.name)?.u32(self.version)?.end()?;
        Ok(())
    }

    fn encode_json(&self) -> JsonValue {
        JsonValue::Array(vec![
            JsonValue::String(self.group.to_string()),
            JsonValue::String(self.name.to_string()),
            JsonValue::Number(self.version.into())
        ])
    }
}

pub(crate) struct CompiledBlock {
    pub(crate) constants: Vec<FullConstant>,
    pub(crate) program: Vec<(usize,Vec<usize>)>
}

impl CompiledBlock {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_map()?.str("constants")?.begin_array()?;
        for c in &self.constants {
            c.encode(encoder)?;
        }
        encoder.end()?.str("program")?.begin_array()?;
        for (opcode,opargs) in &self.program {
            encoder.array((opargs.len()+1) as u64)?.u32(*opcode as u32)?;
            for oparg in opargs {
                encoder.u32(*oparg as u32)?;
            }
        }
        encoder.end()?.end()?; /* program entry; main map */
        Ok(())
    }

    fn encode_json(&self) -> JsonValue {
        let mut out = Object::new();
        let constants = JsonValue::Array(self.constants.iter().map(|c| c.encode_json()).collect());
        out.insert("constants",constants);
        let mut program = vec![];
        for (opcode,opargs) in &self.program {
            let mut value = vec![*opcode];
            value.extend(opargs.iter());
            let value = value.drain(..).map(|v| JsonValue::Number(v.into())).collect();
            program.push(JsonValue::Array(value));
        }
        out.insert("program",JsonValue::Array(program));
        JsonValue::Object(out)
    }
}

impl fmt::Debug for CompiledBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"  constants:\n{}\n  program:\n{}\n",
            sepfmt(&mut self.constants.iter(),"\n","    "),
            sepfmt(&mut self.program.iter(),"\n","    ")
        )
    }
}

pub struct CompiledCode {
    pub(crate) metadata: Metadata,
    pub(crate) code: HashMap<String,CompiledBlock>
}

impl CompiledCode {
    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_map()?.str("metadata")?;
        self.metadata.encode(encoder)?;
        encoder.str("blocks")?.begin_map()?;
        for (name,block) in self.code.iter() {
            encoder.str(name)?;
            block.encode(encoder)?;
        }
        encoder.end()?.end()?;
        Ok(())
    }

    pub(crate) fn encode_json(&self) -> JsonValue {
        let mut out = Object::new();
        out.insert("metadata",self.metadata.encode_json());
        let mut blocks = Object::new();
        for (name,block) in self.code.iter() {
            blocks.insert(&name,block.encode_json());
        }
        out.insert("blocks",JsonValue::Object(blocks));
        JsonValue::Object(out)
    }
}

impl fmt::Debug for CompiledCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys = self.code.keys().collect::<Vec<_>>();
        keys.sort();
        for block in keys.iter() {
            let code = self.code.get(*block).unwrap();
            write!(f,"block: {}\n{:?}\n",block,code)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Opcode(pub usize,pub Vec<usize>);

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"opcode {}{}{};",
                self.0,
                if self.1.len() > 0 { ", " } else { "" },
                sepfmt(&mut self.1.iter(),", ","r")
        )
    }
}
