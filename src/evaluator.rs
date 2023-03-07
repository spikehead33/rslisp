use std::clone;
use std::{
    rc::Rc,
    collections::HashMap,
    cell::RefCell,
};
use crate::parser::{Object, FunctionBody, FunctionDefinition, Param, ParamKind};
use crate::location::Location;

pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    vars: HashMap<String, Object>
}

impl Environment {
    /// initialize the environment with the built-in functions
    /// the only identifier for the builtin function is that their 
    /// location's filename is "__builtin__" while the rol and 
    /// col are both equals to 0
    fn create_builtin_funcdef() -> Object {
        Object::Lambda { 
            value: FunctionDefinition {
                params: vec![Param {
                    kind: ParamKind::Variadic ,
                    loc: Some(Location::new("__builtin__".to_string(), 0, 0))
                }],
                body: FunctionBody(vec![])
            },
            loc: None
        }
    }

    /// check if the function is a builtin function
    pub fn is_builtin(object: &Object) -> bool {
        object
        .loc()
        .and_then(|l| Some(l.filename() == "__builtin__".to_string()))
        .unwrap_or(false)
    }

    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        let vars = HashMap::from_iter([
            ("+".to_string(), Environment::create_builtin_funcdef()),
            ("-".to_string(), Environment::create_builtin_funcdef()),
            ("*".to_string(), Environment::create_builtin_funcdef()),
            ("/".to_string(), Environment::create_builtin_funcdef()),
            ("%".to_string(), Environment::create_builtin_funcdef()),
            (">".to_string(), Environment::create_builtin_funcdef()),
            ("<".to_string(), Environment::create_builtin_funcdef()),
            ("=".to_string(), Environment::create_builtin_funcdef()),
            (">=".to_string(), Environment::create_builtin_funcdef()),
            ("<=".to_string(), Environment::create_builtin_funcdef()),
            ("/=".to_string(), Environment::create_builtin_funcdef()),
        ]);

        Self {
            parent,
            vars
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.vars.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                self.parent
                    .as_ref()
                    .and_then(|e| e.borrow().get(name).clone())
            }
        }
    }

    pub fn set(&mut self, name: &str, obj: Object) {
        self.vars.insert(name.to_string(), obj);
    }
}

pub fn eval(object: Object, env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    eval_obj(&object, env)
}

pub fn eval_obj(obj: &Object, env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    match obj {
        Object::Void { .. }
        | Object::Lambda { .. }
        | Object::Bool { .. }
        | Object::Integer { .. }
        | Object::Float { .. }
        | Object::Str { .. } => Ok(obj.clone()),
        Object::Symbol { value: ref s, .. } => eval_symbol(s.as_str(), env),
        Object::List { value, .. }
        | Object::Module { value, .. } => eval_list(&value.as_slice(), env),
    }
}

pub fn eval_symbol(s: &str, env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    env.borrow()
        .get(s)
        .and_then(|s| Some(s.clone()))
        .ok_or(format!("Symbol not found: {:?}", s))
}

pub fn eval_list(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    match list.first() {
        Some(Object::Symbol { ref value, ..}) => match value.as_str() {
            "define" => eval_define(&list[1..], env),
            "if" => eval_if(&list[1..], env),
            "lambda" => eval_function_definition(&list[1..], env),
            _ => eval_function_call(&list[1..], env)
        },
        None => Ok(Object::Void { loc: None }),  // Empty list `()`
        _ => {
            unreachable!()
        }
    }
}

pub fn eval_define(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    let object = if let Some(obj) = list.first() {
        obj
    } else {
        return Err("".to_string());
    };

    let name = if let Object::Symbol { value, .. } = object {
        value.clone()
    } else {
        return Err(format!(
            "Expect Symbol/identifier but {} found at {:?}", object, object.loc()))
    };

    let val = if let Some(obj) = list.get(1) {
        eval_obj(obj, env)
    } else {
        Err(format!("Expect binding an Object to a variable in {:?}", object.loc()))
    }?;

    env.borrow_mut().set(name.as_str(), val);  // update the environment
    Ok(Object::Void { loc: None })
}

pub fn eval_if(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    // (if (boolean-expression) true-case false-case)
    let condition = list
        .first()
        .and_then(|object| {
            match object {
                Object::Bool { value, .. } => Some(value),
                // Object::List { value, .. } => eval_list(list, env)?
                _ => unimplemented!()
            }
        });

    if matches!(condition, Some(true)) {
        list.get(1)
    } else {
        list.get(2)
    }
    .map_or_else(|| Err(format!("follow-up action not found for the if-expression")), |o| eval_obj(o, env))
}

pub fn eval_function_definition(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    // (lambda (x y) (* x y))
    let params = list.first();
    let body = list.get(1);
}

pub fn eval_function_call(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    todo!()
}

pub fn eval_builtin_func(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    todo!()
}

pub fn eval_builtin_plus_func(list: &[Object], env: &Rc<RefCell<Environment>>) -> Result<Object, String> {
    todo!()
}