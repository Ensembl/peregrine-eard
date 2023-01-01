use std::collections::HashMap;
use minicbor::{Decoder, Decode, decode::Error, data::Type};

use super::{globalcontext::GlobalBuildContext, value::Value, program::{ProgramBuilder, Program}};

pub(crate) fn cbor_map<'b,F,T>(d: &mut Decoder<'b>, obj: &mut T, mut cb: F) -> Result<(),Error>
        where F: FnMut(&str,&mut T,&mut Decoder<'b>) -> Result<(),Error> {
    let entries = d.map()?;
    let mut index = 0;
    loop {
        let key = d.str()?;
        (cb)(key,obj,d)?;
        index += 1;
        if let Some(len) = entries { if len == index { break; } }
        if let Type::Break = d.datatype()? { d.skip()?; break; }
    }
    Ok(())
}

fn cbor_array<'b,F,T>(d: &mut Decoder<'b>, obj: &mut T, mut cb: F) -> Result<(),Error>
        where F: FnMut(u64,&mut T,&mut Decoder<'b>) -> Result<(),Error> {
    let entries = d.array()?;
    let mut index = 0;
    loop {
        (cb)(index,obj,d)?;
        index += 1;
        if let Some(len) = entries { if len == index { break; } }
        if let Type::Break = d.datatype()? { d.skip()?; break; }
    }
    Ok(())           
}

#[derive(Clone,PartialEq,Eq,Hash,Debug,PartialOrd,Ord)]
pub struct Metadata {
    pub group: String,
    pub name: String,
    pub version: u32
}

impl Metadata {
    pub fn new(group: &str, name: &str, version: u32) -> Metadata {
        Metadata { group: group.to_string(), name: name.to_string(), version }
    }
}

impl<'b> Decode<'b,()> for Metadata {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut ()) -> Result<Self, Error> {
        let mut out = Metadata { group: "".to_string(), name: "".to_string(), version: 0 };
        cbor_array(d, &mut out, |idx,out,d| {
            match idx {
                0 => { out.group = d.str()?.to_string(); },
                1 => { out.name = d.str()?.to_string(); },
                2 => { out.version = d.u32()?; },
                _ => {}
            }
            Ok(())
        })?;
        Ok(out)
    }
}

#[derive(Debug,Clone)]
pub(crate) struct CompiledBlock {
    pub(crate) constants: Vec<Value>,
    pub(crate) program: Vec<(usize,Vec<usize>)>
}

impl CompiledBlock {
    pub(crate) fn to_program(&self, gbctx: &GlobalBuildContext, program: &mut ProgramBuilder) -> Result<Program,String> {
        for (i,c) in self.constants.iter().enumerate() {
            program.add_constant(i,c.clone());
        }
        for (opcode,regs) in &self.program {
            program.add_opcode(gbctx,*opcode,regs.to_vec())?;
        }
        Ok(program.to_program())
    }
}

impl<'b> Decode<'b,()> for CompiledBlock {
    fn decode(d: &mut Decoder<'b>, ctx: &mut ()) -> Result<Self, Error> {
        let mut out = CompiledBlock { constants: vec![], program: vec![] };
        cbor_map(d,&mut out,|key,out,d| {
            if key == "constants" {
                cbor_array(d,&mut out.constants,|_,out,d| {
                    out.push(Value::decode(d,ctx)?);
                    Ok(())
                })?;
            } else if key == "program" {
                cbor_array(d,&mut out.program, |_,out,d| {
                    let mut instr = vec![];
                    cbor_array(d,&mut instr, |_,out,d| {
                        out.push(d.u32()? as usize);
                        Ok(())
                    })?;
                    if instr.len() > 0 {
                        let opcode = instr.remove(0);
                        out.push((opcode,instr));
                    }
                    Ok(())
                })?;
            } else {
                d.skip()?;
            }
            Ok(())
        })?;
        Ok(out)
    }
}

#[derive(Debug,Clone)]
pub(crate) struct CompiledCode {
    pub(crate) metadata: Metadata,
    pub(crate) code: HashMap<String,CompiledBlock>
}

impl<'b> Decode<'b,()> for CompiledCode {
    fn decode(d: &mut Decoder<'b>, ctx: &mut ()) -> Result<Self, Error> {
        let mut out = (None,HashMap::new());
        cbor_map(d,&mut out,|key,out,d| {
            let (metadata,code) = out;
            if key == "metadata" { *metadata = Some(Metadata::decode(d,ctx)?); }
            else if key == "blocks" {
                for res in d.map_iter::<String,CompiledBlock>()? {
                    let (key,value) = res?;
                    code.insert(key,value);
                }
            } else {
                d.skip()?;
            }
            Ok(())
        })?;
        Ok(CompiledCode { metadata: out.0.unwrap(), code: out.1 })
    }
}

#[derive(Debug)]
struct OpcodeVersion {
    version: HashMap<String,(u32,u32)>
}

impl OpcodeVersion {
    fn new() -> OpcodeVersion {
        OpcodeVersion { version: HashMap::new() }
    }

    fn decode(&mut self, d: &mut Decoder) -> Result<(),Error> {
        cbor_map(d,self,|key,out,d| {
            let mut ver = vec![0,0];
            cbor_array(d,&mut ver,|i,out,d| {
                out[i as usize] = d.u32()?;
                Ok(())
            })?;
            let ver = if let (Some(a),Some(b)) = (ver.get(0),ver.get(1)) {
                (*a,*b)
            } else {
                return Err(Error::message(format!("bad version")));
            };
            out.version.insert(key.to_string(),ver);
            Ok(())
        })?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ObjectFile {
    version: OpcodeVersion,
    pub(crate) code: Vec<CompiledCode>
}

impl ObjectFile {
    pub(crate) fn decode(bytes: Vec<u8>) -> Result<ObjectFile,Error> {
        let mut decoder = Decoder::new(&bytes);
        let mut out = ObjectFile {
            version: OpcodeVersion::new(),
            code: vec![]
        };
        cbor_map(&mut decoder, &mut out, |key,out,d| {
            if key == "code" {
                for part in d.array_iter::<CompiledCode>()? {
                    eprintln!("adding part");
                    out.code.push(part?);
                } 
            } else if key == "version" {
                out.version.decode(d)?;
            }
            Ok(())
        })?;
        Ok(out)
    }
}
