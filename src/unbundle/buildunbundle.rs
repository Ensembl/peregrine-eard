/* By this stage have a plausible expression tree for each statement except that it has embedded
 * bundles and repeaters. This algorithm identifies which actual variables are used in each passing
 * of a bundle across the argument and return boundaries for procedures and functions, known as
 * bundle transits. It does this by going through the program in reverse, first captuing usage and 
 * then, when a bundle transit is found, recording that usage. The actual unbundling is done
 * elsewhere.
 * 
 * We transit the program in reverse, first capturing usages and associating them with bundle
 * transits going back through the code from the end to the start, from result to argument.
 * 
 * The locations of transits are identified by a tuple of a list of call indexes and an enum
 * identifying argument/return and its index, eg ([7,6],Arg(3)) or ([10],Return(2)).
 * 
 * Prefix variable usage for a bundle is recorded in a hashset. These are assembled into a stack
 * for each call context.
 * 
 * One non-trivial task is working out _where_ bundle transits take place as they can be implicit, 
 * ie not locally indicated by the syntax. (Repeaters are easy as they are always explicit and only
 * at the statement level).
 * 
 * 1. There is a bundle transit where a function or procedure accepts a bundle argument with an 
 * explicit "*" in the argument list. 
 * 
 * 2. There is a bundle transit when a function or procedure returns a bundle result with an
 * explicit "*" in the return list.
 * 
 * 3. There is a bundle transit out of a function or procedure when its return epxression
 * contains a function which returns a bundle. These are the *implicit* bundle returns which are
 * not directly indicated by syntax.
 *
 * An implicit bundle return in b(). Note that we cannot determine that b returns a bundle by its
 * own syntax: we only know it by examining its return expression.
 *  
 * function a() { let a.a = 1; *a }
 * function b() { a() }
 *
 * Due to implicit returns, we delegate whether or not a function accepts or returns a bundle in
 * the n-th position to the function definition, which is determined recursively.
 * 
 * Algorithm:
 *   For each statment in reverse order:
 *     1. If it is a repeater statement, copy from the lhs repeater to the rhs repeater.
 *     2. Clear any bundles used in the lvalue.
 *     3. For each return argument to the call:
 *        a. if callee confirms it is a bundle return, add a transit from the caller's bundle
 *           to a new bundle for the callee. If the caller discards, use an empty, anonymous bundle.
 *        b. in the callee, iterate through the return expression, recursing all of step 3 for
 *           functions and recording bundle.names as used.
 *        c. iterate through the statements, recursing all of step 1.
 *        d. iterate through the arguments, for each which is a bundle add an argument transit
 *           into the caller. If the expression for the argument in the caller is a function,
 *           recurse step 3 and recording bundle.names as used.
 */

use std::{sync::Arc, collections::{HashSet, HashMap}};
use crate::{buildtree::{BTStatement, BTStatementValue, BTLValue, BTProcCall, BTExpression, BTRegisterType, BuildTree, BTFuncCall}, model::{OrBundleRepeater, Variable, OrBundle, TypedArgument}, parsetree::at, unbundle::unbundleaux::{BundleNamespace, Transits, Position}};
use super::repeater::find_repeater_arguments;

// TODO global bundles

fn sorted<T: Clone+Ord>(set: Option<&HashSet<T>>) -> Option<Vec<T>> {
    set.as_ref().map(|set| {
        let mut out : Vec<T> = set.iter().cloned().collect();
        out.sort();
        out
    })
}

fn all_sorted<T: Clone+Ord>(list: &[Option<HashSet<T>>]) -> Vec<Option<Vec<T>>> {
    list.iter().map(|set| {
        sorted(set.as_ref())
    }).collect()
}

fn non_bundle(caller_expects: Option<&HashSet<String>>) -> Result<(),String> {
    if caller_expects.is_some() { Err("unexpected bundle".to_string()) } else { Ok(( ))}
}

fn bundle(caller_expects: Option<&HashSet<String>>) -> Result<&HashSet<String>,String> {
    if let Some(x) = caller_expects { Ok(x) } else { Err("expected bundle".to_string())}
}

struct BuildUnbundle<'a> {
    trace: Vec<String>,
    tree: &'a BuildTree,
    positions: Vec<(Arc<Vec<String>>,usize)>,
    namespace: BundleNamespace,
    register_bundles: HashMap<usize,HashSet<String>>,
    transits: Transits
}

impl<'a> BuildUnbundle<'a> {
    fn new(tree: &'a BuildTree) -> BuildUnbundle<'a> {
        BuildUnbundle {
            trace: vec![],
            tree,
            positions: vec![],
            namespace: BundleNamespace::new(),
            register_bundles: HashMap::new(),
            transits: Transits::new()
        }
    }

    #[cfg(test)]
    fn trace(&mut self, msg: &str) {
        self.trace.push(msg.to_string());
    }
    
    #[cfg(not(test))]
    fn trace(&mut self, _msg: &str) {}
    
    fn error_at(&self, msg: &str) -> String {
        self.positions.last().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
    }

    fn check_for_repeater(&mut self, stmt: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        if let Some((left,right)) = find_repeater_arguments(stmt)? {
            self.transits.push(stmt.call_index);
            let copied = self.namespace.get(&left).map(|x| x.get_used().clone()).unwrap_or_else(|| HashSet::new());
            self.transits.add(Position::Repeater,copied);
            self.transits.pop();
            self.namespace.merge(&right,&left);
            self.namespace.clear(&left);
        }
        Ok(())
    }

    fn declare(&mut self, var: &OrBundleRepeater<Variable>) -> Result<(),String> {
        self.trace(&format!("  declare = {:?}",var));
        match var {
            OrBundleRepeater::Normal(v) => {
                if let Some(prefix) = &v.prefix {
                    self.namespace.remove(prefix,&v.name);
                }
            },
            OrBundleRepeater::Bundle(prefix) => {
                self.namespace.clear(prefix);
            },
            OrBundleRepeater::Repeater(prefix) => {
                self.namespace.clear(prefix);
            }
        }
        Ok(())
    }

    fn clear_lvalue_bundles(&mut self, stmt: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        for ret in stmt.rets.as_ref().unwrap_or(&vec![]) {
            match ret {
                OrBundleRepeater::Normal(lvalue) => {
                    match lvalue {
                        BTLValue::Variable(v) => {
                            if let Some(prefix) = &v.prefix {
                                self.namespace.remove(prefix,&v.name);
                            }            
                        },
                        BTLValue::Register(_,BTRegisterType::Normal) => {},
                        BTLValue::Register(r,BTRegisterType::Bundle) => {
                            self.register_bundles.remove(r);
                        },
                    }
                },
                OrBundleRepeater::Bundle(prefix) => {
                    self.namespace.clear(prefix);                    
                },
                OrBundleRepeater::Repeater(prefix) => {
                    self.namespace.clear(prefix);
                }
            }
        }
        Ok(())
    }

    fn return_bundles(&mut self, stmt: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Vec<Option<HashSet<String>>> {
        let mut out = vec![];
        if let Some(rets) = &stmt.rets {
            for ret in rets {
                match ret {
                    OrBundleRepeater::Normal(value) => {
                        out.push(match value {
                            BTLValue::Register(r,BTRegisterType::Bundle) => {
                                Some(self.register_bundles.get(r).cloned().unwrap_or_else(|| HashSet::new()))
                            }
                            _ => None
                        });
                    },
                    OrBundleRepeater::Bundle(prefix) => {
                        if let Some(bundle) = self.namespace.get(&prefix) {
                            out.push(Some(bundle.get_used().clone()));
                        } else {
                            out.push(Some(HashSet::new()));
                        }
                    },
                    OrBundleRepeater::Repeater(_) => {
                        out.push(None);
                    }
                }
            }
        }
        out
    }

    fn arg_bundles(&self, args: &[OrBundle<TypedArgument>]) -> Result<Vec<Option<HashSet<String>>>,String> {
        Ok(args.iter().map(|arg| {
            match arg {
                OrBundle::Normal(_) => None,
                OrBundle::Bundle(prefix) => {
                    Some(self.namespace.get(&prefix).map(|x| x.get_used().clone()).unwrap_or_else(|| HashSet::new()))
                }
            }
        }).collect::<Vec<_>>())
    }

    fn expr_returns_bundle(&self, expr: &OrBundle<BTExpression>) -> Result<bool,String> {
        Ok(match expr {
            OrBundle::Normal(expr) => {
                match expr {
                    BTExpression::Constant(_) => false,
                    BTExpression::Variable(_) => false,
                    BTExpression::RegisterValue(_,BTRegisterType::Normal) => false,
                    BTExpression::RegisterValue(_,BTRegisterType::Bundle) => true,
                    BTExpression::Function(func) => {
                        let defn = self.tree.get_function(func)?;
                        self.expr_returns_bundle(&defn.ret[0])?
                    }
                }
            }
            OrBundle::Bundle(_) => true
        })
    }

    fn expr(&mut self, expr: &OrBundle<BTExpression>, caller_expects: Option<&HashSet<String>>) -> Result<(),String> {
        self.trace(&format!("  expr = {:?} expect = {:?}",expr,sorted(caller_expects)));
        match expr {
            OrBundle::Normal(expr) => {
                match expr {
                    BTExpression::Constant(_) => { non_bundle(caller_expects)?; },
                    BTExpression::Variable(v) => { 
                        non_bundle(caller_expects)?;
                        if let Some(prefix) = &v.prefix {
                            self.trace(&format!("      use of {:?}",v));
                            self.namespace.add(prefix,&v.name);
                        }
                    },
                    BTExpression::RegisterValue(_,BTRegisterType::Normal) => { non_bundle(caller_expects)?; },
                    BTExpression::RegisterValue(r,BTRegisterType::Bundle) => {
                        self.register_bundles.insert(*r,bundle(caller_expects)?.clone());
                    },
                    BTExpression::Function(func) => {
                        self.function(func, caller_expects)?;
                    }
                }
            },
            OrBundle::Bundle(prefix) => {
                let caller_expects = caller_expects.ok_or_else(|| "unexpected bundle".to_string())?;
                // TODO check: clear?
                self.namespace.add_all(&prefix,caller_expects);
            }
        }
        Ok(())
    }

    fn return_exprs(&mut self, exprs: &[OrBundle<BTExpression>], caller_bundles: &[Option<HashSet<String>>]) -> Result<(),String> {
        self.trace(&format!("  exprs = {:?}",exprs));
        let mut caller_bundles = caller_bundles.iter();
        for (i,ret) in exprs.iter().enumerate() {
            let caller_bundle = caller_bundles.next();
            if self.expr_returns_bundle(ret)? {
                let caller_expects = match caller_bundle {
                    Some(Some(x)) => x.clone(),
                    Some(None) => { return Err("return argument expected to be bundle".to_string()) },
                    None => HashSet::new()
                };
                self.transits.add(Position::Return(i),caller_expects.clone());
                self.expr(ret,Some(&caller_expects))?;
            } else {
                match caller_bundle {
                    Some(Some(_)) => { return Err("return argument expected to be non-bundle".to_string()) },
                    _ => {}
                }
                self.expr(ret,None)?;
            }
        }
        Ok(())
    }

    fn args(&mut self, expected: &[Option<HashSet<String>>], stmts: &[OrBundleRepeater<BTExpression>]) -> Result<(),String> {
        for (i,(expected,stmt_arg)) in expected.iter().zip(stmts.iter()).enumerate() {
            if let Some(expr) = stmt_arg.skip_repeater() {
                if let Some(expected) = &expected {
                    self.transits.add(Position::Arg(i),expected.clone());
                }
                self.expr(&expr,expected.as_ref())?;
            }
        }
        Ok(())
    }

    fn function(&mut self, func: &BTFuncCall, caller_expects: Option<&HashSet<String>>) -> Result<(),String> {
        self.trace(&format!("  function = {:?}",func));
        self.transits.push(func.call_index);
        self.trace("    push namespace");
        self.namespace.push();
        let defn = self.tree.get_function(func)?;
        /* Return expression */
        if self.expr_returns_bundle(&defn.ret[0])? {
            let bundle = bundle(caller_expects)?;
            self.expr(&defn.ret[0],Some(bundle))?;
            self.transits.add(Position::Return(0),bundle.clone());
        } else {
            self.expr(&defn.ret[0],None)?;
        }
        /* Process nested statements */
        for stmt in defn.block.iter().rev() {
            self.statement(stmt)?;
        }
        /* process arguments */
        let expected_args = self.arg_bundles(&defn.args)?;
        self.trace("    pop namespace");
        self.namespace.pop();
        self.args(&expected_args, &func.args)?;
        self.transits.pop();
        Ok(())
    }

    // TODO too many/few args
    fn procedure(&mut self, stmt: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        self.trace(&format!("  procedure = {:?}",stmt));
        self.check_for_repeater(stmt)?;
        /* Make list of caller returns to match in callee */
        let caller_return_bundles = self.return_bundles(stmt);
        self.clear_lvalue_bundles(stmt)?;
        self.trace(&format!("    ret bundles={:?}",all_sorted(&caller_return_bundles)));
        /* Enter procedure */
        self.transits.push(stmt.call_index);
        let defn = self.tree.get_procedure(stmt)?;
        /* Process return expressions */
        if let Some(defn) = defn {
            self.trace("    push namespace");
            self.namespace.push();
            self.trace("  regular procedure");
            self.return_exprs(&defn.ret, &caller_return_bundles)?;
            /* Process nested statements */
            for stmt in defn.block.iter().rev() {
                self.statement(stmt)?;
            }
            /* Process arguments */
            let expected_args = self.arg_bundles(&defn.args)?;
            self.trace(&format!("    arg bundles = {:?}",all_sorted(&expected_args)));
            self.trace("    pop namespace");
            self.namespace.pop();
            self.args(&expected_args,&stmt.args)?;
        } else {
            self.trace("  assignment");
            if let Some(expr) = stmt.args[0].skip_repeater() {
                self.return_exprs(&[expr], &caller_return_bundles)?;
            }
            self.args(&caller_return_bundles, &stmt.args)?;
        }
        /**/
        self.transits.pop();
        Ok(())
    }

    fn statement(&mut self, stmt: &BTStatement) -> Result<(),String> {
        self.trace(&format!("statement = {:?}",stmt));
        self.positions.push((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            BTStatementValue::Declare(d) => self.declare(d)?,
            BTStatementValue::BundledStatement(s) => self.procedure(s)?,
            _ => {}
        }
        self.positions.pop();
        Ok(())
    }
}

fn do_build_unbundle(tree: &BuildTree) -> Result<BuildUnbundle,String> {
    let mut unbundle = BuildUnbundle::new(tree);
    for stmt in tree.statements.iter().rev() {
        unbundle.statement(stmt).map_err(|e| unbundle.error_at(&e))?;
    }
    Ok(unbundle)
}

pub(crate) fn build_unbundle(tree: &BuildTree) -> Result<HashMap<(Vec<usize>,Position),Vec<String>>,String> {
    let unbundle = do_build_unbundle(tree)?;
    Ok(unbundle.transits.take())
}

#[cfg(test)]
pub(crate) fn trace_build_unbundle(tree: &BuildTree) -> Result<(HashMap<(Vec<usize>,Position),Vec<String>>,Vec<String>),String> {
    let unbundle = do_build_unbundle(tree)?;
    Ok((unbundle.transits.take(),unbundle.trace.clone()))
}
