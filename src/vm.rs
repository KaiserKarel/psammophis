use crate::capabilities::{Capability, Privilege};
use crate::capabilities::{http, fs};
use std::io;

use rustpython_vm::PySettings;
use rustpython_vm::pyobject::PyRef;
use rustpython_vm::exceptions::PyBaseException;
use rustpython_vm::pyobject::{PyObjectRef, PyResult};
use rustpython_vm::function::{OptionalArg};
use rustpython_vm::VirtualMachine;
use rustpython_vm::stdlib::io::{OpenArgs};
use rustpython_vm::obj::objstr::{PyStringRef};
use rustpython_vm::scope::Scope;
use compile::compile;
use rustpython_vm::pyobject::ItemProtocol;

use thiserror::Error;

/// A wrapper around a RustPython VM;
pub struct Vm {
    pub vm: rustpython_vm::VirtualMachine,
    cap: Capability,
}

impl Vm {
    pub fn builder() -> VmBuilder {
        VmBuilder::default()
    }

    pub fn run_string(&self, scope: Scope, source: &str, source_path: String) -> PyResult {
        let code_obj = self.vm
            .compile(source, compile::Mode::Exec, source_path.clone())
            .map_err(|err| self.vm.new_syntax_error(&err))?;

        scope
            .globals
            .set_item("__file__", self.vm.new_str(source_path), &self.vm)?;
        self.vm.run_code_obj(code_obj, scope)
    }

    pub fn new_scope_with_builtins(&self) -> Scope {
        self.vm.new_scope_with_builtins()
    }
}

pub enum RuntimeError {
    SyntaxError(String),
    NameError(String),
}


#[derive(Default)]
pub struct VmBuilder {
    cap: Capability,
}

impl VmBuilder {

    /// Construct the VM with the given capabilities. If the builder already has a set of capabilities,
    /// this will error.
    pub fn with_capabilities(&mut self, caps: Capability) -> Result<&Self, BuildError> {
        self.cap = caps;
        Ok(self)
    }

    pub fn build(&self) -> Result<UninitializedVm, BuildError>{
        Ok(UninitializedVm{cap: self.cap.clone()})
    }

    pub fn fs_total(&self) -> &Self {
        todo!()
    }

    pub fn no_fs(&mut self) -> &Self {
        self.cap.fs = Privilege::None;
        self
    }

    /// Grants the largest set of Operations on files and dir in the specified path. (Read, Write,
    /// Edit, Delete, Chown etc).
    pub fn mount<P: Into<std::path::PathBuf>>(&self, path: P) -> &Self {
        todo!()
    }

    pub fn fs_capability(&mut self, cap: fs::Capability) -> Result<&mut Self, BuildError> {
        self.cap.add(cap)
            .map_err(|e| BuildError::IncompatibleCapabilities)
            .map(|_| self)
    }

    pub fn http_total(&self) -> &Self {
        todo!()
    }

    pub fn no_http(&mut self) -> &Self {
        self.cap.http = Privilege::None;
        self
    }

    pub fn http_capability(&self, cap: http::Capability) -> &Self {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("capabilities are already set")]
    CapabilitiesAlreadySet,

    #[error("no capabilities set")]
    NoCapbilitiesSet,

    #[error("incompatible capabilities")]
    IncompatibleCapabilities
}

use std::sync::Arc;

pub struct UninitializedVm {
    cap: Capability,
}

impl UninitializedVm {
    pub fn initialize(&self) -> Result<Vm, InitializationError>{
        let mut vm = rustpython_vm::VirtualMachine::new(no_initialize());
        // ensure that only files to which we have the capabilities can be opened, by removing
        // the standard open wrapper, and substituting our own.
        {
            let open_wrapper = crate::stdlib::io::io_open_with_capabilities(self.cap.clone().fs);
            vm.set_attr(&vm.builtins, "open", vm.ctx.new_function(open_wrapper))?;
        }

        {
            vm.set_attr(&vm.builtins, "print", vm.ctx.new_function(crate::stdlib::io::builtin_print_to_writer(io::stdout())))?;
        }
        Ok(Vm{vm, cap: self.cap.clone()})
    }
}

/// Creates a PySettings which causes the rustpython_vm::VirtualMachine::new function to not intialize
/// itself; which is necessary to prevent certain import errors; as the stdlib is bundled in the
/// actual binary.
fn no_initialize() -> PySettings {
    PySettings{
        initialization_parameter: rustpython_vm::InitParameter::NoInitialize,
        ..Default::default()
    }
}

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("PyException")]
    PyException
}

impl From<PyRef<PyBaseException>> for InitializationError {
    fn from(_: PyRef<PyBaseException>) -> Self {
        // TODO: obtain some useful error info from the exception.
        InitializationError::PyException
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_capabilities_build() {
        Vm::builder()
            .build().expect("no capability build should succeed")
            .initialize().expect("initialization of no capability vm should succeed");
    }

    #[test]
    fn test_fs_total_build() {
        Vm::builder()
            .fs_total()
            .build().expect("fs total capable should build")
            .initialize().expect("initialization of fs total vm should succeed");
    }

    #[test]
    fn test_no_fs_build() {
        Vm::builder()
            .no_fs()
            .build().expect("no fs capable should build")
            .initialize().expect("initialization of no fs vm should succeed");
    }

    #[test]
    fn test_mount_build() {
        Vm::builder()
            .mount("/a")
            .mount("/b")
            .build().expect("mount should build")
            .initialize().expect("initialization of mount should succeed");
    }

    #[test]
    fn test_fs_capability_build() {
        use crate::{capabilities, capabilities::fs::Operation};

        Vm::builder()
            .fs_capability(
                capabilities::Fs::dir(Operation::Read, "/").expect("should be dir"))
            .expect("add fs root capability")
            .fs_capability(
                capabilities::Fs::file(Operation::Write, "/swapfile").expect("should be file"))
            .expect("add fs swapfile capability")
            .build().expect("no fs capability should build")
            .initialize().expect("initialization of fs capability should succeed");
    }

    #[test]
    fn test_http_total_build() {
        Vm::builder()
            .http_total()
            .build().expect("http total capable should build")
            .initialize().expect("initialization of http total vm should succeed");
    }

    #[test]
    fn test_no_http_build() {
        Vm::builder()
            .no_http()
            .build().expect("no-http vm should build")
            .initialize().expect("initialization of no-http vm should succeed");
    }

    #[test]
    fn test_http_capability_build() {
        use crate::capabilities;
        Vm::builder()
            .http_capability(
                capabilities::Http::url("https://www.google.com/").expect("http capability should parse"))
            .http_capability(
                capabilities::Http::url("https://www.facebook.com/").expect("http capability should parse")
            )
            .build().expect("http capable should build")
            .initialize().expect("initialization of http vm should succeed");
    }
}