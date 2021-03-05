use psammophis;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

pub struct Vm {
    vm: psammophis::Vm,
}

impl Vm {
    pub fn new() -> Vm {
        let vm = psammophis::Vm::builder()
            .build().expect("no capability build should succeed")
            .initialize().expect("initialization of no capability vm should succeed");
        Vm{vm}
    }

    pub fn run_file(&self, file: &str) {
        let name = file;
        let mut file = File::open(file).expect("opening run_file");
        let mut code = String::new();
        file.read_to_string(&mut code);
        let scope = self.vm.new_scope_with_builtins();
        let result= self.vm.run_string(scope, code.as_ref(), name.to_owned()).expect("running file");
    }
}