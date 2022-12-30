use std::{pin::Pin, future::Future};
use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::{Return, Operation}, interpreter::{InterpreterBuilder}, context::{RunContext, ContextItem}};
use super::{print::{op_print, op_format}, seqctors::{op_push_b2, op_push_b3, op_finseq_b, op_infseq_b, op_push_s2, op_push_s3, op_push_n2, op_finseq_s, op_infseq_s, op_finseq_n, op_infseq_n, op_push_n3}, checks::{op_len_n, op_len_s, op_len_b, op_total, op_bound}, arith::{op_max3, op_max2, op_min3, op_min2, op_max3s, op_max2s, op_min3s, op_min2s, op_max3ss, op_max2ss, op_min2ss, op_min3ss}};

pub trait LibcoreTemplate {
    fn print(&self, s: &str);
    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>>;
}

pub struct LibcoreBuilder {
    context: ContextItem<Box<dyn LibcoreTemplate>>
}

fn op_const(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.constants.get(regs[1]).ok_or_else(|| format!("missing constant {}",regs[1]))?;
        *ctx.registers.get_mut(regs[0])? = value.clone();
        Ok(Return::Sync)
    }))
}

fn op_copy(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone();
        ctx.set(regs[0],value)?;
        Ok(Return::Sync)
    }))
}

fn op_async(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_context = gctx.patterns.lookup::<Box<dyn LibcoreTemplate>>("libcore")?;
    Ok(Box::new(move |ctx,_regs| {
        let x = ctx.context.get(&libcore_context).call_up();
        Ok(Return::Async(x))
    }))
}

pub fn build_libcore(builder: &mut InterpreterBuilder) -> Result<LibcoreBuilder,String> {
    let context = builder.add_context::<Box<dyn LibcoreTemplate>>("libcore");
    builder.add_operation(0,Operation::new(op_const));
    builder.add_operation(1,Operation::new(op_async));
    builder.add_operation(3,Operation::new(op_push_n3));
    builder.add_operation(4,Operation::new(op_push_n2));
    builder.add_operation(5,Operation::new(op_len_n));
    builder.add_operation(6,Operation::new(op_total));
    builder.add_operation(7,Operation::new(op_bound));
    builder.add_operation(8,Operation::new(op_infseq_n));
    builder.add_operation(9,Operation::new(op_finseq_n));
    builder.add_operation(21,Operation::new(op_copy));
    builder.add_operation(41,Operation::new(op_infseq_s));
    builder.add_operation(42,Operation::new(op_finseq_s));
    builder.add_operation(43,Operation::new(op_push_s3));
    builder.add_operation(44,Operation::new(op_push_s2));
    builder.add_operation(45,Operation::new(op_len_s));
    builder.add_operation(46,Operation::new(op_max3));
    builder.add_operation(47,Operation::new(op_max2));
    builder.add_operation(51,Operation::new(op_infseq_b));
    builder.add_operation(52,Operation::new(op_finseq_b));
    builder.add_operation(53,Operation::new(op_push_b3));
    builder.add_operation(54,Operation::new(op_push_b2));
    builder.add_operation(55,Operation::new(op_len_b));
    builder.add_operation(56,Operation::new(op_min3));
    builder.add_operation(57,Operation::new(op_min2));
    builder.add_operation(137,Operation::new(op_print));
    builder.add_operation(138,Operation::new(op_format));
    builder.add_operation(141,Operation::new(op_max3s));
    builder.add_operation(142,Operation::new(op_max2s));
    builder.add_operation(143,Operation::new(op_max3ss));
    builder.add_operation(144,Operation::new(op_max2ss));
    builder.add_operation(145,Operation::new(op_min3s));
    builder.add_operation(146,Operation::new(op_min2s));
    builder.add_operation(147,Operation::new(op_min3ss));
    builder.add_operation(148,Operation::new(op_min2ss));
    Ok(LibcoreBuilder { context })
}

pub fn prepare_libcore<F>(context: &mut RunContext, builder: &LibcoreBuilder, libcore_template: F)
        where F: LibcoreTemplate + 'static {
    context.add(&builder.context,Box::new(libcore_template))
}
