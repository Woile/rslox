use crate::{ast::Literal, interpreter::RuntimeError, scanner::Token};
use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Environment {
    values: Rc<RefCell<HashMap<String, Literal>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn define(&self, name: &str, value: Literal) {
        let envmap = &*self.values.clone();
        envmap.borrow_mut().insert(name.to_string(), value);
        println!("updated {:?}", self.values);
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        println!("{:?}", self.values);
        (*self.values.clone().borrow())
            .get(&name.lexeme)
            .and_then(|v| Some(v.clone()))
            .ok_or_else(|| {
                RuntimeError(name.line, format!("Undefined variable {}", name.lexeme)).into()
            })
    }
}
