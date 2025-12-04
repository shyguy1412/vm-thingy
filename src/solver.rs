use std::io::{PipeReader, Read, Write};

pub fn solve(mut stdout: PipeReader, mut controller: impl Controller) {
    let mut things: Vec<String> = vec![];
    loop {
        let line = read_line(&mut stdout);
        println!("{line}");
        let is_thing = line.chars().nth(0).map(|c| c == '-').unwrap_or(false);
        if line == "What do you do?" {
            things.iter().for_each(|thing| controller.take_thing(thing));
        }
        if !is_thing {
            continue;
        }
        let thing: String = line.chars().skip(2).collect();
        things.push(thing);
    }
}

#[allow(dead_code)]
pub trait Controller {
    fn help(&mut self) -> ();
    fn look(&mut self) -> ();
    fn inv(&mut self) -> ();
    fn go_place(&mut self, place: &String) -> ();
    fn take_thing(&mut self, thing: &String) -> ();
    fn drop_thing(&mut self, thing: &String) -> ();
    fn use_thing(&mut self, thing: &String) -> ();
}

impl<T: Write> Controller for T {
    fn help(&mut self) -> () {
        let _ = self.write(format!("help\n").as_bytes());
    }

    fn look(&mut self) -> () {
        let _ = self.write(format!("look\n").as_bytes());
    }

    fn inv(&mut self) -> () {
        let _ = self.write(format!("inv\n").as_bytes());
    }

    fn go_place(&mut self, place: &String) -> () {
        let _ = self.write(format!("go {place}\n").as_bytes());
    }

    fn take_thing(&mut self, thing: &String) -> () {
        let _ = self.write(format!("take {thing}\n").as_bytes());
    }

    fn drop_thing(&mut self, thing: &String) -> () {
        let _ = self.write(format!("drop {thing}\n").as_bytes());
    }

    fn use_thing(&mut self, thing: &String) -> () {
        let _ = self.write(format!("use {thing}\n").as_bytes());
    }
}

fn read_line(reader: &mut dyn Read) -> String {
    let mut buf: Vec<char> = vec![];
    for byte in reader.bytes() {
        let Ok(byte) = byte else { unreachable!() };
        let char = byte as char;
        if char == '\n' {
            break;
        };
        buf.push(char);
    }
    buf.iter().collect()
}
