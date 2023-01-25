use eard_interp::{HandleStore, Return, GlobalBuildContext, GlobalContext};
use ordered_float::OrderedFloat;
use crate::stubs::{ProgramShapesBuilder, LeafRequest, Shape, RunningText, Coords, Image, Wiggle, Rectangle, Plotter, Text, Pen, Patina, Empty, RunningRectangle, RectangleJoin, Polygon};

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

pub(crate) fn op_empty(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let nw = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1] as usize)? as usize)?.clone();
        let leafs = leaf_from_handle(ctx,leafs,regs[2])?;
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Empty(Empty(nw.clone(),se.clone(),leafs)));
        Ok(Return::Sync)
    }))
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

pub(crate) fn op_running_rectangle(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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
        let run = ctx.force_finite_number(regs[2])?.iter().map(|x| OrderedFloat(*x)).collect();
        let leafs = leaf_from_handle(ctx,leafs,regs[4])?;
        let paint = paints.get(ctx.force_number(regs[3])? as usize)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::RunningRectangle(RunningRectangle(nw.clone(),se.clone(),run,paint.clone(),leafs)));
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_rectangle_join(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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
        let leafs_a = leaf_from_handle(ctx,leafs,regs[3])?;
        let leafs_b = leaf_from_handle(ctx,leafs,regs[4])?;
        let paint = paints.get(ctx.force_number(regs[2])? as usize)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::RectangleJoin(RectangleJoin(nw.clone(),se.clone(),paint.clone(),leafs_a,leafs_b)));
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

pub(crate) fn op_running_text(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let pens = ctx.context.get(&pens);
        let nw = coords.get(ctx.force_number(regs[0])? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1])? as usize)?.clone();
        let pen = pens.get(ctx.force_number(regs[2])? as usize)?.clone();
        let text = ctx.force_finite_string(regs[3])?.to_vec();
        let leaf = leaf_from_handle(ctx,leafs,regs[4])?;
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::RunningText(RunningText { nw, se, pen, text, leaf }));
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_image(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let coord = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let images = if ctx.is_finite(regs[1])? {
            ctx.force_finite_string(regs[1])?.clone()
        } else {
            vec![ctx.force_infinite_string(regs[1])?.to_string()]
        };
        let leafs = leaf_from_handle(ctx,leafs,regs[2])?;
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Image(Image(coord.clone(),images.clone(),leafs)));
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_polygon(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<Coords>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let paints = ctx.context.get(&paints);
        let centre = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let radius = ctx.force_finite_number(regs[1])?.iter().map(|x| OrderedFloat(*x)).collect::<Vec<_>>();
        let points = ctx.force_number(regs[2])? as usize;
        let angle = ctx.force_number(regs[3])? as usize;
        let paint = paints.get(ctx.force_number(regs[4])? as usize)?.clone();
        let leaf = leaf_from_handle(ctx,leafs,regs[5])?;
        let shapes = ctx.context.get_mut(&shapes);
        shapes.add_shape(Shape::Polygon(Polygon(centre.clone(),radius.clone(),points,angle,paint.clone(),leaf)));
        Ok(Return::Sync)
    }))
}
