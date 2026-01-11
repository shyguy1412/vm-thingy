use crate::solver::solve;

mod solver;
mod vm;

fn main() {
    const BINARY: &[u8; 60100] = include_bytes!("../challenge.bin");
    let (mut state, (stdout, stdin)) = vm::State::init_with(BINARY);

    let vm_thread = std::thread::spawn(move || {
        loop {
            state = state.next();
            state.done().then(|| state.reset());
        }
    });

    // let _ = std::thread::spawn(move || solve(stdout, stdin));
    let _ = std::thread::spawn(move || play(stdout, stdin));

    let _ = vm_thread.join();
    println!("Terminated");
}

#[allow(unused)]
fn play<
    I: std::io::Write + 'static + std::marker::Send,
    O: std::io::Read + 'static + std::marker::Send,
>(
    mut stdout: O,
    mut stdin: I,
) {
    let read_thread = std::thread::spawn(move || {
        loop {
            let buf = &mut [0];
            let _ = stdout.read(buf);
            print!("{}", buf[0] as char);
        }
    });
    let write_thread = std::thread::spawn(move || {
        loop {
            let buf = &mut [0];
            let Ok(_) = std::io::Read::read(&mut std::io::stdin(), buf) else {
                panic!("Can not read from stdin")
            };
            let _ = stdin.write(buf);
        }
    });
    read_thread.join();
    write_thread.join();

    let mut thingStruct = MyStruct { a: 0 };
    thing(&mut thingStruct);
    thingStruct;
}

struct MyStruct {
    a: i32,
}

fn thing(strc: &mut MyStruct) {
    strc.a = 5;
}
