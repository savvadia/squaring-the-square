use std::fs::File;

mod squares;
mod exhaustive;
// mod concurrency_dummy;
mod coordinator;

const CONFIG_SIZE: usize = 151;

fn main() {
    //time the exhaustive search
    //set start time:
    //create a file for the output:
    let mut file = File::create("OUTPUT.txt").unwrap();
    let mut squares_placed = 0;
    // 50 reaches 152
    // 63 reaches glass size of 220. that's enough for our needs
    let MAX_GLASS_READ_ONLY: [i32; CONFIG_SIZE] = exhaustive::solve_glasses(50); 
    let mut max_square_for_glass: [i32; CONFIG_SIZE] =  [0; CONFIG_SIZE];

    for s in 1..CONFIG_SIZE {
        for g in (1..s).rev() {
            // println!("GLASS s: {} max square g: {}, MAX_GLASS_READ_ONLY[(s - g) as usize]: {} ", s, g, (MAX_GLASS_READ_ONLY[(s - g) as usize]));
            if g <= MAX_GLASS_READ_ONLY[s - (g as usize)].try_into().unwrap() {
                max_square_for_glass[s] = g as i32;
                // if g > 25 {
                    println!("max_square_for_glass square: {} max_square_for_glass: {}", s, g);
                // }
                break;
            } 
        }
    }

    let start = std::time::Instant::now();
    squares_placed = coordinator::coordinator_continuous(150, 150, &MAX_GLASS_READ_ONLY, &max_square_for_glass);
    let end = std::time::Instant::now();

    println!("Squares (total): {} in {} ms", squares_placed, (end- start).as_millis());
}