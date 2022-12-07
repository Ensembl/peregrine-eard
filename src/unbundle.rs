use std::{sync::Arc, collections::{HashMap, HashSet}};

use crate::{buildtree::{BTStatement, BuildTree, BTStatementValue, BTProcCall, BTExpression, BTFuncProcDefinition, BTDefinition, BTLValue, BTRegisterType, BTFuncCall}, parsetree::at, model::{OrBundle, OrBundleRepeater, Variable, OrRepeater}, repeater::{list_repeaters, uses_repeater, is_good_repeater, remove_repeater}};

struct Bundle {
    debug_name: BundleKey,
    used: HashSet<String>,
    dead: bool
}

impl Bundle {
    fn new(debug_name: &BundleKey) -> Bundle {
        Bundle {
            debug_name: debug_name.clone(),
            used: HashSet::new(),
            dead: false
        }
    }

    fn name_used(&mut self, name: &str) {
        println!("name used {:?} in {:?}",name,self.debug_name);
        self.used.insert(name.to_string());
    }

    fn name_unused(&mut self, name: &str) {
        self.used.remove(name);
    }

    fn names(&self) -> &HashSet<String> { &self.used }

    fn merge(&mut self, from: &mut dyn Iterator<Item=&String>) {
        self.used.extend(from.cloned());
    }

    fn kill(&mut self) {
        self.dead = true;
        self.used.clear();
    }
}

impl std::fmt::Debug for Bundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut values = self.used.iter().cloned().collect::<Vec<_>>();
        values.sort();
        write!(f,"{:?}: {}",self.debug_name,values.join(", "))
    }
}

#[derive(Clone,Debug)]
struct BundleHandle(usize);

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
enum BundleKey {
    Name(String),
    Register(usize)
}

pub(crate) struct BundleStack {
    names: Vec<HashMap<BundleKey,usize>>,
    bundles: Vec<Bundle>
}

impl std::fmt::Debug for BundleStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i,bundle) in self.bundles.iter().enumerate() {
            write!(f,"{}: {:?}\n",i,bundle)?;
        }
        Ok(())
    }
}

impl BundleStack {
    fn new() -> BundleStack {
        BundleStack { 
            names: vec![HashMap::new()],
            bundles: vec![] 
        }
    }

    fn push_level(&mut self) {
        self.names.push(HashMap::new());
    }

    fn pop_level(&mut self) {
        self.names.pop();
    }

    fn new_anon_bundle(&mut self) -> BundleHandle {
        let bundle = Bundle::new(&BundleKey::Name("_".to_string()));
        let id = self.bundles.len();
        self.bundles.push(bundle);
        BundleHandle(id)
    }

    fn get(&mut self, key: BundleKey, force: bool) -> Option<BundleHandle> {
        let lookup = self.names.last_mut().unwrap();
        if let Some(id) = lookup.get(&key) {
            if self.bundles[*id].dead {
                lookup.remove(&key);
            }
        }
        if !lookup.contains_key(&key) && force {
            let bundle = Bundle::new(&key);
            let id = self.bundles.len();
            self.bundles.push(bundle);
            lookup.insert(key.clone(),id);
        }
        lookup.get(&key).map(|x| BundleHandle(*x))
    }

    fn get_by_name(&mut self, name: &str) -> BundleHandle {
        self.get(BundleKey::Name(name.to_string()),true).unwrap()
    }

    fn get_by_register(&mut self, reg: usize) -> BundleHandle {
        self.get(BundleKey::Register(reg),true).unwrap()
    }

    fn try_get_by_register(&mut self, reg: usize) -> Option<BundleHandle> {
        self.get(BundleKey::Register(reg),false)
    }

    fn merge(&mut self, to: &BundleHandle, from: &BundleHandle) {
        let names = &self.bundles[from.0].names().iter().cloned().collect::<Vec<_>>();
        let to = &mut self.bundles[to.0];
        to.merge(&mut names.iter());
    }

    fn get_mut(&mut self, handle: &BundleHandle) -> &mut Bundle { &mut self.bundles[handle.0] }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub(crate) enum Position {
    Arg(usize),
    Return(usize),
    Repeater
}

pub(crate) struct Unbundle<'a> {
    bt: &'a BuildTree,
    /* bundles is a stack of scopes for naming of bundles. Missed names will recurse down the
     * stack to outer contexts.
     */
    bundles: BundleStack,
    /* transits records that in the given circumstances (call index stack and Position enum
     * indicating argument number, etc), the bundle required usage of the set of elements
     * given in the value.
     */
    transits: HashMap<(Vec<usize>,Position),HashSet<String>>,
    /* The filepath / line number, for use in error handling.
     */
    position: Option<(Arc<Vec<String>>,usize)>,
    /* call stack is a stack of call indexes used to uniquely identify a context for a bundle's
     * use which functions as a key for later lookup of the bundle contents.
     */
    call_stack: Vec<usize>
}

impl<'a> Unbundle<'a> {
    pub(crate) fn new(bt: &'a BuildTree) -> Unbundle<'a> {
        Unbundle {
            bt,
            bundles: BundleStack::new(),
            transits: HashMap::new(),
            position: None,
            call_stack: vec![]
        }
    }

    fn error_at(&self, msg: &str) -> String {
        let pos = self.position.as_ref().map(|(f,p)| (f.as_slice(),*p));
        at(msg,pos)
    }

    fn unbundle_declare(&mut self, decl: &OrBundleRepeater<Variable>) -> Result<(),String> {
        match decl {
            OrBundleRepeater::Normal(v) => {
                if let Some(prefix) = &v.prefix {
                    let handle = self.bundles.get_by_name(prefix);
                    let bundle = self.bundles.get_mut(&handle);
                    bundle.name_unused(&v.name);
                }
            },
            OrBundleRepeater::Bundle(b) => {
                let handle = self.bundles.get_by_name(b);
                let bundle = self.bundles.get_mut(&handle);
                bundle.kill();
            },
            OrBundleRepeater::Repeater(r) => {
                let handle = self.bundles.get_by_name(r);
                let bundle = self.bundles.get_mut(&handle);
                bundle.kill();
            }
        }
        Ok(())
    }

    fn repeat_bundles<T: Clone+std::fmt::Debug>(&self, xx: &[OrBundleRepeater<T>]) -> Vec<Option<String>> {
        xx.iter().map(|x| {
            match x {
                OrBundleRepeater::Bundle(b) => Some(b.clone()),
                _ => None
            }
        }).collect::<Vec<_>>()
    }

    fn find_usage_expr(&mut self, outside: &Option<BundleHandle>, expr: &OrBundle<BTExpression>) -> Result<(),String> {
        println!("fue {:?}",expr);
        match expr {
            OrBundle::Normal(BTExpression::Variable(v)) => {
                if let Some(prefix) = &v.prefix {
                    let handle = self.bundles.get_by_name(prefix);
                    self.bundles.get_mut(&handle).name_used(&v.name);
                }
            },
            OrBundle::Normal(BTExpression::Function(call)) => {
                for arg in &call.args {
                    if !arg.is_repeater() {
                        println!("219 {:?}",arg);
                        self.find_usage_expr(outside,&arg.no_repeater()?)?;
                    }
                }
                self.unbundle_function(outside,call)?;
            },
            OrBundle::Normal(_) => {},
            OrBundle::Bundle(_) => {
                /* Usage of a bundle in a call is handled elsewhere during func/proc call
                 * processing.
                 */
            }
        }
        Ok(())
    }

    fn unbundle_function(&mut self, outside: &Option<BundleHandle>, call: &BTFuncCall) -> Result<(),String> {
        let defn = self.bt.get_function(call)?;
        self.bundles.push_level();
        self.call_stack.push(call.call_index);
        println!("unbundle function {:?}",self.call_stack);
        self.unbundle_return(&defn.ret[0],outside,0)?;
        for stmt in defn.block.iter().rev() {
            self.unbundle_stmt(stmt)?;
        }
        let inside_args = defn.args.iter().map(|arg| {
            match arg {
                OrBundle::Normal(_) => None,
                OrBundle::Bundle(b) => Some(self.bundles.get_by_name(b)),
            }
        }).collect::<Vec<_>>();
        println!("248 {:?}",inside_args);
        self.bundles.pop_level();
        for (i,(inside,outside)) in inside_args.iter().zip(call.args.iter()).enumerate() {
            self.unbundle_arg(outside, inside, i)?;
        }
        self.call_stack.pop();
        Ok(())
    }

    fn add_transit(&mut self, handle: &Option<BundleHandle>, position: Position) {
        if let Some(handle) = &handle {
            let names = self.bundles.get_mut(handle).names().clone();
            self.transits.insert((self.call_stack.clone(),position),names);
        }
    }

    fn make_ret_bundle(&mut self, expr: &OrBundle<BTExpression>) -> Result<Option<BundleHandle>,String> {
        Ok(match expr {
            OrBundle::Bundle(_) => Some(self.bundles.new_anon_bundle()),
            OrBundle::Normal(BTExpression::Function(f)) => {
                let defn = self.bt.get_function(f)?;
                self.make_ret_bundle(&defn.ret[0])?
            }
            OrBundle::Normal(_) => None
        })
    }

    fn unbundle_return(&mut self, expr: &OrBundle<BTExpression>, outside: &Option<BundleHandle>, arg_index: usize) -> Result<(),String> {
        match expr {
            OrBundle::Bundle(b) => {
                let handle = self.bundles.get_by_name(b);
                self.process_match(&Some(handle),outside)?;
                self.add_transit(outside,Position::Return(arg_index));
            },
            OrBundle::Normal(BTExpression::Function(f)) => {
                self.add_transit(outside,Position::Return(arg_index));
                self.unbundle_function(outside,f)?;
            },
            OrBundle::Normal(BTExpression::RegisterValue(r,BTRegisterType::Bundle)) => {
                let handle = self.bundles.get_by_register(*r);
                self.process_match(&Some(handle),outside)?;
                self.add_transit(outside,Position::Return(arg_index));
            },
            OrBundle::Normal(_) => {}
        }
        Ok(())
    }

    /* Called at a call site in the caller's name context with the bundle of values used from this
     * arg and the expression at the call site. If that expression is an explicit bundle or a bundle
     * register, just apply transitivity and record the names passed through. If it's a function
     * call, it could be a function returning a bundle.  In this case process this function, passing
     * in the argument from our call as its return but also record the names passing through.
     */
    fn unbundle_arg(&mut self, at_call_site: &OrBundleRepeater<BTExpression>, bundle_from_inside: &Option<BundleHandle>, arg_index: usize) -> Result<(),String> {
        match at_call_site {
            OrBundleRepeater::Normal(BTExpression::Function(f)) => {
                self.unbundle_function(bundle_from_inside,f)?;
                self.add_transit(bundle_from_inside,Position::Arg(arg_index));
            }
            OrBundleRepeater::Normal(BTExpression::RegisterValue(r,BTRegisterType::Bundle)) => {
                let handle = self.bundles.get_by_register(*r);
                self.process_match(&Some(handle),bundle_from_inside)?;
                self.add_transit(bundle_from_inside,Position::Arg(arg_index));
            },
            OrBundleRepeater::Bundle(b) => {
                println!("314 bundle_from_inside = {:?}",bundle_from_inside); // ???
                let handle = self.bundles.get_by_name(b);
                self.process_match(&Some(handle),&bundle_from_inside)?;
                self.add_transit(bundle_from_inside,Position::Arg(arg_index));
            },
            OrBundleRepeater::Repeater(_) => { /* An argument is never a bundle */ },
            OrBundleRepeater::Normal(_) => {},
        }
        Ok(())
    }

    fn process_match(&mut self, to: &Option<BundleHandle>, from: &Option<BundleHandle>) -> Result<(),String> {
        match (to,from) {
            (None,None) => {},
            (Some(inside), Some(outside)) => {
                self.bundles.merge(inside,outside);
            }
            _ => { return Err(format!("expected bundle in return inside={:?} outside={:?}",to,from)); }
        }
        Ok(())
    }

    fn get_bundle_from_lvalue(&mut self, lvalue: &BTLValue) -> Option<BundleHandle> {
        match lvalue {
            BTLValue::Register(r,BTRegisterType::Bundle) => Some(self.bundles.get_by_register(*r)),
            _ => None

        }
    }

    /* Build a list of which arguments in the return of a procedure are used by the caller. If
     * the results are disacrded, we need to create fake, anonymous bundles so that the rest of
     * the algorithm can proceed without special cases. Note that bundle can *either* bun register
     * *or* directly in the source.
     * 
     * eg (a,b) = test(); returns (None,None) : 1st branch; not discarded, but not bundles either
     *    (*a,b) = test(); returns (Some(..),None) : 1st branch; first arg is bundle
     *    test(); procedure test() { ...  (*a) } returns (Some(...)) : 2nd branch 
     */
    fn build_proc_return_bundles(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Vec<Option<BundleHandle>>,String> {
        Ok(if let Some(rets) = &call.rets {
            /* returns consumed */
            rets.iter().map(|x| {
                match x {
                    OrBundleRepeater::Normal(n) => self.get_bundle_from_lvalue(n),
                    OrBundleRepeater::Bundle(b) => Some(self.bundles.get_by_name(b)),
                    OrBundleRepeater::Repeater(_) => { None /* a repeater is never a bundle */ }
                }
            }).collect::<Vec<_>>()
        } else {
            /* returns not consumed */
            if let Some(defn) = self.bt.get_procedure(call)? {
                /* proper procedure */
                defn.ret.iter().map(|x| {
                    self.make_ret_bundle(x)
                }).collect::<Result<Vec<_>,_>>()?
            } else {
                /* assignment */
                println!("-A");
                vec![self.make_ret_bundle(&call.args[0].no_repeater()?)?]
            }
        })
    }

    fn list_lhs_repeaters(&self,rets: &[OrBundleRepeater<BTLValue>]) -> Result<Vec<String>,String> {
        let mut repeaters = vec![];
        for ret in rets {
            match ret {
                OrBundleRepeater::Normal(_) => {},
                OrBundleRepeater::Bundle(_) => {},
                OrBundleRepeater::Repeater(r) => { repeaters.push(r.to_string()); },
            }
        }
        Ok(repeaters)
    }


    fn list_rhs_repeaters(&self,args: &[OrBundleRepeater<BTExpression>]) -> Result<Vec<String>,String> {
        let mut repeaters = vec![];
        for arg in args {
            match arg {
                OrBundleRepeater::Normal(x) => {
                    match x {
                        BTExpression::Function(f) => {
                            repeaters.append(&mut self.list_rhs_repeaters(&f.args)?);
                        },
                        _ => {},
                    }
                },
                OrBundleRepeater::Bundle(_) => {},
                OrBundleRepeater::Repeater(r) => { repeaters.push(r.to_string()) }
            }
        }
        Ok(repeaters)
    }
    
    fn find_repeater_arguments(&self,call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Option<(String,String)>,String> {
        /* A good repeater has a single use of the repeater glyph (**) on each side and has a
         * single lhs variable.
         */
        let lhs = match &call.rets {
            Some(x) => {
                let lhs_repeaters = self.list_lhs_repeaters(call.rets.as_ref().unwrap_or(&vec![]))?;
                if lhs_repeaters.len() > 1 {
                    return Err("too many repeaters in lvalue".to_string());
                } else if lhs_repeaters.len() == 0 {
                    None
                } else if call.rets.as_ref().unwrap_or(&vec![]).len() != 1 {
                    return Err("repeater must be the only result in lvalue".to_string());
                } else {
                    match &x[0] {
                        OrBundleRepeater::Repeater(r) => { Some(r) },
                        _ => { None } // lhs is not repeater
                    }
                }
            },
            _ => { None }
        };
        let rhs_repeaters = self.list_rhs_repeaters(&call.args)?;
        if rhs_repeaters.len() > 1 {
            return Err("too many repeaters in rvalue".to_string());
        }
        let rhs = rhs_repeaters.first();
        Ok(match (lhs,rhs) {
            (Some(lhs),Some(rhs)) => Some((lhs.to_string(),rhs.to_string())),
            (None,None) => None,
            _ => { return Err("must be single repeater on left and right".to_string()); }
        })
    }
    
    /* Handles bundles passed in procedure return. For real procedures, these can be
     * 1. explicitly specified as a bundle; or
     * 2. the return of a function call.
     * 
     * For the latter case, we don't create intermediate "fake" bundles, but directly associate
     * the (potentially nested) place where the bundle is explicit with the outer call.
     * 
     */
     fn unbundle_proc(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        let outside_ret = self.build_proc_return_bundles(call)?;
        /* get handles for all bundles in return */
        if let Some(defn) = self.bt.get_procedure(call)? {
            /* real procedure */
            self.bundles.push_level();
            self.call_stack.push(call.call_index);
            println!("unbundle at stack {:?}",self.call_stack);
            /* We have some usage of a bundle at the outside level so we need to record this for
             * the transit out of the procedure for each argument.
             */
            for (i,outside) in outside_ret.iter().enumerate() {
                self.add_transit(&outside,Position::Return(i));
            }
            /* Within the procedure, the return expression may call functions which need to be
             * processed at this point (capturing transits, recursing through contents, etc).
             */
            for (i,(inside,outside)) in defn.ret.iter().zip(outside_ret.iter()).enumerate() {
                self.unbundle_return(inside,outside,i)?;
            }
            /* Ensure expressions used in function arguments in return expression are captured as
             * usages. Note: separate loop, as the previous can terminate early if returns are
             * discarded.
             */
            for ret in &defn.ret {
                let bundle = match ret {
                    OrBundle::Normal(BTExpression::Function(call)) => {
                        todo!()
                    },
                    _ => { None }
                };
                self.find_usage_expr(&bundle,ret)?;
            }
            /* Recursively process each statement contained in the procedure.
             */
            for stmt in defn.block.iter().rev() {
                self.unbundle_stmt(stmt)?;
            }
            /* We've now reached the top of the procedure and are looking at the argument list.
             * Some of these arguments may be bundles. While we still have the interior name
             * context, collect the contents of them and put them into "inside_args". Then pop the
             * name context so that we resolve names at the outer scope. At that point, we can
             * call unbundle_arg() with the variables used and the expression at call site. See
             * unbundle_arg() to see what it does with these. Finally, pop the context for the
             * transit declarations.
             */
            let inside_args = defn.args.iter().map(|arg| {
                match arg {
                    OrBundle::Normal(_) => None,
                    OrBundle::Bundle(b) => Some(self.bundles.get_by_name(b)),
                }
            }).collect::<Vec<_>>();
            self.bundles.pop_level();
            for (i,(bundle_from_inside,at_call_site)) in inside_args.iter().zip(call.args.iter()).enumerate() {
                self.unbundle_arg(at_call_site,bundle_from_inside,i)?;
            }
            self.call_stack.pop();
            for (arg,defn) in call.args.iter().zip(defn.args.iter()) {
                let bundle = match defn {
                    OrBundle::Normal(_) => None,
                    OrBundle::Bundle(b) => Some(self.bundles.get_by_name(b)),
                };
                println!("515 on {:?}",arg);
                self.find_usage_expr(&bundle,&arg.no_repeater()?)?;
            }    
        } else {
            /* assignment procedure */
            /* record return transit */
            self.call_stack.push(call.call_index);
            self.add_transit(&outside_ret[0],Position::Return(0));
            /* bind */
            self.unbundle_return(&call.args[0].no_repeater()?,&outside_ret[0],0)?;
            self.call_stack.pop();
            /* usage in rhs */
            self.find_usage_expr(&outside_ret[0],&call.args[0].no_repeater()?)?;
        }
        Ok(())
    }

    fn unbundle_proccall(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        match self.find_repeater_arguments(call)? {
            Some((lhs,rhs)) => {
                /* repeater */
                println!("UNBUNDLE REPEATER lhs={} rhs={}",lhs,rhs);
                /* Repeat the unbundling process iteratively across targets */
                let lhs_handle = self.bundles.get_by_name(&lhs);
                let rhs_handle = self.bundles.get_by_name(&rhs);
                self.call_stack.push(call.call_index);
                self.bundles.merge(&rhs_handle,&lhs_handle);
                self.add_transit(&Some(lhs_handle),Position::Repeater);
                self.call_stack.pop();
                self.unbundle_proc(call)?;
            },
            _ => {
                self.unbundle_proc(call)?;
            }
        }
        Ok(())
    }

    fn unbundle_stmt(&mut self, stmt: &BTStatement) -> Result<(),String> {
        println!("\nB {:?}",stmt);
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            BTStatementValue::Declare(d) => self.unbundle_declare(d)?,
            BTStatementValue::BundledStatement(s) => self.unbundle_proccall(s)?,
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn unbundle(&mut self) -> Result<(),String> {
        println!("A");
        for stmt in self.bt.statements.iter().rev() {
            self.unbundle_stmt(stmt)?;
        }
        Ok(())
    }

    pub(crate) fn bundle_stack(&self) -> &BundleStack { &self.bundles }
    pub(crate) fn transits(&self) -> &HashMap<(Vec<usize>,Position),HashSet<String>> { &self.transits }
}
