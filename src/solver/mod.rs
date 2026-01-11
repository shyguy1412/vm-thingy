mod controller;
mod input;
mod state;

use controller::Controller;
use input::{Block, Input};
use std::{
    io::{Read, Write},
    process::exit,
};

#[allow(unused)]
pub fn solve<O: Read + 'static + std::marker::Send>(
    mut stdout: O,
    mut controller: impl Controller,
) {
    //crappy litte tee action
    let stdout = {
        let mut a = std::io::pipe().unwrap();

        let _ = std::thread::spawn(move || {
            loop {
                let buf = &mut [0];
                let _ = stdout.read(buf);
                a.1.write(buf);
                print!("{}", buf[0] as char);
            }
        }).join();
        a.0
    };
    let mut input: Input = stdout.into();

    // let mut things: Vec<String> = vec![];
    // let mut game_state: GameState = GameState {
    //     input: &mut input,
    //     controller: &mut controller,
    //     inventory: vec![],
    //     map_root: Location {
    //         id: "root".to_string(),
    //         things: vec![],
    //         paths: vec![],
    //     },
    // };

    //consume chars to the first location block
    // input.init();

    // let t = input.next();

    while let location = input.next() {
        let Some(location) = location else {
            panic!("Iterator returned NONE");
        };
        match location {
            Block::Location(location) => println!("{location}"),
        }
        // controller.go_place(&"south".to_string());
        // match line.as_str() {
        //     "Things of interest here:" => game_state.parse_things(),
        //     _ if line.ends_with("exits:") => game_state.parse_exits(),
        //     // "What do you do?" => game_state,
        //     _ => continue,
        // };

        // let is_thing = line.chars().nth(0).map(|c| c == '-').unwrap_or(false);
        // if line == "What do you do?" {
        //     things.iter().for_each(|thing| controller.take_thing(thing));
        // }
        // if !is_thing {
        //     continue;
        // }
        // let thing: String = line.chars().skip(2).collect();
        // things.push(thing);
    }
    println!("Current block is not a location!\nexiting...");
    exit(1)
}
