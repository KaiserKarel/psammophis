use std::collections::HashSet;
use std::io::prelude::*;
use rustpython_vm::pyobject::{PyObjectRef, PyResult};
use rustpython_vm::function::{OptionalArg, IntoPyNativeFunc, PyNativeFunc, Args, FromArgs};
use rustpython_vm::VirtualMachine;
use pyderive::FromArgs;
use rustpython_vm::stdlib::io::{OpenArgs};
use rustpython_vm::obj::{objstr::{PyStringRef}, objbool::IntoPyBool};
use crate::capabilities::{Privilege, fs::Capability};

pub fn io_open_with_capabilities(capabilities: Privilege<HashSet<Capability>>) -> impl Fn(PyObjectRef, OptionalArg<PyStringRef>, OpenArgs, &VirtualMachine) -> PyResult {
    let open_fn = move |file: PyObjectRef, mode: OptionalArg<PyStringRef>, opts: OpenArgs, vm: &VirtualMachine| {
        println!("called open wrapper");
        match &capabilities {
            Privilege::None => {
                return Err(vm.new_value_error("Filesystem capabilities are not enabled.".to_owned()));
            }
            Privilege::Total => {
                let mode = mode.as_ref().into_option().map(|s| s.as_str());
                return rustpython_vm::stdlib::io::io_open(file, mode, opts, vm)
            },
            Privilege::Limited(caps) => {
                todo!("implement limited caps")
                // let mode = mode.as_ref().into_option().map(|s| s.as_str());
                // let file = file.as_ref().
                //
                // caps.into_iter().for_each(|cap| {
                //     match cap.has_access(file) {
                //         Ok(has) =>
                //         Err(e)  =>
                //     }
                //     ()
                // })
                // python::stdlib::io::io_open(file, mode, opts, vm)
            }
        }
    };
    open_fn
}

#[derive(Debug, Default, FromArgs)]
pub struct PrintOptions {
    #[pyarg(keyword_only, default = "None")]
    sep: Option<PyStringRef>,
    #[pyarg(keyword_only, default = "None")]
    end: Option<PyStringRef>,
    #[pyarg(keyword_only, default = "IntoPyBool::FALSE")]
    flush: IntoPyBool,
    #[pyarg(keyword_only, default = "None")]
    file: Option<PyObjectRef>,
}

use std::sync::Arc;
use std::sync::Mutex;

pub fn builtin_print_to_writer<w: Write>(mut writer: w) -> impl Fn(Args, PrintOptions, &VirtualMachine) -> PyResult<()> {
    let writer = Arc::new(Mutex::new(writer));

    let print_fn = move |args: Args, opts: PrintOptions, vm: &VirtualMachine| {
        let mut writer = writer.lock().unwrap();

        let mut first = true;
        for object in args {
            if first {
                first = false;
            } else {
                writer.write(" ".as_bytes());
            }
            writer.write( vm.to_str(&object)?.as_str().as_bytes());
        }
        Ok(())
    };
    print_fn
}