use std::collections::HashMap;
use minicbor::{Decoder, Decode, decode::Error, data::Type};
use super::{globalcontext::GlobalBuildContext, value::Value, program::{ProgramBuilder, Program}, version::OpcodeVersion};

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

pub(crate) fn cbor_array<'b,F,T>(d: &mut Decoder<'b>, obj: &mut T, mut cb: F) -> Result<(),Error>
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
pub struct ProgramName {
    pub group: String,
    pub name: String,
    pub version: u32
}

impl ProgramName {
    pub fn new(group: &str, name: &str, version: u32) -> ProgramName {
        ProgramName { group: group.to_string(), name: name.to_string(), version }
    }
}

impl<'b> Decode<'b,()> for ProgramName {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut ()) -> Result<Self, Error> {
        let mut out = ProgramName { group: "".to_string(), name: "".to_string(), version: 0 };
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
pub struct Metadata {
    pub(crate) name: ProgramName,
    pub(crate) version: OpcodeVersion
}

impl Metadata {
    fn decode<'b>(d: &mut Decoder<'b>, ctx: &mut ()) -> Result<Self, Error> {
        let mut data = (None,None);
        cbor_map(d, &mut data,|key,md,d| {
            if key == "name" {
                md.0 = Some(ProgramName::decode(d,ctx)?);
            } else if key == "version" {
                md.1 = Some(OpcodeVersion::decode(d)?);
            }
            Ok(())
        })?;
        if data.0.is_none() || data.1.is_none() {
            return Err(Error::message(format!("bad metadata block")));
        }
        Ok(Metadata { name: data.0.unwrap(), version: data.1.unwrap() })
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
            if key == "metadata" {
                *metadata = Some(Metadata::decode(d,ctx)?);
            } else if key == "blocks" {
                for res in d.map_iter::<String,CompiledBlock>()? {
                    let (key,value) = res?;
                    code.insert(key,value);
                }
            } else {
                d.skip()?;
            }
            Ok(())
        })?;
        let metadata = out.0.unwrap();
        Ok(CompiledCode { metadata, code: out.1 })
    }
}

#[derive(Debug)]
pub struct ObjectFile {
    pub(crate) code: Vec<CompiledCode>
}

impl ObjectFile {
    pub fn decode(bytes: Vec<u8>) -> Result<ObjectFile,Error> {
        let mut decoder = Decoder::new(&bytes);
        let mut out = ObjectFile {
            code: vec![]
        };
        for part in decoder.array_iter::<CompiledCode>()? {
            out.code.push(part?);
        } 
        Ok(out)
    }

    pub fn list_programs(&self) -> Vec<ProgramName> {
        self.code.iter().map(|c| c.metadata.name.clone()).collect()
    }
}
