use crate::{GlobalBuildContext, GlobalContext, Return, Value};

fn flip_one_base(input: char) -> char {
    match input {
        'c' => 'g', 'C' => 'G',
        'g' => 'c', 'G' => 'C',
        'a' => 't', 'A' => 'T',
        't' => 'a', 'T' => 'A',
        x => x
    }
}

pub(crate) fn op_base_flip(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let input = ctx.force_string(regs[1])?;
        eprintln!("input {:?}",input);
        let mut out = String::new();
        for data in input.chars() {
            out.push(flip_one_base(data));
        }
        eprintln!("output {:?}",out);
        ctx.set(regs[0],Value::String(out))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_base_flip_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let input = ctx.force_finite_string(regs[1])?;
            let mut out = vec![];
            for s in input {
                let mut flipped = String::new();
                for data in s.chars() {
                    flipped.push(flip_one_base(data));
                }
                out.push(flipped);
            }
            ctx.set(regs[0],Value::FiniteString(out))?;
        } else {
            let input = ctx.force_infinite_string(regs[1])?;
            let mut out = String::new();
            for data in input.chars() {
                out.push(flip_one_base(data));
            }
            ctx.set(regs[0],Value::InfiniteString(out))?;
        }
        Ok(Return::Sync)
    }))
}
