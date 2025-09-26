//! External function implementations for plugin loader v2
//!
//! This module contains all `env.*` external function implementations
//! that were previously in a large switch statement in loader.rs

use crate::bid::{BidError, BidResult};
use crate::box_trait::{NyashBox, StringBox, VoidBox};
use crate::boxes::result::NyashResultBox;
use crate::boxes::future::FutureBox;
use crate::boxes::token_box::TokenBox;
use crate::runtime::modules_registry;
use crate::runtime::global_hooks;

/// Handle external function calls from the runtime
pub fn extern_call(
    iface_name: &str,
    method_name: &str,
    args: &[Box<dyn NyashBox>],
) -> BidResult<Option<Box<dyn NyashBox>>> {
    match iface_name {
        "env.console" => handle_console(method_name, args),
        "env.result" => handle_result(method_name, args),
        "env.modules" => handle_modules(method_name, args),
        "env.task" => handle_task(method_name, args),
        "env.debug" => handle_debug(method_name, args),
        "env.runtime" => handle_runtime(method_name, args),
        "env.future" => handle_future(method_name, args),
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.console.* methods
fn handle_console(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "log" => {
            let trace = std::env::var("NYASH_CONSOLE_TRACE").ok().as_deref() == Some("1");
            for a in args {
                let s = a.to_string_box().value;
                if trace {
                    eprintln!("[console.trace] len={} text=<{:.64}>", s.len(), s);
                }
                println!("{}", s);
            }
            Ok(None)
        }
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.result.* methods
fn handle_result(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "ok" => {
            // Wrap the first argument as Result.Ok; if missing, use Void
            let v = args
                .get(0)
                .map(|b| b.clone_box())
                .unwrap_or_else(|| Box::new(VoidBox::new()));
            Ok(Some(Box::new(NyashResultBox::new_ok(v))))
        }
        "err" => {
            // Wrap the first argument as Result.Err; if missing, synthesize a StringBox("Error")
            let e: Box<dyn NyashBox> = args
                .get(0)
                .map(|b| b.clone_box())
                .unwrap_or_else(|| Box::new(StringBox::new("Error")));
            Ok(Some(Box::new(NyashResultBox::new_err(e))))
        }
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.modules.* methods
fn handle_modules(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "set" => {
            if args.len() >= 2 {
                let key = args[0].to_string_box().value;
                let val = args[1].clone_box();
                modules_registry::set(key, val);
            }
            Ok(None)
        }
        "get" => {
            if let Some(k) = args.get(0) {
                let key = k.to_string_box().value;
                if let Some(v) = modules_registry::get(&key) {
                    return Ok(Some(v));
                }
            }
            Ok(Some(Box::new(VoidBox::new())))
        }
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.task.* methods
fn handle_task(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "cancelCurrent" => {
            let tok = global_hooks::current_group_token();
            tok.cancel();
            Ok(None)
        }
        "currentToken" => {
            let tok = global_hooks::current_group_token();
            let tb = TokenBox::from_token(tok);
            Ok(Some(Box::new(tb)))
        }
        "spawn" => handle_task_spawn(args),
        "wait" => handle_task_wait(args),
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.task.spawn method
fn handle_task_spawn(args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some(b) = args.get(0) {
        // The plugin loader originally included additional spawn logic,
        // but we keep the simplified version here for now
        // TODO: Implement full task spawning logic
        Ok(Some(b.clone_box()))
    } else {
        Ok(None)
    }
}

/// Handle env.task.wait method
fn handle_task_wait(_args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    // Task wait is not yet implemented in the extracted module
    // This functionality will be added when properly integrating with future system
    Err(BidError::PluginError)
}

/// Handle env.debug.* methods
fn handle_debug(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "trace" => {
            if std::env::var("NYASH_DEBUG_TRACE").ok().as_deref() == Some("1") {
                for a in args {
                    eprintln!("[debug.trace] {}", a.to_string_box().value);
                }
            }
            Ok(None)
        }
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.runtime.* methods
fn handle_runtime(method_name: &str, _args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "checkpoint" => {
            if crate::config::env::runtime_checkpoint_trace() {
                eprintln!("[runtime.checkpoint] reached");
            }
            global_hooks::safepoint_and_poll();
            Ok(None)
        }
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.future.* methods
fn handle_future(method_name: &str, args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    match method_name {
        "new" | "birth" => {
            let fut = FutureBox::new();
            if let Some(v) = args.get(0) {
                fut.set_result(v.clone_box());
            }
            Ok(Some(Box::new(fut)))
        }
        "set" => {
            if args.len() >= 2 {
                if let Some(fut) = args[0]
                    .as_any()
                    .downcast_ref::<FutureBox>()
                {
                    fut.set_result(args[1].clone_box());
                }
            }
            Ok(None)
        }
        "await" => handle_future_await(args),
        _ => Err(BidError::PluginError),
    }
}

/// Handle env.future.await method
fn handle_future_await(args: &[Box<dyn NyashBox>]) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some(arg) = args.get(0) {
        if let Some(fut) = arg
            .as_any()
            .downcast_ref::<crate::boxes::future::FutureBox>()
        {
            let max_ms: u64 = crate::config::env::await_max_ms();
            let start = std::time::Instant::now();
            let mut spins = 0usize;

            while !fut.ready() {
                global_hooks::safepoint_and_poll();
                std::thread::yield_now();
                spins += 1;

                if spins % 1024 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }

                if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
                    let err = StringBox::new("Timeout");
                    return Ok(Some(Box::new(NyashResultBox::new_err(Box::new(err)))));
                }
            }

            return match fut.wait_and_get() {
                Ok(v) => Ok(Some(Box::new(NyashResultBox::new_ok(v)))),
                Err(e) => {
                    let err = StringBox::new(format!("Error: {}", e));
                    Ok(Some(Box::new(NyashResultBox::new_err(Box::new(err)))))
                }
            };
        } else {
            return Ok(Some(Box::new(NyashResultBox::new_ok(arg.clone_box()))));
        }
    }

    Ok(Some(Box::new(NyashResultBox::new_err(Box::new(
        StringBox::new("InvalidArgs"),
    )))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_log() {
        let args = vec![Box::new(StringBox::new("test")) as Box<dyn NyashBox>];
        let result = handle_console("log", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_result_ok() {
        let args = vec![Box::new(StringBox::new("success")) as Box<dyn NyashBox>];
        let result = handle_result("ok", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_result_err() {
        let args = vec![];
        let result = handle_result("err", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_unknown_interface() {
        let args = vec![];
        let result = extern_call("unknown.interface", "method", &args);
        assert!(matches!(result, Err(BidError::PluginError)));
    }
}
