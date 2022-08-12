use std::{sync::Arc, collections::{HashMap, HashSet}};

use crate::{buildtree::{BTStatement, BuildTree, BTStatementValue, BTProcCall, BTExpression, BTFuncProcDefinition, BTDefinition, BTLValue, BTRegisterType, BTFuncCall}, parsetree::at, model::{OrBundle, OrBundleRepeater, Variable, OrRepeater}, repeater::{list_repeaters, uses_repeater, is_good_repeater}};

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
    Return(usize)
}

pub(crate) struct Unbundle<'a> {
    bt: &'a BuildTree,
    bundles: BundleStack,
    transits: HashMap<(Vec<usize>,Position),HashSet<String>>,
    position: Option<(Arc<Vec<String>>,usize)>,
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
            OrBundleRepeater::Repeater(_) => todo!(),
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

    fn find_usage_expr(&mut self, expr: &OrBundle<BTExpression>) -> Result<(),String> {
        match expr {
            OrBundle::Normal(BTExpression::Variable(v)) => {
                if let Some(prefix) = &v.prefix {
                    let handle = self.bundles.get_by_name(prefix);
                    self.bundles.get_mut(&handle).name_used(&v.name);
                }
            },
            OrBundle::Normal(BTExpression::Function(call)) => {
                for arg in &call.args {
                    self.find_usage_expr(&arg.no_repeater()?)?;
                }
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

    fn find_usage_proc(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        for arg in &call.args {
            self.find_usage_expr(&arg.no_repeater()?)?;
        }
        Ok(())
    }

    fn unbundle_repeater(&self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        let source = match &call.rets.as_ref().expect("unbundling repeater from non-repeater")[0] {
            OrBundleRepeater::Repeater(r) => r,
            _ => { panic!("unbundling repeater from non-repeater {:?}",call); }
        };
        let target = list_repeaters(&call.args)?[0].to_string();
        Ok(())
    }

    fn unbundle_function(&mut self, outside: &Option<BundleHandle>, call: &BTFuncCall) -> Result<(),String> {
        let defn = self.bt.get_function(call)?;
        self.bundles.push_level();
        self.call_stack.push(call.call_index);
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
            }
            OrBundle::Normal(BTExpression::RegisterValue(r,BTRegisterType::Bundle)) => {
                let handle = self.bundles.get_by_register(*r);
                self.process_match(&Some(handle),outside)?;
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
                let handle = self.bundles.get_by_name(b);
                self.process_match(&Some(handle),&bundle_from_inside)?;
                self.add_transit(bundle_from_inside,Position::Arg(arg_index));
            },
            OrBundleRepeater::Repeater(_) => { todo!() },
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

    fn build_proc_return_bundles(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Vec<Option<BundleHandle>>,String> {
        Ok(if let Some(rets) = &call.rets {
            /* returns consumed */
            rets.iter().map(|x| {
                match x {
                    OrBundleRepeater::Normal(n) => self.get_bundle_from_lvalue(n),
                    OrBundleRepeater::Bundle(b) => Some(self.bundles.get_by_name(b)),
                    OrBundleRepeater::Repeater(_) => { todo!(); }
                }
            }).collect::<Vec<_>>()
        } else {
            /* returns not consumed */
            if let Some(defn) = self.bt.get_procedure(call)? {
                defn.ret.iter().map(|x| {
                    self.make_ret_bundle(x)
                }).collect::<Result<Vec<_>,_>>()?
            } else {
                vec![self.make_ret_bundle(&call.args[0].no_repeater()?)?]
            }
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
            /* record return transit */
            for (i,outside) in outside_ret.iter().enumerate() {
                self.add_transit(&outside,Position::Return(i));
            }
            /* bind retuns */
            for (i,(inside,outside)) in defn.ret.iter().zip(outside_ret.iter()).enumerate() {
                self.unbundle_return(inside,outside,i)?;
            }
            /* usage in return expressions */
            for ret in &defn.ret {
                self.find_usage_expr(ret)?;
            }
            /* process each statement */
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
        } else {
            /* assignment procedure */
            /* record return transit */
            self.call_stack.push(call.call_index);
            self.add_transit(&outside_ret[0],Position::Return(0));
            /* bind */
            self.unbundle_return(&call.args[0].no_repeater()?,&outside_ret[0],0)?;
            self.call_stack.pop();
            /* usage in rhs */
            self.find_usage_expr(&call.args[0].no_repeater()?)?;
        }
        /* find usage in args to this call */
        self.find_usage_proc(call)?;
        Ok(())
    }

    fn unbundle_proccall(&mut self, call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        if uses_repeater(call)? {
            if is_good_repeater(call)? {
                self.unbundle_repeater(call)?;
            } else {
                return Err(self.error_at("bad repeater statement"));
            }
        } else {
            self.unbundle_proc(call)?;
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
