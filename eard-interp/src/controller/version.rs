use std::collections::HashMap;

use minicbor::{Decoder, decode::Error};

use super::objectcode::{cbor_map, cbor_array};

#[derive(Debug,Clone)]
pub(crate) struct OpcodeVersion {
    version: HashMap<String,(u32,u32)>
}

impl OpcodeVersion {
    pub(crate) fn new() -> OpcodeVersion {
        OpcodeVersion { version: HashMap::new() }
    }

    pub(crate) fn add_version(&mut self, name: &str, version: (u32,u32)) {
        self.version.insert(name.to_string(),version);
    }

    pub(crate) fn meets_minimums(&self, minimums: &OpcodeVersion) -> Result<(),String> {
        for (name,encoding) in &self.version {
            if let Some(minimum) = minimums.version.get(name) {
                if minimum.0 != encoding.0 {
                    return Err(format!("library {:?} major versions support={} source={}",name,minimum.0,encoding.0));
                }
                if minimum.1 < encoding.1 {
                    return Err(format!("library {:?} minor versions support<={} source={}",name,minimum.1,encoding.1));
                }
            } else {
                return Err(format!("unsupported library {:?}",name));
            }
        }
        Ok(())
    }

    pub(crate) fn decode(d: &mut Decoder) -> Result<OpcodeVersion,Error> {
        let mut out = OpcodeVersion::new();
        cbor_map(d,&mut out,|key,out,d| {
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
        Ok(out)
    }
}
