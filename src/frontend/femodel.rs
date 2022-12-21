use std::fmt;

#[derive(Clone)]
pub enum OrBundle<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String)
}

impl<T: fmt::Debug+Clone> fmt::Debug for OrBundle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(v) => write!(f,"{:?}",v),
            Self::Bundle(b) => write!(f,"*{}",b),
        }
    }
}

#[derive(Clone)]
pub enum OrBundleRepeater<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String),
    Repeater(String)
}

impl<T: fmt::Debug+Clone> fmt::Debug for OrBundleRepeater<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(v) => write!(f,"{:?}",v),
            Self::Bundle(b) => write!(f,"*{}",b),
            Self::Repeater(r) => write!(f,"**{}",r),
        }
    }
}

impl<T: std::fmt::Debug+Clone> OrBundleRepeater<T> {
    pub(crate) fn map<F,U: std::fmt::Debug+Clone>(&self, cb: F) -> OrBundleRepeater<U> where F: FnOnce(&T) -> U {
        match self {
            OrBundleRepeater::Normal(n) => OrBundleRepeater::Normal(cb(n)),
            OrBundleRepeater::Bundle(b) => OrBundleRepeater::Bundle(b.clone()),
            OrBundleRepeater::Repeater(r) => OrBundleRepeater::Repeater(r.clone())
        }
    }

    pub(crate) fn map_result<F,U: std::fmt::Debug+Clone,E>(&self, cb: F) -> Result<OrBundleRepeater<U>,E> where F: FnOnce(&T) -> Result<U,E> {
        Ok(match self {
            OrBundleRepeater::Normal(n) => OrBundleRepeater::Normal(cb(n)?),
            OrBundleRepeater::Bundle(b) => OrBundleRepeater::Bundle(b.clone()),
            OrBundleRepeater::Repeater(r) => OrBundleRepeater::Repeater(r.clone())
        })
    }

    pub(crate) fn is_repeater(&self) -> bool {
        match self {
            OrBundleRepeater::Repeater(_) => true,
            _ => false
        }
    }

    pub(crate) fn no_repeater(&self) -> Result<OrBundle<T>,String> {
        Ok(match self {
            OrBundleRepeater::Normal(n) => OrBundle::Normal(n.clone()),
            OrBundleRepeater::Bundle(b) => OrBundle::Bundle(b.clone()),
            OrBundleRepeater::Repeater(_) => { return Err(format!("unexpected repeater")) }
        })
    }

    pub(crate) fn skip_repeater(&self) -> Option<OrBundle<T>> {
        match self {
            OrBundleRepeater::Normal(n) => Some(OrBundle::Normal(n.clone())),
            OrBundleRepeater::Bundle(b) => Some(OrBundle::Bundle(b.clone())),
            OrBundleRepeater::Repeater(_) => { None }
        }

    }
}
