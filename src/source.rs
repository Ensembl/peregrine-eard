use std::{sync::Arc, collections::HashMap, fmt};

pub(crate) trait SourceSource {
    fn lookup(&self, pos: &ParsePosition, filename: &str) -> Result<(SourceSourceImpl,String),String>;
}

pub(crate) struct SourceSourceImpl(Box<dyn SourceSource>);

impl SourceSourceImpl {
    pub(crate) fn new<F>(soso: F) -> SourceSourceImpl where F: SourceSource + 'static {
        SourceSourceImpl(Box::new(soso))
    }
}

impl SourceSource for SourceSourceImpl {
    fn lookup(&self, pos: &ParsePosition, filename: &str) -> Result<(SourceSourceImpl,String),String> {
        self.0.lookup(pos,filename)
    }
}

#[derive(Clone)]
pub(crate) struct FixedSourceSource {
    files: Arc<HashMap<String,String>>
}

impl FixedSourceSource {
    pub(crate) fn new(files: HashMap<String,String>) -> FixedSourceSource {
        FixedSourceSource { files: Arc::new(files) }
    }
}

impl SourceSource for FixedSourceSource {
    fn lookup(&self, _pos: &ParsePosition, filename: &str) -> Result<(SourceSourceImpl,String),String> {
        let src = self.files.get(filename).cloned().ok_or_else(|| format!("cannot find '{}'",filename))?;
        Ok((SourceSourceImpl::new(self.clone()),src.to_string()))
    }
}

#[derive(Clone)]
pub(crate) struct FilePosition {
    pub filename: String,
    line_no: u32
}

impl FilePosition {
    fn anon() -> FilePosition {
        FilePosition { filename: "*anon*".to_string(), line_no: 0 }
    }

    fn new(filename: &str) -> FilePosition {
        FilePosition { filename: filename.to_string(), line_no: 0 }
    }
}

impl fmt::Debug for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}:{}",self.filename,self.line_no)
    }
}

#[derive(Clone)]
struct PositionNode(Option<Arc<PositionNode>>,FilePosition);

impl PositionNode {
    fn to_str(&self, prefix: &str, suffix: &str) -> String {
        let rest = self.0.as_ref().map(|parent| {
            format!("{}",parent.to_str(prefix,suffix))
        }).unwrap_or("".to_string());
        format!("{}{:?}{}{}",prefix,self.1,suffix,rest)
    }

    pub(crate) fn contains(&self, filename: &str) -> bool {
        if filename == self.1.filename { return true; }
        self.0.as_ref().map(|p| p.contains(filename)).unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct ParsePosition(PositionNode,Arc<String>);

impl ParsePosition {
    pub(crate) fn new(filename: &str, variety: &str) -> ParsePosition {
        ParsePosition(PositionNode(None,FilePosition::new(filename)),Arc::new(variety.to_string()))
    }

    pub(crate) fn contains(&self, filename: &str) -> bool {
        self.0.contains(filename)
    }

    pub(crate) fn empty(variety: &str) -> ParsePosition {
        ParsePosition(PositionNode(None,FilePosition::anon()),Arc::new(variety.to_string()))
    }

    pub(crate) fn at_line(&self, line_no: u32) -> ParsePosition {
        let mut out = self.clone();
        (out.0).1.line_no = line_no;
        out
    }

    pub(crate) fn update(&mut self, file: &FilePosition) {
        let parent = (self.0).0.clone();
        *self = ParsePosition(PositionNode(parent,file.clone()),self.1.clone());
    }

    pub(crate) fn add(&self, pos: &FilePosition) -> ParsePosition {
        ParsePosition(PositionNode(Some(Arc::new(self.0.clone())),pos.clone()),self.1.clone())
    }

    pub(crate) fn push(&self, filename: &str) -> ParsePosition {
        self.add(&FilePosition::new(filename))
    }

    pub(crate) fn last(&self) -> &FilePosition { &(self.0).1 }

    pub(crate) fn last_str(&self) -> String { format!("{:?}",self.last()) }

    pub(crate) fn full_str(&self) -> String {
        let rest = (self.0).0.as_ref().map(|x|
            x.to_str(&format!(" ({} from ",self.1),")")
        ).unwrap_or("".to_string());
        format!("{:?}{}",(self.0).1,rest)
    }

    pub(crate) fn message(&self, msg: &str) -> String {
        format!("{} at {}",msg,self.full_str())
    }
}

impl fmt::Debug for ParsePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.full_str())
    }
}
