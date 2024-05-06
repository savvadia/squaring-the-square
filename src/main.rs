use crate::squares::Config;
use std::fs::File;

mod squares;
mod exhaustive;
mod concurrency_dummy;
mod coordinator;

fn main() {
    //time the exhaustive search
    //set start time:
    //create a file for the output:
    let mut file = File::create("OUTPUT.txt").unwrap();
    let start = std::time::Instant::now();
    let mut squares_placed = 0;
    // 50 reaches 152
    // 63 reaches glass size of 220. that's enough for our needs
    let MAX_GLASS_READ_ONLY: [i32; 256] = exhaustive::solve_glasses(50); 
    let mut max_square_for_glass: [i32; 256] =  [0; 256];

    for s in 1..256 {
        for g in (1..s).rev() {
            // println!("GLASS s: {} max square g: {}, MAX_GLASS_READ_ONLY[(s - g) as usize]: {} ", s, g, (MAX_GLASS_READ_ONLY[(s - g) as usize]));
            if g <= MAX_GLASS_READ_ONLY[(s - g) as usize] {
                max_square_for_glass[s as usize] = g;
                // if g > 25 {
                    println!("max_square_for_glass square: {} max_square_for_glass: {}", s, g);
                // }
                break;
            } 
        }
    }

    squares_placed = coordinator::coordinator_continuous(1, 150, &MAX_GLASS_READ_ONLY, &max_square_for_glass);

    /* 
     for s in 80..86 {

        let start_s = std::time::Instant::now();
        let size = s;
        let squares_placed_s = coordinator::Coordinator(size);
        squares_placed += squares_placed_s;
        //println!("Total squares placed: {}", squares_placed);
        let end_s = std::time::Instant::now();
        //println!("time {} {}", size, (end_s - start_s).as_millis());
        //println!("Squares/millis: {}", squares_placed_s / (end_s - start_s).as_millis());
    }*/
    
    let end = std::time::Instant::now();
    println!("{}", ((end- start).as_millis()));
    println!("Squares (total): {}", squares_placed);
}