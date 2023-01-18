use eachorevery::eoestruct::{StructTemplate, struct_to_json, StructValue};
use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Value, Return};
use ordered_float::OrderedFloat;
use crate::{stubs::{LeafRequest, ProgramShapesBuilder, Colour, Patina, Coords, Plotter,Pen, Hollow, Dotted }, util::to_number};

fn to_u8(v: f64) -> u8 { v as u8 }

pub(crate) fn op_leaf_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let spec = ctx.force_finite_string(regs[1])?.to_vec();
        let shapes = ctx.context.get_mut(&shapes);
        let mut leaf_list = spec.iter().map(|spec| {
            shapes.use_allotment(&spec).clone()
        }).collect::<Vec<_>>();
        drop(shapes);
        let leafs = ctx.context.get_mut(&leafs);
        let h = leaf_list.drain(..).map(|leaf| leafs.push(leaf) as f64).collect();
        ctx.set(regs[0],Value::FiniteNumber(h))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_leaf(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let spec = ctx.force_string(regs[1])?.to_string();
        let shapes = ctx.context.get_mut(&shapes);
        let leaf = shapes.use_allotment(&spec).clone();
        drop(shapes);
        let leafs = ctx.context.get_mut(&leafs);
        let h = leafs.push(leaf);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_style(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let path = ctx.force_string(regs[0])?.to_string();
        let key = ctx.force_finite_string(regs[1])?;
        let value = ctx.force_finite_string(regs[2])?;
        if key.len() != value.len() {
            return Err(format!("key and value not same length in style"));
        }
        let kvs = key.iter().zip(value.iter()).map(|(k,v)| {
            (k.to_string(),v.to_string())
        }).collect();
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_style(&path,kvs);
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_colour(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    Ok(Box::new(move |ctx,regs| {
        let r = to_u8(ctx.force_number(regs[1])?);
        let g = to_u8(ctx.force_number(regs[2])?);
        let b = to_u8(ctx.force_number(regs[3])?);
        let a = to_u8(ctx.force_number(regs[4])?);
        let colours = ctx.context.get_mut(&colours);
        let h = colours.push(Colour {r,g,b,a});
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_solid(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let colours = ctx.context.get_mut(&colours);
        let colour = colours.get(h)?.clone();
        let paint = Patina::Solid(vec![colour.clone()]);
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_hollow(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let colours = ctx.context.get_mut(&colours);
        let colour = colours.get(h)?.clone();
        let width = ctx.force_number(regs[2])?;
        let paint = Patina::Hollow(Hollow(vec![colour.clone()],OrderedFloat(width)));
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_solid_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let paint = if ctx.is_finite(regs[1])? {
            let h = ctx.force_finite_number(regs[1])?;
            let colours = ctx.context.get(&colours);
            let colour = h.iter().map(|h| {
                colours.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?;
            Patina::Solid(colour)
        } else {
            let h = ctx.force_infinite_number(regs[1])? as usize;
            let colours = ctx.context.get(&colours);
            let colour = colours.get(h)?.clone();
            Patina::Solid(vec![colour])
        };
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_hollow_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let width = ctx.force_number(regs[2])?;
        let paint = if ctx.is_finite(regs[1])? {
            let h = ctx.force_finite_number(regs[1])?;
            let colours = ctx.context.get(&colours);
            let colour = h.iter().map(|h| {
                colours.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?;
            Patina::Hollow(Hollow(colour,OrderedFloat(width)))
        } else {
            let h = ctx.force_infinite_number(regs[1])? as usize;
            let colours = ctx.context.get(&colours);
            let colour = colours.get(h)?.clone();
            Patina::Hollow(Hollow(vec![colour],OrderedFloat(width)))
        };
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn coord_to_eoe(ctx: &GlobalContext, reg: usize) -> Result<Vec<OrderedFloat<f64>>,String> {
    Ok(if ctx.is_finite(reg)? {
        ctx.force_finite_number(reg)?.iter().map(|x| OrderedFloat(*x)).collect::<Vec<_>>()
    } else {
        vec![OrderedFloat(ctx.force_infinite_number(reg)?)]
    })
}

pub(crate) fn op_coord(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    Ok(Box::new(move |ctx,regs| {
        let b = coord_to_eoe(ctx,regs[1])?;
        let n = coord_to_eoe(ctx,regs[2])?;
        let t = coord_to_eoe(ctx,regs[3])?;
        let coords = ctx.context.get_mut(&coords);
        let sb = Coords { b, t, n };
        let h = coords.push(sb);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_graph_type(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let graph_types = gctx.patterns.lookup::<HandleStore<Plotter>>("graph-types")?;
    Ok(Box::new(move |ctx,regs| {
        let height = ctx.force_number(regs[1])? as u32;
        let colour_handle = ctx.force_number(regs[2])? as usize;
        let colours = ctx.context.get(&colours);
        let colour = colours.get(colour_handle)?.clone();
        let graph_type = Plotter {
            height, colour
        };
        let graph_types = ctx.context.get_mut(&graph_types);
        let h = graph_types.push(graph_type);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_pen(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let font = ctx.force_string(regs[1])?.to_string();
        let size = ctx.force_number(regs[2])?;
        let colours = ctx.context.get(&colours);
        let fgd = to_colours(ctx,colours,regs[3])?;
        let bgd = to_colours(ctx,colours,regs[4])?;
        let pen = Pen {
            font, size: OrderedFloat(size), fgd, bgd
        };
        let pens = ctx.context.get_mut(&pens);
        let h = pens.push(pen);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_bp_range(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let value = to_number(&shapes.get_request("bp_range","")?);
        ctx.set(regs[0],Value::Number(value[0]))?;
        ctx.set(regs[1],Value::Number(value[1]))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_special(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let special = ctx.force_string(regs[1])?;
        let paint = Patina::Special(special.to_string());
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn json_ser(tmpl: &StructTemplate) -> Result<String,String> {
    let json = struct_to_json(&tmpl.build()?,None)?;
    Ok(json.to_string())
}

pub(crate) fn op_zmenu(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let templates = ctx.context.get(&templates);
        let variety_h = ctx.force_number(regs[1])? as usize;
        let content_h = ctx.force_number(regs[2])? as usize;
        let zmenu_variety = templates.get(variety_h)?.clone();
        let zmenu_content = templates.get(content_h)?.clone();
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(Patina::ZMenu(json_ser(&zmenu_variety)?,json_ser(&zmenu_content)?));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn to_colours(ctx: &GlobalContext, colours: &HandleStore<Colour>, reg: usize) -> Result<Vec<Colour>,String> {
    if ctx.is_finite(reg)? {
        let values = ctx.force_finite_number(reg)?;
        values.iter().map(|h| {
            colours.get(*h as usize).cloned()
        }).collect::<Result<Vec<_>,_>>()    
    } else {
        Ok(vec![colours.get(ctx.force_infinite_number(reg)? as usize).cloned()?])
    }
}

pub(crate) fn op_paint_dotted(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let colours = ctx.context.get(&colours);
        let colour_a = to_colours(ctx,colours,regs[1])?;
        let colour_b= to_colours(ctx,colours,regs[2])?;
        let length = ctx.force_number(regs[3])?;
        let width = ctx.force_number(regs[4])?;
        let prop = ctx.force_number(regs[5])?;
        let paint = Patina::Dotted(Dotted(colour_a,colour_b,OrderedFloat(length),OrderedFloat(width),OrderedFloat(prop)));
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_metadata(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let templates = ctx.context.get(&templates);
        let key = ctx.force_string(regs[1])?.to_string();
        let values_id = ctx.force_finite_string(regs[2])?;
        let values_h = ctx.force_finite_number(regs[3])?;
        let values = values_h.iter().zip(values_id.iter()).map(|(h,id)| {
            let build = templates.get(*h as usize)?.build()?;
            let value = StructValue::new_expand(&build,None)?;
            Ok::<_,String>((id.to_string(),value))
        }).collect::<Result<Vec<_>,_>>()?;
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(Patina::Metadata(key.to_string(),values));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}
