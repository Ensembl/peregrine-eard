use ordered_float::OrderedFloat;
use pest_consume::{Parser, Error, match_nodes};
use crate::{model::{checkstypes::{CheckType, Check, AtomicTypeSpec, TypeSpec, TypedArgument, ArgTypeSpec}, constants::Constant, compiled::Opcode, codeblocks::{CodeImplArgument, CodeReturn, CodeImplVariable, CodeArgument, ImplBlock, CodeBlock, CodeModifier}}, controller::source::ParsePosition};
use super::{parsetree::{ PTExpression, PTCall, PTFuncDef, PTProcDef, PTStatement, PTStatementValue, FuncProcModifier, CodeArgModifier }, femodel::{OrBundle, OrBundleRepeater}, buildtree::Variable};

#[derive(Parser)]
#[grammar = "frontend/eard.pest"]
struct EardParser;

#[derive(Clone)]
struct ParseFixedState {
    position: ParsePosition,
    context: usize,
    optimise: bool
}

#[allow(unused)]
pub type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule, ParseFixedState>;

pub fn parse_string(input: &str) -> Result<String,String> {
    /* Same as JSON's definition plus \0 */
    let mut out = String::new();
    let mut u_left = 0;
    let mut u_val = 0;
    let mut seen_bs = false;
    for c in input.chars() {
        if seen_bs {
            if u_left > 0 {
                u_left -= 1;
                u_val = u_val*16 + c.to_digit(16).unwrap();
                seen_bs = u_left != 0;
                if !seen_bs { 
                    out.push(char::from_u32(u_val).ok_or_else(|| format!("bad unicode"))?);
                }
            } else {
                seen_bs = false;
                match c {
                    'r' => out.push('\r'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'f' => out.push('\x12'),
                    'b' => out.push('\x08'),
                    '0' => out.push('\0'),
                    'u' => { u_left = 4; u_val = 0; seen_bs = true; },
                    x => out.push(x)
                }
            }
        } else if c == '\\' {
            seen_bs = true;
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

macro_rules! left_rec {
    ($expr:tt,$infix:tt,$input:expr) => {{
        let mut out = None;
        let mut op = None;
        for child in $input.into_children() {
            match child.as_rule() {
                Rule::$expr => {
                    let new = Self::$expr(child)?;
                    if let Some(prev) = out {
                        out = Some(PTExpression::Infix(Box::new(prev),op.clone().unwrap(),Box::new(new)));
                    } else {
                        out = Some(new);
                    }
                },
                Rule::$infix => {
                    op = Some(child.as_str().trim().to_string());
                },
                _ => unreachable!()
            }
        }
        Ok(out.unwrap())
    }};
}

fn check(input: Node, check_type: CheckType) -> PestResult<Check> {
    Ok(Check { check_type, name: input.into_children().next().unwrap().as_str().to_string(), force: false })
}

fn decl_check(input: Node, check_type: CheckType) -> PestResult<Check> {
    let mut kids = input.into_children();
    let force = kids.next().unwrap();
    let id = kids.next().unwrap();
    Ok(Check { check_type, name: id.as_str().to_string(), force: force.as_str() == "!" })
}

#[pest_consume::parser]
impl EardParser {
    fn string_inner(input: Node) -> PestResult<String> {
        parse_string(input.as_str()).map_err(|e| input.error(e))
    }

    fn multi_string_inner(input: Node) -> PestResult<String> {
        parse_string(input.as_str()).map_err(|e| input.error(e))
    }

    fn string(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [string_inner(s)] => s,
          [multi_string_inner(s)] => s,
        ))
    }

    fn world(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn debug(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

    fn fold(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [identifier(s)] => s ))
    }

    fn special(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [identifier(s)] => s ))
    }

    fn code_modifier(input: Node) -> PestResult<Option<CodeModifier>> {
        let optimise = input.user_data().optimise;
        Ok(match_nodes!(input.into_children();
          [world(_)] => Some(CodeModifier::World),
          [debug(_)] => if optimise { None } else  { Some(CodeModifier::World) },
          [fold(f)] => Some(CodeModifier::Fold(f.to_string())),
          [special(f)] => Some(CodeModifier::Special(f.to_string()))
        ))
    }

    fn export(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn entry(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn version_spec_part(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

    fn version_spec(input: Node) -> PestResult<Vec<String>> {
        Ok(match_nodes!(input.into_children();
          [version_spec_part(p)..] => p.collect()
        ))
    }

    fn version(input: Node) -> PestResult<Vec<String>> {
        Ok(match_nodes!(input.into_children();
          [version_spec(p)] => p
        ))
    }

    fn func_modifier(input: Node) -> PestResult<FuncProcModifier> {
        Ok(match_nodes!(input.into_children();
          [export(_)] => FuncProcModifier::Export,
          [version(v)] => FuncProcModifier::Version(v),
        ))
    }

    fn proc_modifier(input: Node) -> PestResult<FuncProcModifier> {
        Ok(match_nodes!(input.into_children();
          [export(_)] => FuncProcModifier::Export,
          [entry(_)] => FuncProcModifier::Entry,
          [version(v)] => FuncProcModifier::Version(v),
        ))
    }

    fn boolean(input: Node) -> PestResult<bool> {
        Ok(match input.as_str() {
            "false" => false,
            "true" => true,
            _ => unreachable!()
        })
    }

    fn number(input: Node) -> PestResult<f64> {
        input.as_str().parse::<f64>().map_err(|e| input.error(e))
    }

    fn opcode_opcode(input: Node) -> PestResult<usize> {
        Ok(input.as_str().parse::<usize>().unwrap())
    }

    fn finite_seq(input: Node) -> PestResult<PTExpression> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::expression(child)?);
        }
        Ok(PTExpression::FiniteSequence(out))
    }

    fn infinite_seq(input: Node) -> PestResult<PTExpression> {
        Ok(match_nodes!(input.into_children();
            [expression(s)] => PTExpression::InfiniteSequence(Box::new(s))
        ))
    }

    fn prog_version(input: Node) -> PestResult<u32> {
        input.as_str().parse::<u32>().map_err(|e| input.error(e))
    }

    fn header(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
          [string(group),string(name),prog_version(v)] => {
            PTStatementValue::Header(group,name,v)
          }
        ))
    }

    fn include(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [string(s)] => s ))
    }

    fn fixed_include(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [string(s)] => s ))
    }

    fn bundle(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [prefix(s)] => s ))
    }

    fn repeater(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [prefix(s)] => s ))
    }

    fn constant(input: Node) -> PestResult<Constant> {
        Ok(match_nodes!(input.into_children();
          [string(s)] => Constant::String(s),
          [number(n)] => Constant::Number(OrderedFloat(n)),
          [boolean(b)] => Constant::Boolean(b)
        ))
    }

    fn identifier(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }
    fn macro_identifier(input: Node) -> PestResult<String> { 
        let v = input.as_str();
        Ok(v[0..v.len()-1].to_string())
    }
    fn prefix(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

    fn register(input: Node) -> PestResult<usize> {
        input.as_str()[1..].trim().parse::<usize>().map_err(|e| input.error(e))
    }

    fn arg_atomic_type(input: Node) -> PestResult<AtomicTypeSpec> {
        match input.as_str() {
            "string" => Ok(AtomicTypeSpec::String),
            "number" => Ok(AtomicTypeSpec::Number),
            "boolean" => Ok(AtomicTypeSpec::Boolean),
            _ => {
                let inner = input.into_children().next().unwrap();
                Ok(match inner.as_rule() {
                    Rule::identifier => AtomicTypeSpec::Handle(inner.as_str().to_string()),
                    Rule::arg_wild_type => AtomicTypeSpec::Handle(inner.as_str().to_string()),
                    _ => unreachable!()
                })
            }
        }
    }

    fn arg_seq_type(input: Node) -> PestResult<AtomicTypeSpec> {
        Self::arg_atomic_type(input.into_children().next().unwrap())
    }

    fn arg_type(input: Node) -> PestResult<TypeSpec> {
        Ok(match_nodes!(input.into_children();
          [arg_atomic_type(a)] => TypeSpec::Atomic(a),
          [arg_seq_type(s)] => TypeSpec::Sequence(s),
          [arg_atom_wild_type(s)] => TypeSpec::AtomWildcard(s),
          [arg_seq_wild_type(s)] => TypeSpec::SequenceWildcard(s),
          [arg_wild_type(s)] => TypeSpec::Wildcard(s)
        ))
    }

    fn arg_types(input: Node) -> PestResult<Vec<TypeSpec>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::arg_type(child)?);
        }
        Ok(out)
    }

    fn arg_check_variable(input: Node) -> PestResult<String> { 
        Ok(input.as_str().trim().to_string())
    }

    fn arg_wild_type(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
            [arg_check_variable(x)] => x
        ))
    }

    fn arg_seq_wild_type(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
            [arg_wild_type(x)] => x
        ))
    }

    fn arg_atom_wild_type(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
            [arg_wild_type(x)] => x
        ))
    }

    fn argument(input: Node) -> PestResult<OrBundleRepeater<PTExpression>> {
        Ok(match_nodes!(input.into_children();
            [expression(x)] => OrBundleRepeater::Normal(x),
            [bundle(x)] => OrBundleRepeater::Bundle(x),
            [repeater(x)] => OrBundleRepeater::Repeater(x)
        ))
    }

    fn arguments(input: Node) -> PestResult<Vec<OrBundleRepeater<PTExpression>>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::argument(child)?);
        }
        Ok(out)
    }

    fn func_or_proc_call(input: Node) -> PestResult<PTCall> {
        Ok(match_nodes!(input.into_children();
            [identifier(name),argument(args)..] => PTCall { name, args: args.collect(), is_macro: false },
            [identifier(name)] => PTCall { name, args: vec![], is_macro: false }
        ))
    }

    fn capture(input: Node) -> PestResult<OrBundle<Variable>> {
        Ok(match_nodes!(input.into_children();
            [variable(s)] => OrBundle::Normal(s),
            [bundle(b)] => OrBundle::Bundle(b)
        ))
    }

    fn capture_decl(input: Node) -> PestResult<Vec<OrBundle<Variable>>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::capture(child)?);
        }
        Ok(out)
    }

    fn capture_decls(input: Node) -> PestResult<Vec<OrBundle<Variable>>> {
        Ok(match_nodes!(input.into_children();
            [capture_decl(t)..] => t.flatten().collect::<Vec<OrBundle<Variable>>>()
        ))
    }

    fn macro_call(input: Node) -> PestResult<PTCall> {
        Ok(match_nodes!(input.into_children();
            [macro_identifier(name),argument(args)..] => PTCall { name, args: args.collect(), is_macro: true },
            [macro_identifier(name)] => PTCall { name, args: vec![], is_macro: true }
        ))
    }

    fn arg_check(input: Node) -> PestResult<Check> {
        Ok(match_nodes!(input.into_children();
          [arg_length_check(c)] => c,
          [arg_lengthorinf_check(c)] => c,
          [arg_total_check(c)] => c,
          [arg_ref_check(c)] => c
        ))
    }

    fn check_annotation(input: Node) -> PestResult<Check> {
        Ok(match_nodes!(input.into_children();
          [length_check(c)] => c,
          [lengthorinf_check(c)] => c,
          [total_check(c)] => c,
          [ref_check(c)] => c
        ))
    }

    fn length_check(input: Node) -> PestResult<Check> { decl_check(input,CheckType::Length) }
    fn lengthorinf_check(input: Node) -> PestResult<Check> { decl_check(input,CheckType::LengthOrInfinite) }
    fn total_check(input: Node) -> PestResult<Check> { decl_check(input,CheckType::Sum) }
    fn ref_check(input: Node) -> PestResult<Check> { decl_check(input,CheckType::Reference) }
    fn arg_length_check(input: Node) -> PestResult<Check> { check(input,CheckType::Length) }
    fn arg_lengthorinf_check(input: Node) -> PestResult<Check> { check(input,CheckType::LengthOrInfinite) }
    fn arg_total_check(input: Node) -> PestResult<Check> { check(input,CheckType::Sum) }
    fn arg_ref_check(input: Node) -> PestResult<Check> { check(input,CheckType::Reference) }

    fn let_decl(input: Node) -> PestResult<OrBundle<(Variable,Vec<Check>)>> {
        Ok(match_nodes!(input.into_children();
            [variable(v),check_annotation(c)..] => OrBundle::Normal((v,c.collect())),
            [bundle(b)] => OrBundle::Bundle(b)
        ))
    }

    fn let_decls(input: Node) -> PestResult<Vec<OrBundleRepeater<(Variable,Vec<Check>)>>> {
        Ok(match_nodes!(input.into_children();
            [let_decl(v)..] => {
                v.map(|x| {
                    match x {
                        OrBundle::Normal((v,c)) => OrBundleRepeater::Normal((v,c)),
                        OrBundle::Bundle(b) => OrBundleRepeater::Bundle(b)
                    }
                }).collect()
            },
            [repeater(v)] =>
                vec![OrBundleRepeater::Repeater(v)]
        ))
    }

    fn rhs_tuple(input: Node) -> PestResult<Vec<PTExpression>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::expression(child)?);
        }
        Ok(out)
    }

    fn let_rhs_tuple(input: Node) -> PestResult<Vec<OrBundle<PTExpression>>> {
        Ok(match_nodes!(input.into_children();
            [expr_or_bundle(b)..] => b.collect()
        ))
    }

    fn let_statement(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
            [let_decls(d),let_rhs_tuple(t)] =>
                PTStatementValue::LetStatement(d,t),
        ))
    }

    fn let_repeater_statement(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
            [repeater(a),repeater(b)] =>
                PTStatementValue::LetRepeaterStatement(a,b),
        ))
    }

    fn modify_statement(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
          [variable(v)..,rhs_tuple(t)] => {
              PTStatementValue::ModifyStatement(v.collect(),t)
          }
        ))
    }

    fn code_modifiers(input: Node) -> PestResult<Vec<CodeModifier>> {
        let mut out = vec![];
        for child in input.into_children() {
            if let Some(child) = Self::code_modifier(child)? {
                out.push(child);
            }
        }
        Ok(out)
    }

    fn func_modifiers(input: Node) -> PestResult<Vec<FuncProcModifier>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::func_modifier(child)?);
        }
        Ok(out)
    }

    fn proc_modifiers(input: Node) -> PestResult<Vec<FuncProcModifier>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::proc_modifier(child)?);
        }
        Ok(out)
    }

    fn opcode_args(input: Node) -> PestResult<Vec<usize>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::register(child)?);
        }
        Ok(out)
    }

    fn opcode_statement(input: Node) -> PestResult<Opcode> {
        Ok(match_nodes!(input.into_children();
            [opcode_opcode(code),opcode_args(args)] => {
                Opcode(code,args)
            }
        ))
    }

    fn variable(input: Node) -> PestResult<Variable> {
        Ok(match_nodes!(input.into_children();
            [prefix(p),identifier(id)] => {
                Variable {
                    prefix: Some(p),
                    name: id
                }
            },
            [identifier(id)] => {
                Variable {
                    prefix: None,
                    name: id
                }
            }
        ))
    }

    fn code_arg(input: Node) -> PestResult<CodeArgument> {
        Ok(match_nodes!(input.into_children();
            [code_arg_modifiers(m),arg_type(t),arg_check(c)..] => {
                CodeArgument {
                    arg_type: t,
                    modifiers: m,
                    checks: c.collect()
                }
            }
        ))
    }

    fn code_arguments(input: Node) -> PestResult<Vec<CodeArgument>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::code_arg(child)?);
        }
        Ok(out)
    }

    fn code_return(input: Node) -> PestResult<Vec<CodeArgument>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::code_arg(child)?);
        }
        Ok(out)
    }

    fn code_header(input: Node) -> PestResult<CodeBlock> {
        Ok(match_nodes!(input.into_children();
            [code_modifiers(m),identifier(id),code_arguments(args),code_return(r)] => {
                CodeBlock { 
                    name: id,
                    arguments: args,
                    results: r,
                    impls: vec![],
                    modifiers: m
                }
            }
        ))
    }

    fn code_arg_modifier(input: Node) -> PestResult<CodeArgModifier> {
        Ok(match input.as_str() {
            "large" => CodeArgModifier::Large,
            "sparse" => CodeArgModifier::Sparse,
            _ => { panic!("unexpected code_arg_modifier"); }
        })
    }

    fn code_arg_modifiers(input: Node) -> PestResult<Vec<CodeArgModifier>> {
        Ok(match_nodes!(input.into_children();
            [code_arg_modifier(m)..] => {
                m.collect()
            }
        ))
    }

    fn impl_real_arg(input: Node) -> PestResult<CodeImplVariable> {
        Ok(match_nodes!(input.into_children();
            [register(r),arg_type(t)] => {
                CodeImplVariable {
                    reg_id: r,
                    arg_type: t
                }
            }
        ))
    }

    fn impl_arg_var(input: Node) -> PestResult<CodeImplArgument> {
        Ok(match_nodes!(input.into_children();
            [impl_real_arg(v)] => CodeImplArgument::Register(v),
            [constant(c)] => CodeImplArgument::Constant(c)
        ))
    }

    fn impl_arguments(input: Node) -> PestResult<Vec<CodeImplArgument>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::impl_arg_var(child)?);
        }
        Ok(out)
    }

    fn impl_return_var(input: Node) -> PestResult<CodeReturn> {
        Ok(match_nodes!(input.into_children();
            [impl_real_arg(v)] => CodeReturn::Register(v),
            [register(c)] => CodeReturn::Repeat(c),
            [constant(c)] => CodeReturn::Constant(c)
        ))
    }

    fn impl_return(input: Node) -> PestResult<Vec<CodeReturn>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::impl_return_var(child)?);
        }
        Ok(out)
    }

    fn impl_header(input: Node) -> PestResult<ImplBlock> {
        Ok(match_nodes!(input.into_children();
            [impl_arguments(args),impl_return(rets)] => {
                ImplBlock {
                    arguments: args, 
                    results: rets, 
                    command: None
                }
            }
        ))
    }

    fn code_impl(input: Node) -> PestResult<ImplBlock> {
        Ok(match_nodes!(input.into_children();
            [impl_header(mut block),opcode_statement(stmt)] => {
                block.command = Some(stmt);
                block
            },
            [impl_header(block)] => {
                block
            },
        ))
    }

    fn code_block(input: Node) -> PestResult<CodeBlock> {
        let input2 = input.clone();
        Ok(match_nodes!(input.into_children();
            [code_header(mut block),code_impl(stmt)..] => {
                block.add_impls(stmt.collect()).map_err(|e| input2.error(e))?;
                block
            }
        ))
    }

    fn inner_block(input: Node) -> PestResult<PTStatement> {
        let line_no = input.as_span().start_pos().line_col().0;
        let position = input.user_data().position.at_line(line_no as u32);
        let context = input.user_data().context;
        let value = match_nodes!(input.into_children();
            [let_statement(s)] => s,
            [let_repeater_statement(s)] => s,
            [modify_statement(s)] => s,
            [expression(c)] => PTStatementValue::Expression(c),
            [macro_call(c)] => PTStatementValue::MacroCall(c)
        );
        Ok(PTStatement { value, position, context })
    }

    fn version_part(input: Node) -> PestResult<u32> {
        Ok(input.as_str().parse::<u32>().unwrap())
    }

    fn version_num(input: Node) -> PestResult<(u32,u32)> {
        Ok(match_nodes!(input.into_children();
            [version_part(a),version_part(b)] => (a,b)
        ))
    }

    fn lib_version(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
            [string(name),version_num(p)] => PTStatementValue::Version(name,p.0,p.1)
        ))
    }

    fn block(input: Node) -> PestResult<PTStatement> {
        let context = input.user_data().context;
        let line_no = input.as_span().start_pos().line_col().0;
        let position = input.user_data().position.at_line(line_no as u32);
        let value = match_nodes!(input.into_children();
            [header(v)] => v,
            [lib_version(v)] => v,
            [code_block(block)] => PTStatementValue::Code(block),
            [include(s)] => PTStatementValue::Include(s,false),
            [fixed_include(s)] => PTStatementValue::Include(s,true),
            [function(f)] => PTStatementValue::FuncDef(f),
            [procedure(p)] => PTStatementValue::ProcDef(p),
            [inner_block(b)] => { return Ok(b); },
        );
        Ok(PTStatement { value, position, context })
    }

    fn funcproc_arg_extras(input: Node) -> PestResult<ArgTypeSpec> {
        Ok(match_nodes!(input.into_children();
            [arg_types(block),arg_check(checks)..] => ArgTypeSpec {
                arg_types: block,
                checks: checks.collect()
            }
        ))
    }


    fn funcproc_arg_named(input: Node) -> PestResult<TypedArgument> {
        Ok(match_nodes!(input.into_children();
            [identifier(id),funcproc_arg_extras(typespec)] => {
                TypedArgument { id, typespec }
            },
            [identifier(id)] => {
                TypedArgument { id, typespec: ArgTypeSpec { arg_types: vec![], checks: vec![] } }
            },
        ))
    }

    fn funcproc_arg(input: Node) -> PestResult<OrBundle<TypedArgument>> {
        Ok(match_nodes!(input.into_children();
            [funcproc_arg_named(arg)] => OrBundle::Normal(arg),
            [bundle(prefix)] => OrBundle::Bundle(prefix)
        ))
    }

    fn funcproc_args(input: Node) -> PestResult<Vec<OrBundle<TypedArgument>>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::funcproc_arg(child)?);
        }
        Ok(out)
    }

    fn funcproc_return_type(input: Node) -> PestResult<ArgTypeSpec> {
        Ok(match_nodes!(input.into_children();
            [arg_types(t),arg_check(c)..] => {
                ArgTypeSpec {
                    arg_types: t,
                    checks: c.collect()
                }
            }
        ))
    }

    fn function_return(input: Node) -> PestResult<Option<ArgTypeSpec>> {
        Ok(match_nodes!(input.into_children();
            [funcproc_return_type(t)] => Some(t),
            [] => None
        ))
    }

    fn function_value(input: Node) -> PestResult<OrBundle<PTExpression>> {
        Ok(match_nodes!(input.into_children();
            [expression(x)] => OrBundle::Normal(x),
            [bundle(x)] => OrBundle::Bundle(x)
        ))
    }

    fn function(input: Node) -> PestResult<PTFuncDef> {
        Ok(match_nodes!(input.into_children();
            [func_modifiers(f),identifier(id),funcproc_args(c),function_return(r),capture_decls(p),inner_block(b)..,function_value(x)] =>
                PTFuncDef {
                    name: id,
                    args: c,
                    captures: p,
                    block: b.collect(),
                    value: x,
                    value_type: r,
                    modifiers: f
                }
        ))
    }

    fn procedure_return(input: Node) -> PestResult<Vec<ArgTypeSpec>> {
        Ok(match_nodes!(input.into_children();
            [funcproc_return_type(t)..] => t.collect()
        ))
    }

    fn procedure_return_option(input: Node) -> PestResult<Option<Vec<ArgTypeSpec>>> {
        Ok(match_nodes!(input.into_children();
            [procedure_return(t)] => Some(t),
            [] => None
        ))
    }

    fn procedure_expression(input: Node) -> PestResult<Vec<OrBundle<PTExpression>>> {
        Ok(match_nodes!(input.into_children();
            [expr_or_bundle(x)..] => x.collect(),
        ))
    }

    fn procedure(input: Node) -> PestResult<PTProcDef> {
        Ok(match_nodes!(input.into_children();
            [proc_modifiers(f),identifier(id),funcproc_args(args),procedure_return_option(ret_args),capture_decls(p),inner_block(b)..,procedure_expression(r)] => {
                PTProcDef {
                    name: id,
                    args,
                    captures: p,
                    block: b.collect(),
                    ret: r,
                    ret_type: ret_args,
                    modifiers: f
                }
            }
        ))
    }

    fn prefix_operators(input: Node) -> PestResult<String> { Ok(input.as_str().trim().to_string()) }

    fn expr1(input: Node) -> PestResult<PTExpression> {
        left_rec!(simple_expression,infix1,input)
    }

    fn expr2(input: Node) -> PestResult<PTExpression> {
        left_rec!(expr1,infix2,input)
    }

    fn expr3(input: Node) -> PestResult<PTExpression> {
        left_rec!(expr2,infix3,input)
    }

    fn expr4(input: Node) -> PestResult<PTExpression> {
        left_rec!(expr3,infix4,input)
    }

    fn expr5(input: Node) -> PestResult<PTExpression> {
        left_rec!(expr4,infix5,input)
    }

    fn expr6(input: Node) -> PestResult<PTExpression> {
        left_rec!(expr5,infix6,input)
    }

    fn simple_expression(input: Node) -> PestResult<PTExpression> {
        Ok(match_nodes!(input.into_children();
            [constant(c)] => PTExpression::Constant(c),
            [variable(v)] => PTExpression::Variable(v),
            [prefix_operators(p),simple_expression(x)] => PTExpression::Prefix(p,Box::new(x)),
            [func_or_proc_call(c)] => PTExpression::Call(c),
            [macro_call(c)] => PTExpression::Call(c),
            [finite_seq(x)] => x,
            [infinite_seq(x)] => x,
            [expression(x)] => x,
        ))
    }

    fn expression(input: Node) -> PestResult<PTExpression> {
        Ok(match_nodes!(input.into_children();
            [expr6(x)] => x
        ))
    }

    fn expr_or_bundle(input: Node) -> PestResult<OrBundle<PTExpression>> {
        Ok(match_nodes!(input.into_children();
            [expression(x)] => OrBundle::Normal(x),
            [bundle(b)] => OrBundle::Bundle(b)
        ))
    }

    fn file(input: Node) -> PestResult<Vec<PTStatement>> {
        let mut out = vec![];
        for child in input.into_children() {
            match child.as_rule() {
                Rule::block => {
                    out.push(Self::block(child)?);
                },
                _ => {}
            }
        }
        Ok(out)
    }
}

fn do_parse_eard(input: &str, position: &ParsePosition, optimise: bool, context: usize) -> PestResult<Vec<PTStatement>> {
    let state = ParseFixedState { 
        position: position.clone(),
        optimise,
        context
    };
    let input = EardParser::parse_with_userdata(Rule::file,input,state)?.single()?;
    EardParser::file(input)
}

pub(crate) fn parse_eard(position: &ParsePosition, filename: &str, fixed: bool, optimise: bool, context: usize) -> Result<Vec<PTStatement>,String> {
    let (input,new_pos) = position.push(filename,fixed)?;
    do_parse_eard(&input,&new_pos,optimise,context).map_err(|e| e.to_string())
}
