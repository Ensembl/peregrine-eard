use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Value, Return};
use ordered_float::OrderedFloat;
use crate::{stubs::{LeafRequest, ProgramShapesBuilder, Colour, Patina, Coords, Shape, Rectangle, Request, Plotter, Wiggle, Pen, Text, Hollow }, data::{Response, DataValue}};

fn to_u8(v: f64) -> u8 { (v*255.) as u8 }

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

fn leaf_from_handle(ctx: &GlobalContext, leafs: &HandleStore<LeafRequest>, reg: usize) -> Result<Vec<LeafRequest>,String> {
    Ok(if !ctx.is_finite(reg)? {
        vec![leafs.get(ctx.force_infinite_number(reg)? as usize)?.clone()]
    } else if ctx.is_atomic(reg)? {
        vec![leafs.get(ctx.force_number(reg)? as usize)?.clone()]
    } else {
        ctx.force_finite_number(reg)?.iter().map(|h| {
            leafs.get(*h as usize).cloned()
        }).collect::<Result<Vec<_>,_>>()?
    })
}

pub(crate) fn op_rectangle(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let paints = ctx.context.get(&paints);
        let nw = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1] as usize)? as usize)?.clone();
        let leafs = leaf_from_handle(ctx,leafs,regs[3])?;
        let paint = paints.get(ctx.force_number(regs[2])? as usize)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Rectangle(Rectangle(nw.clone(),se.clone(),paint.clone(),leafs)));
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_request(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    Ok(Box::new(move |ctx,regs| {
        let backend = ctx.force_string(regs[1])?.to_string();
        let endpoint = ctx.force_string(regs[2])?.to_string();
        let req = Request::new(&backend,&endpoint);
        let requests = ctx.context.get_mut(&requests);
        let h = requests.push(req);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_scope(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[0])? as usize;
        let k = ctx.force_string(regs[1])?.to_string();
        let v = ctx.force_string(regs[2])?.to_string();
        let requests = ctx.context.get_mut(&requests);
        let req = requests.get_mut(h)?;
        req.scope(&k,&v);
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_get_data(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let requests = ctx.context.get(&requests);
        let req = requests.get(h)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        let res = shapes.add_request(&req)?;
        let responses = ctx.context.get_mut(&responses);
        let h = responses.push(res);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::Boolean(b) => b.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::Number(n) => n.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::String(n) => n.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteString(value))?;
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

pub(crate) fn op_wiggle(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    let graph_types = gctx.patterns.lookup::<HandleStore<Plotter>>("graph-types")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let bp_left = ctx.force_number(regs[0])?;
        let bp_right = ctx.force_number(regs[1])?;
        let graph_type = ctx.force_number(regs[2])? as usize;
        let values = ctx.force_finite_number(regs[3])?;
        let full_values = if ctx.is_finite(regs[4])? {
            let present = ctx.force_finite_boolean(regs[4])?;
            values.iter().zip(present.iter().cycle()).map(|(v,p)| {
                if *p { Some(OrderedFloat(*v)) } else { None }
            }).collect()
        } else {
            let present = ctx.force_infinite_boolean(regs[4])?;
            if present {
                values.iter().map(|x| Some(OrderedFloat(*x))).collect()
            } else {
                vec![None;values.len()]
            }
        };
        let leafs = ctx.context.get(&leafs);
        let leaf = ctx.force_number(regs[5])? as usize;
        let leaf = leafs.get(leaf)?;
        let graph_types = ctx.context.get(&graph_types);
        let graph_type = graph_types.get(graph_type)?;
        let wiggle = Wiggle(
            OrderedFloat(bp_left),OrderedFloat(bp_right),
            graph_type.clone(),full_values,leaf.clone()
        );
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Wiggle(wiggle));
        Ok(Return::Sync)
    }))
}

fn to_boolean(data: &DataValue) -> Vec<bool> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => b.to_vec(),
        DataValue::Number(n) => { n.iter().map(|v| *v!=0.).collect() },
        DataValue::String(s) => { s.iter().map(|v| v!="").collect() }
    }
}

fn to_number(data: &DataValue) -> Vec<f64> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => { b.iter().map(|b| if *b {1.} else {0.}).collect() },
        DataValue::Number(n) => { n.to_vec() },
        DataValue::String(s) => { s.iter().map(|v| v.parse::<f64>().unwrap_or(0.)).collect() }
    }
}

fn to_string(data: &DataValue) -> Vec<String> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => { b.iter().map(|b| (if *b {"true"} else {"false"}).to_string()).collect() },
        DataValue::Number(n) => { n.iter().map(|n| n.to_string()).collect() },
        DataValue::String(s) => { s.to_vec() }
    }
}

pub(crate) fn op_setting_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_boolean(&shapes.get_setting(key,path)?);
        let value = value.iter().any(|v| *v);
        ctx.set(regs[0],Value::Boolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let mut value = to_number(&shapes.get_setting(key,path)?);
        let value = value.drain(..).reduce(f64::max).unwrap_or(0.);
        ctx.set(regs[0],Value::Number(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_string(&shapes.get_setting(key,path)?);
        let value = value.join("\n");
        ctx.set(regs[0],Value::String(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_boolean_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_boolean(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_number(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_string(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteString(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_pen(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let font = ctx.force_string(regs[1])?.to_string();
        let size = ctx.force_number(regs[2])?;
        let fgd_h = ctx.force_number(regs[3])? as usize;
        let bgd_h = ctx.force_number(regs[4])? as usize;
        let colours = ctx.context.get(&colours);
        let fgd = colours.get(fgd_h)?.clone();
        let bgd = colours.get(bgd_h)?.clone();
        let pen = Pen {
            font, size: OrderedFloat(size), fgd, bgd
        };
        let pens = ctx.context.get_mut(&pens);
        let h = pens.push(pen);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_text(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let pens = ctx.context.get(&pens);
        let coords = coords.get(ctx.force_number(regs[0])? as usize)?.clone();
        let pen = pens.get(ctx.force_number(regs[1])? as usize)?.clone();
        let text = ctx.force_finite_string(regs[2])?.to_vec();
        let leaf = leaf_from_handle(ctx,leafs,regs[3])?;
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Text(Text { coords, pen, text, leaf }));
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
