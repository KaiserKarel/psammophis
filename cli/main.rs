use rustpython_vm::VirtualMachine;
use rustpython_vm::scope::Scope;
use rustpython_vm::pyobject::PyResult;
use compile::compile;
use rustpython_vm::pyobject::ItemProtocol;
use psammophis::vm::Vm;

fn main() -> anyhow::Result<()> {
    let vm = Vm::builder()
        .build()?
        .initialize()?;
    let scope = vm.new_scope_with_builtins();
    let result= vm.run_string(scope, "print('hello, world!')", "./hello.py".to_owned());
    return match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let args = e.args();
            dbg!(&e.typ().name);
            panic!("err")
        }
    }
}