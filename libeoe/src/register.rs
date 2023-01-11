use eachorevery::eoestruct::{StructTemplate, StructVarGroup, StructPair, StructVar};
use eard_interp::{ContextItem, HandleStore, InterpreterBuilder, Operation, RunContext};
use crate::ops::{op_boolean, op_number, op_string, op_null, op_group, op_var_boolean, op_var_number, op_var_string, op_array, op_pair, op_object, op_var, op_all, op_condition};

pub struct LibEoEBuilder {
    templates: ContextItem<HandleStore<StructTemplate>>,
    groups: ContextItem<HandleStore<StructVarGroup>>,
    vars: ContextItem<HandleStore<StructVar>>,
    pairs: ContextItem<HandleStore<StructPair>>,
}

/* todo: replace on interp with mini stack machine for complex constructions */
pub fn build_libeoe(builder: &mut InterpreterBuilder) -> Result<LibEoEBuilder,String> {
    let templates = builder.add_context::<HandleStore<StructTemplate>>("eoetemplates")?;
    let groups = builder.add_context::<HandleStore<StructVarGroup>>("eoegroups")?;
    let pairs = builder.add_context::<HandleStore<StructPair>>("eoepairs")?;
    let vars = builder.add_context::<HandleStore<StructVar>>("eoevars")?;
    builder.add_version("libeoe",(0,0));
    builder.add_operation(512,Operation::new(op_boolean));
    builder.add_operation(513,Operation::new(op_number));
    builder.add_operation(514,Operation::new(op_string));
    builder.add_operation(515,Operation::new(op_null));
    builder.add_operation(516,Operation::new(op_group));
    builder.add_operation(517,Operation::new(op_var_boolean));
    builder.add_operation(518,Operation::new(op_var_number));
    builder.add_operation(519,Operation::new(op_var_string));
    builder.add_operation(520,Operation::new(op_array));
    builder.add_operation(521,Operation::new(op_pair));
    builder.add_operation(522,Operation::new(op_object));
    builder.add_operation(523,Operation::new(op_var));
    builder.add_operation(524,Operation::new(op_all));
    builder.add_operation(525,Operation::new(op_condition));
    Ok(LibEoEBuilder { templates, groups, pairs, vars })
}

pub fn prepare_libeoe(context: &mut RunContext, builder: &LibEoEBuilder) -> Result<(),String> {
    context.add(&builder.templates,HandleStore::new());
    context.add(&builder.groups,HandleStore::new());
    context.add(&builder.pairs,HandleStore::new());
    context.add(&builder.vars,HandleStore::new());
    Ok(())
}
