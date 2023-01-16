use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, value::Value, operation::Return, handles::HandleStore};

fn apply_template(template: &str, values: &[String]) -> Option<String> {
    let mut out = String::new();
    let mut lone_obrace = false;
    let mut lone_cbrace = false;
    let mut value = None;
    for c in template.chars() {
        if lone_obrace {
            if c == '{' { out.push('{'); }
            else if c.is_digit(10) { 
                value = c.to_string().parse::<usize>().ok();
                if value.is_none() { return None; }
            }
            else { return None; }
            lone_obrace = false;
        } else if lone_cbrace {
            if c == '}' { out.push('}'); }
            else { return None; }
            lone_cbrace = false;
        } else if value.is_some() {
            if c == '}' { out.push_str(&values[value.unwrap()]); value = None; }
            else if c.is_digit(10) {
                let more = c.to_string().parse::<usize>().ok();
                if more.is_none() { return None; }
                value = Some(value.unwrap()*10+more.unwrap());
            }
        } else if c == '{' {
            lone_obrace = true;
        } else if c == '}' {
            lone_cbrace = true;
        } else {
            out.push(c);
        }
    }
    if lone_obrace || lone_cbrace || value.is_some() { return None; }
    return Some(out);
}

pub(crate) fn op_concat(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let sep = ctx.force_string(regs[1])?;
        let parts = ctx.force_finite_string(regs[2])?;
        ctx.set(regs[0],Value::String(parts.join(sep)))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let first = ctx.force_string(regs[1])?;
        let second = ctx.force_string(regs[2])?;
        ctx.set(regs[0],Value::String(format!("{}{}",first,second)))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let base = ctx.force_finite_string(regs[1])?;
            let extra = ctx.force_string(regs[2])?;
            let out = base.iter().map(|s| format!("{}{}",s,extra)).collect();
            ctx.set(regs[0],Value::FiniteString(out))?;    
        } else {
            let first = ctx.force_infinite_string(regs[1])?;
            let second = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::InfiniteString(format!("{}{}",first,second)))?;    
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str_revs(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let extra = ctx.force_string(regs[1])?;
            let base = ctx.force_finite_string(regs[2])?;
            let out = base.iter().map(|s| format!("{}{}",extra,s)).collect();
            ctx.set(regs[0],Value::FiniteString(out))?;
        } else {
            let first = ctx.force_string(regs[1])?;
            let second = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::InfiniteString(format!("{}{}",first,second)))?;    
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str_ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let a = ctx.force_finite_string(regs[1])?;
                let b = ctx.force_finite_string(regs[2])?;
                if a.len() != b.len() {
                    return Err(format!("lengths do not match in push_str {:?} {:?}",a,b));
                }
                let out = a.iter().zip(b.iter()).map(|(a,b)| format!("{}{}",a,b)).collect();
                ctx.set(regs[0],Value::FiniteString(out))?;
            },
            (true, false) => {
                let a = ctx.force_finite_string(regs[1])?;
                let b = ctx.force_infinite_string(regs[2])?;
                let out = a.iter().map(|a| format!("{}{}",a,b)).collect();
                ctx.set(regs[0],Value::FiniteString(out))?;
            },
            (false, true) => {
                let a = ctx.force_infinite_string(regs[1])?;
                let b = ctx.force_finite_string(regs[2])?;
                let out = b.iter().map(|b| format!("{}{}",a,b)).collect();
                ctx.set(regs[0],Value::FiniteString(out))?;
            },
            (false, false) => {
                let a = ctx.force_infinite_string(regs[1])?;
                let b = ctx.force_infinite_string(regs[2])?;
                ctx.set(regs[0],Value::InfiniteString(format!("{}{}",a,b)))?;
            }
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_split(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let sep = ctx.force_string(regs[1])?;
        let input = ctx.force_string(regs[2])?;
        let out = input.split(sep).map(|x| x.to_string()).collect();
        ctx.set(regs[0],Value::FiniteString(out))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_template(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let template = ctx.force_string(regs[1])?;
        let input = ctx.force_finite_string(regs[2])?;
        let out = apply_template(template,&input).unwrap_or("".to_string());
        ctx.set(regs[0],Value::String(out))?;
        Ok(Return::Sync)
    }))
}

fn rotate(input: &Vec<Vec<String>>) -> Vec<Vec<String>> {
    let num = input.iter().map(|x| x.len()).max().unwrap_or(0);
    let mut out = vec![];
    for pos in 0..num {
        let mut more = vec![];
        for item in input.iter() {
            more.push(item.get(pos).map(|x| x.to_string()).unwrap_or_else(|| "".to_string()));
        }
        out.push(more);
    }
    out
}

pub(crate) fn op_split_start(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_splits = gctx.patterns.lookup::<HandleStore<Vec<Vec<String>>>>("splits")?;
    Ok(Box::new(move |ctx,regs| {
        let sep = ctx.force_string(regs[1])?;
        let input = ctx.force_finite_string(regs[2])?;
        let mut split = vec![];
        for item in input {
            let parts = item.split(sep).map(|x| x.to_string());
            let parts = parts.collect::<Vec<_>>();
            split.push(parts);
        }
        let out = rotate(&split);
        let splits = ctx.context.get_mut(&libcore_splits);        
        let h = splits.push(out);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_split_get(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_splits = gctx.patterns.lookup::<HandleStore<Vec<Vec<String>>>>("splits")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let idx = ctx.force_number(regs[2])? as usize;
        let splits = ctx.context.get(&libcore_splits);        
        let values = splits.get(h)?;
        ctx.set(regs[0],Value::FiniteString(values.get(idx).cloned().unwrap_or(vec![])))?;
        Ok(Return::Sync)
    }))
}

#[derive(Debug)]
pub(crate) struct Template {
    template: String,
    parts: Vec<Vec<String>>
}

impl Template {
    fn new(template: &str) -> Template {
        Template { template: template.to_string(), parts: vec![] }
    }

    fn add(&mut self, index: usize, value: Vec<String>) {
        if self.parts.len() <= index {
            self.parts.resize_with(index+1,|| vec![]);
        }
        self.parts[index] = value;
    }

    fn get(&self) -> Vec<String> {
        let rotated = rotate(&self.parts);
        rotated.iter().map(|parts| {
            apply_template(&self.template,&parts).unwrap_or("".to_string())
        }).collect()
    }
}

pub(crate) fn op_template_start(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_templates = gctx.patterns.lookup::<HandleStore<Template>>("templates")?;
    Ok(Box::new(move |ctx,regs| {
        let spec = ctx.force_string(regs[1])?;
        let tmpl = Template::new(spec);
        let templates = ctx.context.get_mut(&libcore_templates);
        let h = templates.push(tmpl);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_template_set(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_templates = gctx.patterns.lookup::<HandleStore<Template>>("templates")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[0])? as usize;
        let pos = ctx.force_number(regs[1])? as usize;
        let values = ctx.force_finite_string(regs[2])?.clone();
        let templates = ctx.context.get_mut(&libcore_templates);
        let tmpl = templates.get_mut(h)?;
        tmpl.add(pos,values);
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_template_end(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_templates = gctx.patterns.lookup::<HandleStore<Template>>("templates")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let templates = ctx.context.get_mut(&libcore_templates);
        let tmpl = templates.get_mut(h)?;
        let out = tmpl.get();
        ctx.set(regs[0],Value::FiniteString(out))?;
        Ok(Return::Sync)
    }))
}
