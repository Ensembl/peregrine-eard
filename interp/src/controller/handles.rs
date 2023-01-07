pub struct HandleStore<T> {
    objects: Vec<T>
}

impl<T> HandleStore<T> {
    pub fn new() -> HandleStore<T> {
        HandleStore { objects: vec![] }
    }
    
    pub fn push(&mut self, value: T) -> usize {
        let h = self.objects.len();
        self.objects.push(value);
        h
    }

    pub fn get(&self, reg: usize) -> Result<&T,String> {
        self.objects.get(reg).ok_or_else(|| format!("getting register r{} before setting",reg))
    }

    pub fn get_mut(&mut self, reg: usize) -> Result<&mut T,String> {
        self.objects.get_mut(reg).ok_or_else(|| format!("getting register r{} before setting",reg))
    }
}

impl<T: Default+Clone> HandleStore<T> {
    pub fn init(&mut self, max: usize, value: T) {
        self.objects.resize(max+1,value);
    }
}
