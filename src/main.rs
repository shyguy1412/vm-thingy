use crate::solver::solve;

mod solver;
mod vm;

fn main() {
    const BINARY: &[u8; 60100] = include_bytes!("../challenge.bin");
    let (mut state, (stdout, stdin)) = vm::State::init_with(BINARY);

    let vm_thread = std::thread::spawn(move || {
        loop {
            state.next();
            state.done().then(||state.reset());
        }
    });

    let _ = std::thread::spawn(move || solve(stdout, stdin));

    let _ = vm_thread.join();
    println!("Terminated");
}
