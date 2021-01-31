use core::panic;
use std::time::Duration;

use crate::error::{RegisError, RegisErrorVariant};
use crate::source::{CanonicalPath, RelativePath};

use super::function::ProcedureVariant;
use super::native::ExternalCallContext;
use super::value::Value;
use super::FrameVariant;

pub fn print(arguments: &[Value], _: &mut ExternalCallContext) -> Result<Value, RegisError> {
    print!("{}", arguments.first().unwrap());
    Ok(Value::Null)
}

pub fn println(arguments: &[Value], _: &mut ExternalCallContext) -> Result<Value, RegisError> {
    println!("{}", arguments.first().unwrap());
    Ok(Value::Null)
}

pub fn len(arguments: &[Value], _: &mut ExternalCallContext) -> Result<Value, RegisError> {
    Ok(Value::Int(match arguments.first().unwrap() {
        Value::String(string) => string.len(),
        Value::List(list) => list.borrow().len(),
        Value::Object(object) => object.borrow().len(),
        other => {
            return Err(RegisError::new(
                None,
                RegisErrorVariant::TypeError {
                    message: format!("Cannot get @len() of type '{}'.", other.type_of()),
                },
            ))
        }
    } as i64))
}

pub fn import(
    arguments: &[Value],
    ExternalCallContext { interpreter }: &mut ExternalCallContext,
) -> Result<Value, RegisError> {
    let path = match arguments.first().unwrap() {
        Value::String(path) => path.to_string(),
        other => {
            return Err(RegisError::new(
                None,
                RegisErrorVariant::TypeError {
                    message: format!(
                        "Path passed to @import() must be a string. Got '{}'.",
                        other.type_of()
                    ),
                },
            ))
        }
    };

    let resolved = if let Some(relative) = RelativePath::from(&path) {
        let root = match interpreter.top_frame().unwrap().variant() {
            FrameVariant::Call(function) => match function.procedure() {
                ProcedureVariant::Internal(procedure) => procedure.environment().path().parent(),
                ProcedureVariant::External(..) => {
                    panic!("@import() cannot be called from external functions.")
                }
            },
            FrameVariant::Module(path) => path.parent(),
        };

        root.join(relative)
    } else if let Some(canonical) = CanonicalPath::from(&path) {
        Some(canonical)
    } else {
        None
    };

    if let Some(resolved) = resolved {
        interpreter.load_module(&resolved)?;
        let module = interpreter.modules.get(&resolved).unwrap();
        Ok(Value::Object(module.exports().clone()))
    } else {
        Err(RegisError::new(
            None,
            RegisErrorVariant::ModuleDoesNotExistError { path },
        ))
    }
}

pub fn sleep(arguments: &[Value], _: &mut ExternalCallContext) -> Result<Value, RegisError> {
    let seconds = match arguments.first().unwrap() {
        Value::Int(seconds) if *seconds >= 0 => *seconds as f64,
        Value::Float(seconds) if *seconds >= 0.0 => *seconds,
        other => {
            return Err(RegisError::new(
                None,
                RegisErrorVariant::TypeError {
                    message: format!(
                        "Number of seconds passed to @sleep() must be a positive int or float. Got '{}'.",
                        other.type_of()
                    ),
                },
            ))
        }
    };

    std::thread::sleep(Duration::from_secs_f64(seconds));
    Ok(Value::Null)
}
