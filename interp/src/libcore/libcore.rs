use std::{pin::Pin, future::Future};
use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::{Return, Operation, AsyncReturn}, interpreter::{InterpreterBuilder}, context::{RunContext, ContextItem}, handles::HandleStore};
use super::{
    print::{op_print, op_format, op_comma_format, op_comma_format_s}, 
    seqctors::{op_push_b2, op_push_b3, op_finseq_b, op_infseq_b, op_push_s2, op_push_s3, op_push_n2, op_finseq_s, op_infseq_s, op_finseq_n, op_infseq_n, op_push_n3},
    checks::{op_len_n, op_len_s, op_len_b, op_total, op_bound, op_check_l, op_check_t, op_check_b, op_check_tt, op_check_li, op_check_ii},
    arith::{op_max3, op_max2, op_min3, op_min2, op_max3s, op_max2s, op_min3s, op_min2s, op_max3ss, op_max2ss, op_min2ss, op_min3ss, op_add3, op_add2, op_add3s, op_add2s, op_add3ss, op_add2ss, op_sub2ss, op_sub3ss, op_sub2s, op_sub3s, op_sub2, op_sub3, op_mul3, op_mul2, op_div3, op_div2, op_mul3s, op_div3s, op_mul2s, op_div2s, op_mul3ss, op_mul2ss, op_div3ss, op_div2ss, op_gt, op_ge, op_gt_s, op_ge_s, op_gt_ss, op_ge_ss, op_eq_num, op_eq_str, op_eq_num_s, op_eq_str_s, op_eq_num_ss, op_eq_str_ss, op_mod3, op_mod2, op_mod3s, op_mod2s, op_mod3ss, op_mod2ss, op_max_s, op_min_s},
    opntn::{op_neg2, op_neg1, op_neg2s, op_neg1s}, 
    opbtb::{op_not2, op_not1, op_not2s, op_not1s},
    seq::{op_repeat, op_if, op_set, op_set_m, op_set_skip, op_set_skip_m, op_set_at, op_set_at_m, op_set_from, op_set_from_m, op_index, op_index_s, op_count, op_enumerate, op_any, op_all, op_position, op_select, op_if_s},
    opbbtb::{op_eq3_bool, op_eq3_bool_s, op_eq3_bool_ss, op_and3, op_and2_s, op_or3, op_or2_s, op_or2_ss, op_and2, op_and3_ss, op_or3_s, op_or3_ss, op_and3_s, op_and2_ss, op_or2}, 
    string::{op_concat, op_push_str, op_push_str_s, op_push_str_revs, op_split, op_template, op_split_start, op_split_get, Template, op_template_start, op_template_set, op_template_end, op_push_str_ss}, convert::{op_to_bool, op_to_bool_m, op_to_bool_s, op_to_bool_s_m, op_to_num, op_to_num_m, op_to_num_s, op_to_num_s_m, op_to_str, op_to_str_m, op_to_str_s, op_to_str_s_m}, bio::{op_base_flip, op_base_flip_s, op_ruler_interval, op_ruler_markings}
};

pub trait LibcoreTemplate {
    fn print(&self, s: &str);
    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>>;
}

#[derive(Clone)]
pub struct LibcoreBuilder {
    context: ContextItem<Box<dyn LibcoreTemplate>>,
    splits: ContextItem<HandleStore<Vec<Vec<String>>>>,
    templates: ContextItem<HandleStore<Template>>
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
        Ok(Return::Async(AsyncReturn::new::<(),_>(x,|_,_,_| Ok(()))))
    }))
}

pub fn build_libcore(builder: &mut InterpreterBuilder) -> Result<LibcoreBuilder,String> {
    let context = builder.add_context::<Box<dyn LibcoreTemplate>>("libcore")?;
    let splits = builder.add_context::<HandleStore<Vec<Vec<String>>>>("splits")?;
    let templates = builder.add_context::<HandleStore<Template>>("templates")?;
    builder.add_version("libcore",(0,0));
    builder.add_operation(0,Operation::new(op_const));
    builder.add_operation(1,Operation::new(op_async));
    /* 2 is reserved */
    builder.add_operation(3,Operation::new(op_push_n3));
    builder.add_operation(4,Operation::new(op_push_n2));
    builder.add_operation(5,Operation::new(op_len_n));
    builder.add_operation(6,Operation::new(op_total));
    builder.add_operation(7,Operation::new(op_bound));
    builder.add_operation(8,Operation::new(op_infseq_n));
    builder.add_operation(9,Operation::new(op_finseq_n));
    builder.add_operation(10,Operation::new(op_check_l));
    builder.add_operation(11,Operation::new(op_check_tt));
    builder.add_operation(12,Operation::new(op_select));
    builder.add_operation(13,Operation::new(op_check_t));
    builder.add_operation(14,Operation::new(op_check_b));
    builder.add_operation(15,Operation::new(op_check_ii));
    builder.add_operation(16,Operation::new(op_check_li));
    builder.add_operation(17,Operation::new(op_add3));
    builder.add_operation(18,Operation::new(op_add2));
    builder.add_operation(19,Operation::new(op_sub3));
    builder.add_operation(20,Operation::new(op_sub2));
    builder.add_operation(21,Operation::new(op_copy));
    builder.add_operation(22,Operation::new(op_mul3));
    builder.add_operation(23,Operation::new(op_mul2));
    builder.add_operation(24,Operation::new(op_div3));
    builder.add_operation(25,Operation::new(op_div2));
    builder.add_operation(26,Operation::new(op_gt));
    builder.add_operation(27,Operation::new(op_ge));
    builder.add_operation(28,Operation::new(op_not2));
    builder.add_operation(29,Operation::new(op_not1));
    builder.add_operation(30,Operation::new(op_eq_num));
    builder.add_operation(31,Operation::new(op_eq_str));
    builder.add_operation(32,Operation::new(op_eq3_bool));
    builder.add_operation(33,Operation::new(op_eq_num_s));
    builder.add_operation(34,Operation::new(op_eq_str_s));
    builder.add_operation(35,Operation::new(op_eq3_bool_s));
    builder.add_operation(36,Operation::new(op_eq_num_ss));
    builder.add_operation(37,Operation::new(op_eq_str_ss));
    builder.add_operation(38,Operation::new(op_eq3_bool_ss));
    builder.add_operation(39,Operation::new(op_not2s));
    builder.add_operation(40,Operation::new(op_not1s));
    builder.add_operation(41,Operation::new(op_infseq_s));
    builder.add_operation(42,Operation::new(op_finseq_s));
    builder.add_operation(43,Operation::new(op_push_s3));
    builder.add_operation(44,Operation::new(op_push_s2));
    builder.add_operation(45,Operation::new(op_len_s));
    builder.add_operation(46,Operation::new(op_max3));
    builder.add_operation(47,Operation::new(op_max2));
    builder.add_operation(48,Operation::new(op_gt_s));
    builder.add_operation(49,Operation::new(op_ge_s));
    builder.add_operation(50,Operation::new(op_repeat));
    builder.add_operation(51,Operation::new(op_infseq_b));
    builder.add_operation(52,Operation::new(op_finseq_b));
    builder.add_operation(53,Operation::new(op_push_b3));
    builder.add_operation(54,Operation::new(op_push_b2));
    builder.add_operation(55,Operation::new(op_len_b));
    builder.add_operation(56,Operation::new(op_min3));
    builder.add_operation(57,Operation::new(op_min2));
    builder.add_operation(58,Operation::new(op_gt_ss));
    builder.add_operation(59,Operation::new(op_ge_ss));
    builder.add_operation(60,Operation::new(op_add3s));
    builder.add_operation(61,Operation::new(op_add2s));
    builder.add_operation(62,Operation::new(op_sub3s));
    builder.add_operation(63,Operation::new(op_sub2s));
    builder.add_operation(64,Operation::new(op_mul3s));
    builder.add_operation(65,Operation::new(op_mul2s));
    builder.add_operation(66,Operation::new(op_div3s));
    builder.add_operation(67,Operation::new(op_div2s));
    builder.add_operation(68,Operation::new(op_neg2));
    builder.add_operation(69,Operation::new(op_neg1));
    builder.add_operation(70,Operation::new(op_add3ss));
    builder.add_operation(71,Operation::new(op_add2ss));
    builder.add_operation(72,Operation::new(op_sub3ss));
    builder.add_operation(73,Operation::new(op_sub2ss));
    builder.add_operation(74,Operation::new(op_mul3ss));
    builder.add_operation(75,Operation::new(op_mul2ss));
    builder.add_operation(76,Operation::new(op_div3ss));
    builder.add_operation(77,Operation::new(op_div2ss));
    builder.add_operation(78,Operation::new(op_neg2s));
    builder.add_operation(79,Operation::new(op_neg1s));
    builder.add_operation(80,Operation::new(op_and3));
    builder.add_operation(81,Operation::new(op_and2));
    builder.add_operation(82,Operation::new(op_and3_s));
    builder.add_operation(83,Operation::new(op_and2_s));
    builder.add_operation(84,Operation::new(op_and3_ss));
    builder.add_operation(85,Operation::new(op_and2_ss));
    builder.add_operation(86,Operation::new(op_or3));
    builder.add_operation(87,Operation::new(op_or2));
    builder.add_operation(88,Operation::new(op_or3_s));
    builder.add_operation(89,Operation::new(op_or2_s));
    builder.add_operation(90,Operation::new(op_or3_ss));
    builder.add_operation(91,Operation::new(op_or2_ss));
    builder.add_operation(92,Operation::new(op_if));
    builder.add_operation(93,Operation::new(op_if_s));
    builder.add_operation(94,Operation::new(op_set));
    builder.add_operation(95,Operation::new(op_set_m));
    builder.add_operation(96,Operation::new(op_set_skip));
    builder.add_operation(97,Operation::new(op_set_skip_m));
    builder.add_operation(98,Operation::new(op_set_at));
    builder.add_operation(99,Operation::new(op_set_at_m));
    builder.add_operation(100,Operation::new(op_set_from));
    builder.add_operation(101,Operation::new(op_set_from_m));
    builder.add_operation(102,Operation::new(op_index));
    builder.add_operation(103,Operation::new(op_index_s));
    builder.add_operation(104,Operation::new(op_count));
    builder.add_operation(105,Operation::new(op_enumerate));
    builder.add_operation(106,Operation::new(op_concat));
    builder.add_operation(107,Operation::new(op_push_str));
    builder.add_operation(108,Operation::new(op_push_str_s));
    builder.add_operation(109,Operation::new(op_split));
    builder.add_operation(110,Operation::new(op_template));
    builder.add_operation(111,Operation::new(op_split_start));
    builder.add_operation(112,Operation::new(op_split_get));
    builder.add_operation(113,Operation::new(op_template_start));
    builder.add_operation(114,Operation::new(op_template_set));
    builder.add_operation(115,Operation::new(op_template_end));
    builder.add_operation(116,Operation::new(op_any));
    builder.add_operation(117,Operation::new(op_all));
    builder.add_operation(118,Operation::new(op_position));
    builder.add_operation(119,Operation::new(op_mod3));
    builder.add_operation(120,Operation::new(op_mod2));
    builder.add_operation(121,Operation::new(op_mod3s));
    builder.add_operation(122,Operation::new(op_mod2s));
    builder.add_operation(123,Operation::new(op_mod3ss));
    builder.add_operation(124,Operation::new(op_mod2ss));
    builder.add_operation(125,Operation::new(op_to_bool));
    builder.add_operation(126,Operation::new(op_to_bool_m));
    builder.add_operation(127,Operation::new(op_to_bool_s));
    builder.add_operation(128,Operation::new(op_to_bool_s_m));
    builder.add_operation(129,Operation::new(op_to_num));
    builder.add_operation(130,Operation::new(op_to_num_m));
    builder.add_operation(131,Operation::new(op_to_num_s));
    builder.add_operation(132,Operation::new(op_to_num_s_m));
    builder.add_operation(133,Operation::new(op_to_str));
    builder.add_operation(134,Operation::new(op_to_str_m));
    builder.add_operation(135,Operation::new(op_to_str_s));
    builder.add_operation(136,Operation::new(op_to_str_s_m));
    builder.add_operation(137,Operation::new(op_print));
    builder.add_operation(138,Operation::new(op_format));
    builder.add_operation(139,Operation::new(op_max_s));
    builder.add_operation(140,Operation::new(op_min_s));
    builder.add_operation(141,Operation::new(op_max3s));
    builder.add_operation(142,Operation::new(op_max2s));
    builder.add_operation(143,Operation::new(op_max3ss));
    builder.add_operation(144,Operation::new(op_max2ss));
    builder.add_operation(145,Operation::new(op_min3s));
    builder.add_operation(146,Operation::new(op_min2s));
    builder.add_operation(147,Operation::new(op_min3ss));
    builder.add_operation(148,Operation::new(op_min2ss));
    builder.add_operation(149,Operation::new(op_base_flip));
    builder.add_operation(150,Operation::new(op_base_flip_s));
    builder.add_operation(151,Operation::new(op_ruler_interval));
    builder.add_operation(152,Operation::new(op_ruler_markings));
    builder.add_operation(153,Operation::new(op_comma_format));
    builder.add_operation(154,Operation::new(op_comma_format_s));
    builder.add_operation(155,Operation::new(op_push_str_ss));
    builder.add_operation(156,Operation::new(op_push_str_revs));
    Ok(LibcoreBuilder { context, splits, templates })
}

pub fn prepare_libcore<F>(context: &mut RunContext, builder: &LibcoreBuilder, libcore_template: F)
        where F: LibcoreTemplate + 'static {
    context.add(&builder.context,Box::new(libcore_template));
    context.add(&builder.splits,HandleStore::new());
    context.add(&builder.templates,HandleStore::new());
}
