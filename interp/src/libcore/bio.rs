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

pub(crate) fn op_base_flip(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

pub(crate) fn op_base_flip_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn ruler_interval(region: i64, max_points: i64, seps: &[i64]) -> i64 {
    let mut b10_value = 1;
    for _ in 0..20 {
        for mul in seps {
            let value = b10_value*mul;
            if region / value < max_points {
                return value;
            }
        }
        b10_value *= 10;
    }
    b10_value
}

pub(crate) fn op_ruler_interval(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let region = ctx.force_number(regs[1])? as i64;
        let max_points = ctx.force_number(regs[2])? as i64;
        let seps = ctx.force_finite_number(regs[3])?.iter().map(|x| *x as i64).collect::<Vec<_>>();
        let interval = ruler_interval(region,max_points,&seps);
        ctx.set(regs[0],Value::Number(interval as f64))?;
        Ok(Return::Sync)
    }))
}

fn ruler_markings(interval: i64, min: i64, max: i64) -> Vec<f64> {
    let mut out = vec![];
    let mut pos = min;
    if pos%interval != 0 { pos += interval - (pos%interval); }
    while pos < max {
        out.push(pos as f64);
        pos += interval;
    }
    out
}

pub(crate) fn op_ruler_markings(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let interval = ctx.force_number(regs[1])? as i64;
        let min = ctx.force_number(regs[2])? as i64;
        let max = ctx.force_number(regs[3])? as i64;
        let interval = ruler_markings(interval,min,max);
        ctx.set(regs[0],Value::FiniteNumber(interval))?;
        Ok(Return::Sync)
    }))
}
