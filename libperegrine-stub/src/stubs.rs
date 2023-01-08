use std::{collections::{HashSet, HashMap}, sync::{Arc, Mutex}};

use ordered_float::OrderedFloat;
use serde::{Serialize, ser::{SerializeSeq, SerializeMap}};

use crate::{data::{Response, DataValue}, StubResponses};

#[derive(PartialEq,Eq,Hash,Clone,PartialOrd, Ord)]
pub(crate) struct LeafRequest {
    name: String
}

impl Serialize for LeafRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.serialize_str(&self.name)
    }
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Colour {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8
}

impl Serialize for Colour {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.r)?;
        seq.serialize_element(&self.g)?;
        seq.serialize_element(&self.b)?;
        seq.serialize_element(&self.a)?;
        seq.end()
    }
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hollow(pub(crate) Vec<Colour>,pub(crate) OrderedFloat<f64>);

impl Serialize for Hollow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("colour",&self.0)?;
        map.serialize_entry("width",&self.1.0)?;
        map.end()
    }
}

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Patina {
    Solid(Vec<Colour>),
    Hollow(Hollow)
}

impl Serialize for Patina {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Patina::Solid(s) => {
                map.serialize_key("solid")?;
                map.serialize_value(s)?;
            },
            Patina::Hollow(s) => {
                map.serialize_key("hollow")?;
                map.serialize_value(s)?;
            }
        }
        map.end()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord,Clone)]
pub(crate) struct Coords {
    pub(crate) b: Vec<OrderedFloat<f64>>,
    pub(crate) t: Vec<OrderedFloat<f64>>,
    pub(crate) n: Vec<OrderedFloat<f64>>
}

impl Serialize for Coords {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.b.iter().map(|x| x.0).collect::<Vec<_>>())?;
        seq.serialize_element(&self.t.iter().map(|x| x.0).collect::<Vec<_>>())?;
        seq.serialize_element(&self.n.iter().map(|x| x.0).collect::<Vec<_>>())?;
        seq.end()                
    }
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) struct Pen {
    pub(crate) font: String,
    pub(crate) size: OrderedFloat<f64>,
    pub(crate) fgd: Colour,
    pub(crate) bgd: Colour
}

impl Serialize for Pen {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_key("font")?;
        map.serialize_value(&self.font)?;
        map.serialize_key("size")?;
        map.serialize_value(&self.size.0)?;
        map.serialize_key("fgd")?;
        map.serialize_value(&self.fgd)?;
        map.serialize_key("bgd")?;
        map.serialize_value(&self.bgd)?;
        map.end()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Text {
    pub(crate) coords: Coords,
    pub(crate) pen: Pen,
    pub(crate) text: Vec<String>,
    pub(crate) leaf: Vec<LeafRequest>
}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_key("coords")?;
        map.serialize_value(&self.coords)?;
        map.serialize_key("pen")?;
        map.serialize_value(&self.pen)?;
        map.serialize_key("text")?;
        map.serialize_value(&self.text)?;
        map.serialize_key("leaf")?;
        map.serialize_value(&self.leaf)?;
        map.end()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Plotter {
    pub(crate) height: u32,
    pub(crate) colour: Colour
}

impl Serialize for Plotter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_key("height")?;
        map.serialize_value(&self.height)?;
        map.serialize_key("colour")?;
        map.serialize_value(&self.colour)?;
        map.end()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Rectangle(pub(crate) Coords,pub(crate) Coords,pub(crate) Patina,pub(crate) Vec<LeafRequest>);

impl Serialize for Rectangle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.serialize_element(&self.2)?;
        seq.serialize_element(&self.3)?;
        seq.end()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Wiggle(
    pub(crate) OrderedFloat<f64>, pub(crate) OrderedFloat<f64>, pub(crate) Plotter, 
    pub(crate) Vec<Option<OrderedFloat<f64>>>, pub(crate) LeafRequest
);

impl Serialize for Wiggle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let values = self.3.iter().map(|x| {
            x.map(|x| x.0)
        }).collect::<Vec<_>>();
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("left",&self.0.0)?;
        map.serialize_entry("right",&self.1.0)?;
        map.serialize_entry("graph_type",&self.2)?;
        map.serialize_entry("values",&values)?;
        map.serialize_entry("leaf",&self.4)?;
        map.end()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Shape {
    Rectangle(Rectangle),
    Text(Text),
    Wiggle(Wiggle)
}

impl Serialize for Shape {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Shape::Rectangle(r) => {
                map.serialize_key("rectangle")?;
                map.serialize_value(r)?;
            },
            Shape::Wiggle(r) => {
                map.serialize_key("wiggle")?;
                map.serialize_value(r)?;
            },
            Shape::Text(t) => {
                map.serialize_key("text")?;
                map.serialize_value(t)?;
            }
        }
        map.end()                
    }
}

#[derive(Clone)]
struct RequestScope(Vec<(String,String)>);

impl Serialize for RequestScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(None)?;
        for (k,v) in &self.0 {
            map.serialize_key(k)?;
            map.serialize_value(v)?;
        }
        map.end()
    }
}

#[derive(Clone)]
pub(crate) struct Request {
    backend: String,
    endpoint: String,
    scope: RequestScope
}

impl Request {
    pub(crate) fn new(backend: &str, endpoint: &str) -> Request {
        Request {
            backend: backend.to_string(),
            endpoint: endpoint.to_string(),
            scope: RequestScope(vec![])
        }
    }

    pub(crate) fn scope(&mut self, key: &str, value: &str) {
        self.scope.0.push((key.to_string(),value.to_string()));
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_key("backend")?;
        map.serialize_value(&self.backend)?;
        map.serialize_key("endpoint")?;
        map.serialize_value(&self.endpoint)?;
        map.serialize_key("scope")?;
        map.serialize_value(&self.scope)?;
        map.end()
    }
}

pub(crate) struct ProgramShapesBuilderImpl {
    stubs: StubResponses,
    requests: Vec<Request>,
    leafs: HashSet<LeafRequest>,
    style: HashMap<String,Vec<(String,String)>>,
    shapes: Vec<Shape>,
    used: bool
}

impl ProgramShapesBuilderImpl {
    fn new(stubs: StubResponses) -> ProgramShapesBuilderImpl {
        ProgramShapesBuilderImpl {
            stubs,
            requests: vec![],
            leafs: HashSet::new(),
            style: HashMap::new(),
            shapes: vec![],
            used: false
        }
    }

    fn use_allotment(&mut self, spec: &str) -> LeafRequest {
        self.used = true;
        let leaf = LeafRequest { name: spec.to_string() };
        self.leafs.insert(leaf.clone());
        leaf
    }

    fn add_style(&mut self, spec: &str, pairs: Vec<(String,String)>) {
        self.used = true;
        self.style.entry(spec.to_string()).or_insert(vec![]).extend_from_slice(&pairs);
    }

    fn add_shape(&mut self, shape: Shape) {
        self.used = true;
        self.shapes.push(shape);
    }

    fn get_setting(&self, key: &str, path: &[String]) -> Result<&DataValue,String> {
        self.stubs.get_setting(key,path)
    }

    fn get_request(&self, key: &str, part: &str) -> Result<&DataValue,String> {
        self.stubs.get_request(key,part)
    }

    pub(crate) fn add_request(&mut self, req: &Request) -> Result<Response,String> {
        self.used = true;
        self.requests.push(req.clone());
        self.stubs.get(&req.backend,&req.endpoint).cloned()
    }
}

impl Serialize for ProgramShapesBuilderImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(4))?;
        let mut leafs = self.leafs.iter().collect::<Vec<_>>();
        leafs.sort();
        map.serialize_key("leafs")?;
        map.serialize_value(&leafs)?;
        let mut shapes = self.shapes.iter().collect::<Vec<_>>();
        shapes.sort();
        map.serialize_key("shapes")?;
        map.serialize_value(&shapes)?;
        let mut styles = self.style.iter().map(|(k,v)| (k.clone(),v.clone())).collect::<Vec<_>>();
        for (_,v) in styles.iter_mut() {
            v.sort();
        }
        styles.sort();
        map.serialize_key("styles")?;
        map.serialize_value(&styles)?;
        map.serialize_key("requests")?;
        map.serialize_value(&self.requests)?;
        map.end()
    }
}

#[derive(Clone)]
pub(crate) struct ProgramShapesBuilder(Arc<Mutex<ProgramShapesBuilderImpl>>);

impl ProgramShapesBuilder {
    pub(crate) fn new(stubs: StubResponses) -> ProgramShapesBuilder {
        ProgramShapesBuilder(Arc::new(Mutex::new(ProgramShapesBuilderImpl::new(stubs))))
    }

    pub(crate) fn use_allotment(&mut self, spec: &str) -> LeafRequest {
        self.0.lock().unwrap().use_allotment(spec)
    }

    pub(crate) fn add_style(&mut self, spec: &str, pairs: Vec<(String,String)>) {
        self.0.lock().unwrap().add_style(spec,pairs);
    }

    pub(crate) fn add_shape(&mut self, shape: Shape) {
        self.0.lock().unwrap().add_shape(shape);
    }

    pub(crate) fn add_request(&mut self, req: &Request) -> Result<Response,String> {
        self.0.lock().unwrap().add_request(req)
    }

    pub(crate) fn get_setting(&self, key: &str, path: &[String]) -> Result<DataValue,String> {
        self.0.lock().unwrap().get_setting(key,path).cloned()
    }

    pub(crate) fn get_request(&self, key: &str, part: &str) -> Result<DataValue,String> {
        self.0.lock().unwrap().get_request(key,part).cloned()
    }
}

#[derive(Clone)]
pub struct StubDump(pub(crate) ProgramShapesBuilder);

impl StubDump {
    pub fn used(&self) -> bool {
        (self.0).0.lock().unwrap().used
    }
}

impl Serialize for StubDump {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(3))?;
        let shapes = (self.0).0.lock().unwrap();
        map.serialize_key("actions")?;
        map.serialize_value(&*shapes)?;
        map.end()                                
    }
}
