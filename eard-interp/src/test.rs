use std::{time::Duration};
use async_std::task::{self, block_on};
use crate::{globalcontext::{GlobalContext, GlobalBuildContext}, operation::{Operation, Return, OperationStore}, interpreter::{self, Interpreter, InterpreterBuilder}, objectcode::{CompiledBlock, Metadata, CompiledCode, ObjectFile}, context::RunContext, value::Value};

fn wait(ctx: &mut GlobalContext, regs: &[usize]) -> Return {
    Return::Async(Box::pin(task::sleep(Duration::from_millis(100))))
}

fn printer(s: &str) {
    eprintln!("{}",s);
}

fn print(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Return>,String> {
    let printer = gctx.patterns.lookup::<Box<dyn Fn(&str)>>("printer")?;
    Ok(Box::new(move |ctx: &mut GlobalContext,_regs| {
        ctx.context.get(&printer)("hello");
        Return::Sync
    }))
}

fn load_example_program(interp: &mut Interpreter, metadata: &Metadata) {
    let block = CompiledBlock {
        constants: vec![],
        program: vec![
            (1,vec![])
        ]
    };
    interp.build(&metadata,"main",block).expect("build");
}

#[test]
fn test_smoke() {
    /* build interpreter */
    let mut builder = InterpreterBuilder::new();
    let printer_ctx = builder.add_context::<Box<dyn Fn(&str)>>("printer");
    builder.add_operation(1,Operation::new(print));
    let mut interp = Interpreter::new(builder);
    /* load example program */
    let metadata = Metadata {
        group: "test".to_string(),
        name: "test".to_string(),
        version: 1
    };
    load_example_program(&mut interp,&metadata);
    /* run example program */
    let mut context = RunContext::new();
    context.add(printer_ctx,Box::new(|s| printer(s)));
    block_on(interp.run(&metadata,"main",context)).expect("run");
}

fn test_const(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Return>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.constants.get(regs[1]).expect("bad constant");
        *ctx.registers.get_mut(regs[0]).expect("bad reg") = value.clone();
        Return::Sync
    }))
}

fn test_format(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Return>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.registers.get(regs[0]).expect("bad reg");
        let out = format!("{:?}",value);
        *ctx.registers.get_mut(regs[1]).expect("bad reg") = Value::String(out);
        Return::Sync
    }))
}

fn test_print(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Return>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.registers.get(regs[0]).expect("bad reg");
        match value {
            Value::String(s) => eprintln!("{}",s),
            _ => {}
        }
        Return::Sync
    }))
}

#[test]
fn test_load() {
    let block = ObjectFile::decode(include_bytes!("test/hello.eardo").to_vec()).expect("decoding");
    let mut builder = InterpreterBuilder::new();
    builder.add_operation(0,Operation::new(test_const));
    builder.add_operation(137,Operation::new(test_print));
    builder.add_operation(138,Operation::new(test_format));
    let mut interp = Interpreter::new(builder);
    interp.build(&block.code[0].metadata, "main",block.code[0].code.get("main").unwrap().clone()).expect("build error");
    let mut context = RunContext::new();
    block_on(interp.run(&block.code[0].metadata,"main",context)).expect("run");
}
