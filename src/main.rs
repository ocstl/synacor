extern crate synacor;

fn main() {
    let mut test = synacor::VirtualMachine::new();
    test.load_program("challenge.bin");
    let result = test.execute_program();

    println!("{:?}", result);
}
