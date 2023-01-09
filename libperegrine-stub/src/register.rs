use eard_interp::{Operation, HandleStore, ContextItem, InterpreterBuilder, RunContext};
use crate::{stubs::{LeafRequest, Colour, Patina, ProgramShapesBuilder, Coords, StubDump, Request, Plotter, Pen}, ops::{op_leaf, op_leaf_s, op_style, op_colour, op_paint_solid, op_paint_solid_s, op_coord, op_rectangle, op_request, op_scope, op_get_data, op_data_boolean, op_data_number, op_data_string, op_graph_type, op_wiggle, op_setting_boolean, op_setting_number, op_setting_string, op_setting_boolean_seq, op_setting_number_seq, op_setting_string_seq, op_pen, op_text, op_paint_hollow, op_paint_hollow_s, op_bp_range, op_paint_special, op_image, op_running_text}, data::{StubResponses, Response}};

#[derive(Clone)]
pub struct LibPeregrineBuilder {
    shapes: ContextItem<ProgramShapesBuilder>,
    leafs: ContextItem<HandleStore<LeafRequest>>,
    colours: ContextItem<HandleStore<Colour>>,
    paint: ContextItem<HandleStore<Patina>>,
    coords: ContextItem<HandleStore<Coords>>,
    requests: ContextItem<HandleStore<Request>>,
    responses: ContextItem<HandleStore<Response>>,
    graph_types: ContextItem<HandleStore<Plotter>>,
    pens: ContextItem<HandleStore<Pen>>
}

pub fn build_libperegrine(builder: &mut InterpreterBuilder) -> Result<LibPeregrineBuilder,String> {
    let leafs = builder.add_context::<HandleStore<LeafRequest>>("leaf");
    let colours = builder.add_context::<HandleStore<Colour>>("colours");
    let paint = builder.add_context::<HandleStore<Patina>>("paint");
    let shapes = builder.add_context::<ProgramShapesBuilder>("shapes");
    let coords = builder.add_context::<HandleStore<Coords>>("coords");
    let requests = builder.add_context::<HandleStore<Request>>("requests");
    let responses = builder.add_context::<HandleStore<Response>>("responses");
    let graph_types = builder.add_context::<HandleStore<Plotter>>("graph-types");
    let pens = builder.add_context::<HandleStore<Pen>>("pens");
    builder.add_version("libperegrine",(0,0));
    builder.add_operation(256,Operation::new(op_leaf));
    builder.add_operation(257,Operation::new(op_leaf_s));
    builder.add_operation(258,Operation::new(op_style));
    builder.add_operation(259,Operation::new(op_colour));
    builder.add_operation(260,Operation::new(op_paint_solid));
    builder.add_operation(261,Operation::new(op_paint_solid_s));
    builder.add_operation(262,Operation::new(op_coord));
    builder.add_operation(263,Operation::new(op_rectangle));
    builder.add_operation(264,Operation::new(op_request));
    builder.add_operation(265,Operation::new(op_scope));
    builder.add_operation(266,Operation::new(op_get_data));
    builder.add_operation(267,Operation::new(op_data_boolean));
    builder.add_operation(268,Operation::new(op_data_number));
    builder.add_operation(269,Operation::new(op_data_string));
    builder.add_operation(270,Operation::new(op_graph_type));
    builder.add_operation(271,Operation::new(op_wiggle));
    builder.add_operation(272,Operation::new(op_setting_boolean));
    builder.add_operation(273,Operation::new(op_setting_number));
    builder.add_operation(274,Operation::new(op_setting_string));
    builder.add_operation(275,Operation::new(op_setting_boolean_seq));
    builder.add_operation(276,Operation::new(op_setting_number_seq));
    builder.add_operation(277,Operation::new(op_setting_string_seq));
    builder.add_operation(278,Operation::new(op_pen));
    builder.add_operation(279,Operation::new(op_text));
    builder.add_operation(280,Operation::new(op_paint_hollow));
    builder.add_operation(281,Operation::new(op_paint_hollow_s));
    builder.add_operation(282,Operation::new(op_bp_range));
    builder.add_operation(283,Operation::new(op_paint_special));
    builder.add_operation(284,Operation::new(op_image));
    builder.add_operation(285,Operation::new(op_running_text));
    Ok(LibPeregrineBuilder {
        leafs, shapes, colours, paint, coords, requests, responses, graph_types, pens
    })
}

pub fn prepare_libperegrine(context: &mut RunContext, builder: &LibPeregrineBuilder, stub_responses: StubResponses) -> Result<StubDump,String> {
    let shapes = ProgramShapesBuilder::new(stub_responses);
    context.add(&builder.leafs,HandleStore::new());
    context.add(&builder.colours,HandleStore::new());
    context.add(&builder.paint,HandleStore::new());
    context.add(&builder.coords,HandleStore::new());
    context.add(&builder.requests,HandleStore::new());
    context.add(&builder.responses,HandleStore::new());
    context.add(&builder.graph_types,HandleStore::new());
    context.add(&builder.pens,HandleStore::new());
    context.add(&builder.shapes,shapes.clone());
    Ok(StubDump(shapes))
}
