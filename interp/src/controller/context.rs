use std::{marker::PhantomData, any::Any, mem, sync::Arc, collections::HashMap, fmt};

pub struct ContextItem<T>(usize,PhantomData<T>);

impl<T> Clone for ContextItem<T> {
    fn clone(&self) -> Self {
        ContextItem(self.0.clone(),self.1.clone())
    }
}

impl<T> fmt::Debug for ContextItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

pub struct ContextTemplateBuilder {
    name: HashMap<String,(usize,Box<dyn Any>)>
}

impl ContextTemplateBuilder {
    pub fn new() -> ContextTemplateBuilder {
        ContextTemplateBuilder { name: HashMap::new() }
    }

    pub fn add<T: 'static>(&mut self, name: &str) -> Result<ContextItem<T>,String> {
        let creator : PhantomData<T> = PhantomData;
        let h = self.name.len();
        if self.name.contains_key(name) {
            return Err(format!("duplicate context key {}",name));
        }
        self.name.insert(name.to_string(),(h,Box::new(creator)));
        Ok(ContextItem(h,PhantomData))
    }

    pub fn build(&mut self) -> ContextTemplate {
        ContextTemplate { name: Arc::new(mem::replace(&mut self.name,HashMap::new())) }
    }
}

#[derive(Clone)]
pub struct ContextTemplate {
    name: Arc<HashMap<String,(usize,Box<dyn Any>)>>
}

impl ContextTemplate {
    pub fn lookup<T: 'static>(&self, name: &str) -> Result<ContextItem<T>,String> {
        let (index,atom) = self.name.get(name).ok_or_else(|| format!("cannot find context {}",name))?;
        if !atom.is::<PhantomData<T>>() {
            return Err(format!("wrong type for context {}: is {:?}",name,atom.type_id()));
        }
        Ok(ContextItem(*index,PhantomData))
    }
}

pub struct RunContext {
    context: Vec<Option<Box<dyn Any>>>
}

impl RunContext {
    pub fn new() -> RunContext {
        RunContext { context: vec![] }
    }

    pub fn add<T: Any>(&mut self, atom: &ContextItem<T>, value: T) {
        if self.context.len() <= atom.0 {
            self.context.resize_with(atom.0+1,|| None);
        }
        self.context[atom.0] = Some(Box::new(value));
    }

    pub fn get<T: Any>(&self, atom: &ContextItem<T>) -> &T {
        self.context[atom.0].as_ref().unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: Any>(&mut self, atom: &ContextItem<T>) -> &mut T {
        self.context[atom.0].as_mut().unwrap().downcast_mut::<T>().unwrap()
    }
}
