use std::{sync::Arc, collections::HashMap, fmt, path::PathBuf, env::current_dir, fs::read_to_string};

pub(crate) trait SourceSource {
    fn lookup(&self, filename: &str, fixed: bool) -> Result<(SourceSourceImpl,String),String>;
}

pub(crate) struct SourceSourceImpl(Box<dyn SourceSource>);

impl SourceSourceImpl {
    pub(crate) fn new<F>(soso: F) -> SourceSourceImpl where F: SourceSource + 'static {
        SourceSourceImpl(Box::new(soso))
    }
}

impl SourceSource for SourceSourceImpl {
    fn lookup(&self,filename: &str, fixed: bool) -> Result<(SourceSourceImpl,String),String> {
        self.0.lookup(filename,fixed)
    }
}

#[derive(Clone)]
pub(crate) struct NoneSourceSource;

impl SourceSource for NoneSourceSource {
    fn lookup(&self, filename: &str, _fixed: bool) -> Result<(SourceSourceImpl,String),String> {
        return Err(format!("No loader for {}",filename))
    }
}

#[derive(Clone,Debug)]
pub struct FixedSourceSource {
    files: Arc<HashMap<String,String>>
}

impl FixedSourceSource {
    pub fn new(files: HashMap<String,String>) -> FixedSourceSource {
        FixedSourceSource { files: Arc::new(files) }
    }

    pub fn new_vec(mut files: Vec<(&str,&str)>) -> FixedSourceSource {
        let files = files.drain(..).map(|(k,v)| (k.to_string(),v.to_string())).collect::<HashMap<_,_>>();
        Self::new(files)
    }
}

impl SourceSource for FixedSourceSource {
    fn lookup(&self, filename: &str, fixed: bool) -> Result<(SourceSourceImpl,String),String> {
        if !fixed {
            return Err("cannot find files from inside fixed source".to_string());
        }
        let src = self.files.get(filename).cloned().ok_or_else(|| format!("cannot find '{}'",filename))?;
        Ok((SourceSourceImpl::new(self.clone()),src.to_string()))
    }
}

#[derive(Clone)]
pub(crate) struct FileSourceSource {
    rel_path: PathBuf
}

impl FileSourceSource {
    fn new() -> Result<FileSourceSource,String> {
        let cwd = current_dir().map_err(|e| format!("couldn't get current directory: {}",e))?;
        Ok(FileSourceSource { rel_path: cwd })
    }

    fn lookup_file(&self, filename: &str, fixed: bool) -> Result<(FileSourceSource,String),String> {
        if fixed {
            return Err("cannot find fixed sources from inside file".to_string());
        }
        let mut new_path = self.rel_path.clone();
        new_path.push(filename);
        let rel_path = new_path.parent().ok_or_else(|| format!("Cannot find parent directory of {}",filename))?;
        let new_source = FileSourceSource{ rel_path: rel_path.to_path_buf() };
        let contents = read_to_string(new_path).map_err(|e| format!("cannot read {}: {}",filename,e))?;
        Ok((new_source,contents))
    }
}

impl SourceSource for FileSourceSource {
    fn lookup(&self, filename: &str, fixed: bool) -> Result<(SourceSourceImpl,String),String> {
        let (source,input) = self.lookup_file(filename,fixed)?;
        Ok((SourceSourceImpl::new(source),input))
    }
}

pub(crate) struct CombinedSourceSourceBuilder {
    file: FileSourceSource,
    fixed: Vec<FixedSourceSource>
}

impl CombinedSourceSourceBuilder {
    pub(crate) fn new() -> Result<CombinedSourceSourceBuilder,String> {
        Ok(CombinedSourceSourceBuilder { file: FileSourceSource::new()?, fixed: vec![] })
    }

    pub(crate) fn add_fixed(&mut self, source: &FixedSourceSource) {
        self.fixed.push(source.clone());
    }
}

pub(crate) struct CombinedSourceSource {
    file: FileSourceSource,
    fixed: Arc<Vec<FixedSourceSource>>
}

impl CombinedSourceSource {
    pub(crate) fn new(builder: &CombinedSourceSourceBuilder) -> CombinedSourceSource {
        CombinedSourceSource { file: builder.file.clone(), fixed: Arc::new(builder.fixed.clone()) }
    }
}

impl SourceSource for CombinedSourceSource {
    fn lookup(&self, filename: &str, fixed: bool) -> Result<(SourceSourceImpl,String),String> {
        if fixed {
            for src in self.fixed.as_ref() {
                if let Some(out) = src.lookup(filename,true).ok() {
                    return Ok(out);
                }
            }
            return Err(format!("missing builtin file {}",filename));
        } else {
            let (file,input) = self.file.lookup_file(filename,false)?;
            let soso = CombinedSourceSource {
                file, fixed: self.fixed.clone()
            };
            Ok((SourceSourceImpl::new(soso),input))
        }
    }
}

#[derive(Clone)]
pub(crate) struct FilePosition {
    soso: Arc<SourceSourceImpl>,
    filename: String,
    suppress: bool,
    line_no: u32
}

impl FilePosition {
    fn anon(soso: SourceSourceImpl) -> FilePosition {
        FilePosition { soso: Arc::new(soso), filename: "*anon*".to_string(), line_no: 0, suppress: true }
    }

    fn new(soso: SourceSourceImpl, filename: &str) -> FilePosition {
        FilePosition { soso: Arc::new(soso), filename: filename.to_string(), line_no: 0, suppress: false }
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
        if self.1.suppress {
            rest
        } else {
            format!("{}{:?}{}{}",prefix,self.1,suffix,rest)
        }
    }

    pub(crate) fn contains(&self, filename: &str) -> bool {
        if filename == self.1.filename { return true; }
        self.0.as_ref().map(|p| p.contains(filename)).unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct ParsePosition(PositionNode,Arc<String>);

impl ParsePosition {
    pub(crate) fn contains(&self, filename: &str) -> bool {
        self.0.contains(filename)
    }

    pub(crate) fn root(soso: SourceSourceImpl,variety: &str) -> ParsePosition {
        ParsePosition(PositionNode(None,FilePosition::anon(soso)),Arc::new(variety.to_string()))
    }

    pub(crate) fn empty(variety: &str) -> ParsePosition {
        Self::root(SourceSourceImpl::new(NoneSourceSource),variety)
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

    pub(crate) fn push(&self, filename: &str, fixed: bool) -> Result<(String,ParsePosition),String> {
        let (soso,input) = self.last().soso.lookup(filename,fixed)?;
        let new_pos = self.add(&FilePosition::new(soso,filename));
        Ok((input,new_pos))
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
