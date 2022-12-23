use std::collections::HashMap;
use crate::model::{CompiledCode, Step, CompiledBlock};

fn find_entries(steps: &[Step]) -> Vec<String> {
    steps.iter().filter_map(|step| {
        match step {
            Step::Entry(e) => Some(e.to_string()),
            _ => None
        }
    }).collect()
}

fn make_block(steps: &[Step], entry: &str) -> CompiledBlock {
    let mut constants = vec![];
    let mut program = vec![];
    let mut on = true;
    for step in steps {
        match step {
            Step::Constant(r,c) => {
                if on {
                    program.push((0,vec![r.clone(),constants.len()]));
                    constants.push(c.clone());
                }
            },
            Step::Opcode(code,args) => {
                if on {
                    program.push((*code,args.to_vec()));
                }
            },
            Step::Entry(this_entry) =>{
                on = this_entry == entry;
            },
        }
    }
    CompiledBlock { constants, program }
}

pub(crate) fn make_program(steps: &[Step]) -> CompiledCode {
    let entries = find_entries(steps);
    let code = entries.iter().map(|entry| (entry.to_string(),make_block(steps,entry))).collect::<HashMap<_,_>>();
    CompiledCode { code }
}
