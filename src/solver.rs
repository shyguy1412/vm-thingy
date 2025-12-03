use std::io::{PipeReader, PipeWriter, Read};

enum Doable {
    ThingOfInterest(String),
    PointOfInterest(String),
}

pub fn solve(mut stdout: PipeReader, mut stdin: PipeWriter) {
    loop {
        let char = read_character(&mut stdout).unwrap();
        print!("{}", char)
    }
}

fn read_character(reader: &mut dyn Read) -> Option<char> {

    let mut buf = [0u8];
    let _ = reader.read(&mut buf);

    Some(buf[0] as char)
}
