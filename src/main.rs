mod vm;

fn main() {
    const BINARY: &[u8; 60100] = include_bytes!("../challenge.bin");

    let mut state = vm::State::init_with(BINARY);

    while !state.done() {
        state.next()
    }

    println!("Terminated");
}
