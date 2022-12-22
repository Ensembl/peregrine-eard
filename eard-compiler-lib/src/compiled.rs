use crate::model::{CompiledCode, Step};

pub(crate) fn make_program(steps: &[Step]) -> CompiledCode {
    let mut constants = vec![];
    let mut program = vec![];
    for step in steps {
        match step {
            Step::Constant(r,c) => {
                program.push((0,vec![constants.len()]));
                constants.push(c.clone());
            },
            Step::Opcode(code,args) => {
                program.push((*code,args.to_vec()));
            },
        }
    }
    CompiledCode { constants, program }
}
