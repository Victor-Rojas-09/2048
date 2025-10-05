#![allow(unused)]

pub mod board;
pub mod eval;
pub mod search;

use std::{
    thread,
    time::{Duration, Instant},
};

use board::*;

fn main() {
    let init = PlayableBoard::init();

    println!("Starting game!");

    play(init);
}

pub fn play(init: PlayableBoard) {
    let mut num_moves = 0;
    let mut cur = init;
    loop {
        println!("{cur}");
        // TO REMOVE eventually: slow down the program to make it easier to follow
        thread::sleep(Duration::from_millis(300));

        let start_action_selection = Instant::now();
        let action = match search::select_action(cur) {
            Some(action) => action,
            None => {
                println!("GAME OVER!");
                println!("Num moves: {num_moves}");
                return;
            }
        };
        // print the selected action, together with the time by the `select_action` function
        println!(
            "\n[{:.2}ms] Playing action {action:?}:",
            start_action_selection.elapsed().as_secs_f64() * 1000.0
        );
        let played = cur.apply(action).expect("invalid action");
        num_moves += 1;
        println!("{played}");

        println!("Adding random tile:");

        cur = played.with_random_tile();
    }
}
