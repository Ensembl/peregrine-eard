use std::sync::Arc;
use pest_consume::{Parser, Error, match_nodes};
use crate::{parsetree::{ PTCodeRegisterArgument, PTCodeArgument, PTCodeBlock, PTCodeCommand, PTExpression, PTCall, PTFuncDef, PTProcDef, PTStatement, PTStatementValue, PTLetAssign, PTTypedArgument }, compiler::EarpCompiler, model::{CodeModifier, Variable, Check, CheckType, FuncProcModifier, CallArg, Constant, OrBundle, AtomicTypeSpec, TypeSpec, ArgTypeSpec}};

#[derive(Parser)]
#[grammar = "earp.pest"]
struct EarpParser;

#[derive(Clone)]
struct ParseFixedState {
    filename: Arc<Vec<String>>,
    context: usize
}

#[allow(unused)]
type PestResult<T> = std::result::Result<T, Error<Rule>>;
#[allow(unused)]
type Node<'i> = pest_consume::Node<'i, Rule, ParseFixedState>;

fn parse_string(input: Node) -> PestResult<String> {
    /* Same as JSON's definition plus \0 */
    let mut out = String::new();
    let mut u_left = 0;
    let mut u_val = 0;
    let mut seen_bs = false;
    for c in input.as_str().chars() {
        if seen_bs {
            if u_left > 0 {
                u_left -= 1;
                u_val = u_val*16 + c.to_digit(16).unwrap();
                seen_bs = u_left != 0;
                if !seen_bs { 
                    out.push(char::from_u32(u_val).ok_or_else(|| input.error("bad unicode"))?);
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
    Ok(Check { check_type, name: input.into_children().next().unwrap().as_str().to_string() })
}

#[pest_consume::parser]
impl EarpParser {
    fn string_inner(input: Node) -> PestResult<String> {
        parse_string(input)
    }

    fn multi_string_inner(input: Node) -> PestResult<String> {
        parse_string(input)
    }

    fn string(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children();
          [string_inner(s)] => s,
          [multi_string_inner(s)] => s,
        ))
    }

    fn code_modifier(input: Node) -> PestResult<CodeModifier> {
        Ok(match input.as_str() {
            "world" => CodeModifier::World,
            _ => unreachable!()
        })
    }

    fn funcproc_modifier(input: Node) -> PestResult<FuncProcModifier> {
        Ok(match input.as_str() {
            "export" => FuncProcModifier::Export,
            _ => unreachable!()
        })
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

    fn include(input: Node) -> PestResult<String> {
        Ok(match_nodes!(input.into_children(); [string(s)] => s ))
    }

    fn flag_statement(input: Node) -> PestResult<String> {
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
          [number(n)] => Constant::Number(n),
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
        input.as_str()[1..].parse::<usize>().map_err(|e| input.error(e))
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

    fn arg_check_variable(input: Node) -> PestResult<String> { Ok(input.as_str().to_string()) }

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

    fn argument(input: Node) -> PestResult<CallArg<PTExpression>> {
        Ok(match_nodes!(input.into_children();
            [expression(x)] => CallArg::Expression(x),
            [bundle(x)] => CallArg::Bundle(x),
            [repeater(x)] => CallArg::Repeater(x)
        ))
    }

    fn arguments(input: Node) -> PestResult<Vec<CallArg<PTExpression>>> {
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

    fn length_check(input: Node) -> PestResult<Check> { check(input,CheckType::Length) }
    fn lengthorinf_check(input: Node) -> PestResult<Check> { check(input,CheckType::LengthOrInfinite) }
    fn total_check(input: Node) -> PestResult<Check> { check(input,CheckType::Sum) }
    fn ref_check(input: Node) -> PestResult<Check> { check(input,CheckType::Reference) }
    fn arg_length_check(input: Node) -> PestResult<Check> { check(input,CheckType::Length) }
    fn arg_lengthorinf_check(input: Node) -> PestResult<Check> { check(input,CheckType::LengthOrInfinite) }
    fn arg_total_check(input: Node) -> PestResult<Check> { check(input,CheckType::Sum) }
    fn arg_ref_check(input: Node) -> PestResult<Check> { check(input,CheckType::Reference) }

    fn let_decl(input: Node) -> PestResult<(Variable,Vec<Check>)> {
        Ok(match_nodes!(input.into_children();
            [variable(v),check_annotation(c)..] =>
                (v,c.collect())
        ))
    }

    fn let_decls(input: Node) -> PestResult<Vec<PTLetAssign>> {
        Ok(match_nodes!(input.into_children();
            [let_decl(v)..] =>
                v.map(|(v,c)| PTLetAssign::Variable(v,c)).collect(),
            [repeater(v)] =>
                vec![PTLetAssign::Repeater(v)]
        ))
    }

    fn rhs_tuple(input: Node) -> PestResult<Vec<PTExpression>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::expression(child)?);
        }
        Ok(out)
    }

    fn let_statement(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
            [let_decls(d),rhs_tuple(t)] =>
                PTStatementValue::LetStatement(d,t),
        ))
    }

    fn modify_statement(input: Node) -> PestResult<PTStatementValue> {
        Ok(match_nodes!(input.into_children();
          [variable(v)..,rhs_tuple(t)] => {
              PTStatementValue::ModifyStatement(v.collect(),t)
          }
        ))
    }

    fn code_variable(input: Node) -> PestResult<PTCodeRegisterArgument> {
        Ok(match_nodes!(input.into_children();
            [register(r),arg_types(t),arg_check(c)..] => {
                PTCodeRegisterArgument {
                    reg_id: r,
                    arg_types: t,
                    checks: c.collect()
                }
            }
        ))
    }

    fn code_argument(input: Node) -> PestResult<PTCodeArgument> {
        Ok(match_nodes!(input.into_children();
            [code_variable(v)] => PTCodeArgument::Register(v),
            [constant(c)] => PTCodeArgument::Constant(c)
        ))
    }

    fn code_arguments(input: Node) -> PestResult<Vec<PTCodeArgument>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::code_argument(child)?);
        }
        Ok(out)
    }

    fn code_modifiers(input: Node) -> PestResult<Vec<CodeModifier>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::code_modifier(child)?);
        }
        Ok(out)
    }

    fn funcproc_modifiers(input: Node) -> PestResult<Vec<FuncProcModifier>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::funcproc_modifier(child)?);
        }
        Ok(out)
    }

    fn code_return(input: Node) -> PestResult<Vec<PTCodeRegisterArgument>> {
        let mut out = vec![];
        for child in input.into_children() {
            out.push(Self::code_variable(child)?);
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

    fn opcode_statement(input: Node) -> PestResult<PTCodeCommand> {
        Ok(match_nodes!(input.into_children();
            [opcode_opcode(code),opcode_args(args)] => {
                PTCodeCommand::Opcode(code,args)
            }
        ))
    }

    fn code_statement(input: Node) -> PestResult<PTCodeCommand> {
        Ok(match_nodes!(input.into_children();
            [opcode_statement(stmt)] => stmt,
            [register_statement(reg)] => PTCodeCommand::Register(reg)
        ))
    }

    fn register_statement(input: Node) -> PestResult<usize> {
        Ok(match_nodes!(input.into_children();
            [register(r)] => r
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

    fn code_header(input: Node) -> PestResult<PTCodeBlock> {
        Ok(match_nodes!(input.into_children();
            [code_modifiers(m),identifier(id),code_arguments(args),code_return(r)] => {
                PTCodeBlock { 
                    name: id,
                    arguments: args,
                    results: r,
                    commands: vec![],
                    modifiers: m
                }
            }
        ))
    }

    fn code_block(input: Node) -> PestResult<PTCodeBlock> {
        Ok(match_nodes!(input.into_children();
            [code_header(mut block),code_statement(stmt)..] => {
                block.commands = stmt.collect();
                block
            }
        ))
    }

    fn inner_block(input: Node) -> PestResult<PTStatement> {
        let filename = input.user_data().filename.clone();
        let line_no = input.as_span().start_pos().line_col().0;
        let context = input.user_data().context;
        let value = match_nodes!(input.into_children();
            [let_statement(s)] => s,
            [modify_statement(s)] => s,
            [func_or_proc_call(c)] => PTStatementValue::BareCall(c),
            [macro_call(c)] => PTStatementValue::BareCall(c)
        );
        Ok(PTStatement {
            value,
            file: filename,
            line_no,
            context
        })
    }

    fn block(input: Node) -> PestResult<PTStatement> {
        let filename = input.user_data().filename.clone();
        let context = input.user_data().context;
        let line_no = input.as_span().start_pos().line_col().0;
        let value = match_nodes!(input.into_children();
            [code_block(block)] => PTStatementValue::Code(block),
            [include(s)] => PTStatementValue::Include(s),
            [flag_statement(s)] => PTStatementValue::Flag(s),
            [function(f)] => PTStatementValue::FuncDef(f),
            [procedure(p)] => PTStatementValue::ProcDef(p),
            [inner_block(b)] => { return Ok(b); },
        );
        Ok(PTStatement {
            value,
            file: filename,
            line_no,
            context
        })
    }

    fn funcproc_arg_extras(input: Node) -> PestResult<ArgTypeSpec> {
        Ok(match_nodes!(input.into_children();
            [arg_types(block),arg_check(checks)..] => ArgTypeSpec {
                arg_types: block,
                checks: checks.collect()
            }
        ))
    }


    fn funcproc_arg_named(input: Node) -> PestResult<PTTypedArgument> {
        Ok(match_nodes!(input.into_children();
            [identifier(id),funcproc_arg_extras(typespec)] => {
                PTTypedArgument { id, typespec }
            },
            [identifier(id)] => {
                PTTypedArgument { id, typespec: ArgTypeSpec { arg_types: vec![], checks: vec![] } }
            },
        ))
    }

    fn funcproc_arg(input: Node) -> PestResult<OrBundle<PTTypedArgument>> {
        Ok(match_nodes!(input.into_children();
            [funcproc_arg_named(arg)] => OrBundle::Normal(arg),
            [bundle(prefix)] => OrBundle::Bundle(prefix)
        ))
    }

    fn funcproc_args(input: Node) -> PestResult<Vec<OrBundle<PTTypedArgument>>> {
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
            [funcproc_modifiers(f),identifier(id),funcproc_args(c),function_return(r),inner_block(b)..,function_value(x)] =>
                PTFuncDef {
                    name: id,
                    args: c,
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
            [expression(x)..] => x.map(|x| OrBundle::Normal(x)).collect(),
            [bundle(b)] => vec![OrBundle::Bundle(b)]
        ))
    }

    fn procedure(input: Node) -> PestResult<PTProcDef> {
        Ok(match_nodes!(input.into_children();
            [funcproc_modifiers(f),identifier(id),funcproc_args(args),procedure_return_option(ret_args),inner_block(b)..,procedure_expression(r)] => {
                PTProcDef {
                    name: id,
                    args,
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

fn do_parse_earp(input: &str, filename: &[String], context: usize) -> PestResult<Vec<PTStatement>> {
    let state = ParseFixedState { 
        filename: Arc::new(filename.to_vec()),
        context
    };
    let input = EarpParser::parse_with_userdata(Rule::file,input,state)?.single()?;
    EarpParser::file(input)
}

pub fn parse_earp(compiler: &EarpCompiler, filename: &[String], context: usize) -> Result<Vec<PTStatement>,String> {
    let input = compiler.load_source(filename.last().unwrap())?; // XXX
    do_parse_earp(&input,filename,context).map_err(|e| e.to_string())
}
