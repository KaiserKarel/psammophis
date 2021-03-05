mod common;

#[test]
fn hello_world() {
    let vm = common::Vm::new();
    vm.run_file("snippets/hello_world.py")
}

#[test]
fn open_file() {
    let vm = common::Vm::new();
    vm.run_file("snippets/open_file.py")
}