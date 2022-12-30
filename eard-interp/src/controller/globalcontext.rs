use std::sync::Arc;

use super::{context::{ContextTemplateBuilder, ContextTemplate, RunContext}, value::Value, handles::HandleStore};

pub struct GlobalBuildContext {
    pub patterns: ContextTemplate
}

impl GlobalBuildContext {
    pub(crate) fn new(mut patterns: ContextTemplateBuilder) -> GlobalBuildContext {
        GlobalBuildContext { patterns: patterns.build() }
    }
}

macro_rules! force {
    ($name:ident,$mname:ident,$out:ty,false) => {
        pub fn $name(&self, reg: usize) -> Result<&$out,String> {
            self.registers.get(reg)?.$name()
        }

        pub fn $mname(&mut self, reg: usize) -> Result<&mut $out,String> {
            self.registers.get_mut(reg)?.$mname()
        }
    
    };

    ($name:ident,$mname:ident,$out:ty,true) => {
        pub fn $name(&self, reg: usize) -> Result<$out,String> {
            self.registers.get(reg)?.$name()
        }

        pub fn $mname(&mut self, reg: usize) -> Result<&mut $out,String> {
            self.registers.get_mut(reg)?.$mname()
        }
    
    };
}

pub struct GlobalContext {
    pub registers: HandleStore<Value>,
    pub constants: Arc<Vec<Value>>,
    pub context: RunContext
}

impl GlobalContext {
    pub(crate) fn new(max_reg: usize, constants: &Arc<Vec<Value>>, context: RunContext) -> GlobalContext {
        let mut regs = HandleStore::new();
        regs.init(max_reg,Value::default());
        GlobalContext {
            registers: regs, constants: constants.clone(), context
        }
    }

    pub fn set(&mut self, reg: usize, value: Value) -> Result<(),String> {
        *self.registers.get_mut(reg)? = value;
        Ok(())
    }

    pub fn get(&self, reg: usize) -> Result<&Value,String> {
        self.registers.get(reg)
    }

    force!(force_boolean,force_boolean_mut,bool,true);
    force!(force_number,force_number_mut,f64,true);
    force!(force_string,force_string_mut,str,false);
    force!(force_finite_boolean,force_finite_boolean_mut,Vec<bool>,false);
    force!(force_finite_number,force_finite_number_mut,Vec<f64>,false);
    force!(force_finite_string,force_finite_string_mut,Vec<String>,false);
    force!(force_infinite_boolean,force_infinite_boolean_mut,bool,true);
    force!(force_infinite_number,force_infinite_number_mut,f64,true);
    force!(force_infinite_string,force_infinite_string_mut,str,false);

    pub fn is_finite(&self, reg: usize) -> Result<bool,String> {
        Ok(self.registers.get(reg)?.is_finite())
    }
}
