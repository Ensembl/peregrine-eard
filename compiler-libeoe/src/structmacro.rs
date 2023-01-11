use std::collections::HashMap;
use eard_compiler_lib::{OrBundleRepeater, PTExpression, ParsePosition, PTStatement, PTStatementValue, PTCall, Variable, OrBundle, Constant};
use rand::random;
use crate::{parser::parse, parsetree::VarType};

pub(crate) struct StructMacro {
    pos: ParsePosition,
    out: Vec<PTStatement>,
    context: usize,
    base: String,
    varnum: usize,
    pub(crate) group_vars: Vec<String>,
    pub(crate) var_vars: Vec<Option<String>>,
    pub(crate) var_ids: HashMap<usize,usize>
}

impl StructMacro {
    fn new(pos: &ParsePosition, context: usize, var_ids: HashMap<usize,usize>) -> Result<StructMacro,String> {
        Ok(StructMacro {
            pos: pos.clone(),
            out: vec![],
            context,
            base: format!("{:x}{:x}",random::<u64>(),random::<u64>()),
            varnum: 0,
            group_vars: vec![],
            var_vars: vec![],
            var_ids
        })
    }

    fn new_var(&mut self) -> String {
        self.varnum += 1;
        format!("__eoes_internal_{}_{}",self.base,self.varnum)
    }

    fn add_let(&mut self, lhs: Variable, rhs: PTExpression) -> Result<(),String> {
        self.out.push(PTStatement {
            value: PTStatementValue::LetStatement(
                vec![OrBundleRepeater::Normal((lhs,vec![]))],
                vec![OrBundle::Normal(rhs)]
            ),
            position: self.pos.clone(), 
            context: self.context
        });
        Ok(())
    }

    pub(crate) fn call(&self, name: &str, args: &[PTExpression]) -> Result<PTExpression,String> {
        let args = args.iter().map(|r| OrBundleRepeater::Normal(r.clone())).collect();
        Ok(PTExpression::Call(PTCall {
            name: name.to_string(),
            args,
            is_macro: false
        }))
    }

    fn extract_var(&mut self, i: usize, vartype: &VarType, expr: &OrBundleRepeater<PTExpression>) -> Result<(),String> {
        let expr = match expr {
            OrBundleRepeater::Normal(x) => x,
            _ => { return Err(format!("struct argument must be expression")); },
        };
        let varname = self.new_var();
        if let Some(group_idx) = self.var_ids.get(&i) {
            let group = &self.group_vars[*group_idx];
            let expr = self.call(&vartype.to_var_call(),&[
                PTExpression::Variable(Variable { prefix: None, name: group.to_string() }),
                expr.clone()
            ])?;
            self.add_let(Variable { prefix: None, name: varname.clone() },expr)?;
            self.var_vars.push(Some(varname));    
        } else {
            self.var_vars.push(None);
        }
        Ok(())
    }

    fn add_group(&mut self) -> Result<(),String> {
        let varname = self.new_var();
        let expr = self.call("eoe_group",&[])?;
        self.add_let(Variable { prefix: None, name: varname.clone() },expr)?;
        self.group_vars.push(varname);
        Ok(())
    }
}

pub(crate) fn struct_macro(expr: &[OrBundleRepeater<PTExpression>], pos: &ParsePosition, context: usize) -> Result<Vec<PTStatement>, String> {
    /* Check and parse */

    if expr.len() < 2 {
        return Err(format!("struct macro expects at least two arguments"));
    }
    let ident = match &expr[0] {
        OrBundleRepeater::Normal(PTExpression::Variable(x)) => x.name.clone(),
        _ => { return Err(format!("struct needs identifier as first argument")); },
    };
    let spec = match &expr[1] {
        OrBundleRepeater::Normal(PTExpression::Constant(Constant::String(c))) => c.clone(),
        _ => { return Err(format!("struct needs string spec as second argument")); },
    };

    /* Make parse tree */

    let mut spec = parse(&spec)?;
    let mut next_group = 0;
    let mut var_ids = HashMap::new();
    spec.assign_all_ids(&mut next_group, &mut var_ids)?;
    let mut struct_macro = StructMacro::new(pos,context,var_ids)?;

    /* Create groups */

    for _ in 0..next_group {
        struct_macro.add_group()?;
    }

    /* Get argument types and put them into vars */

    let mut types = vec![];
    spec.arg_types(&mut types);
    for (i,vartype) in types.iter().enumerate() {
        if expr.len() <= i+2 {
            return Err(format!("struct macro given too few arguments for template"));
        }
        struct_macro.extract_var(i,vartype,&expr[i+2])?;
    }

    /* Build main expression */

    let expr = spec.build_expression(&mut struct_macro)?;
    struct_macro.add_let(Variable { prefix: None, name: ident },expr)?;
    Ok(struct_macro.out)
}
