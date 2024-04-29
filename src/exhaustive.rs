use crate::squares::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, Once};
use crate::coordinator::Message;
use std::io::Write;

//import config
use crate::squares::Config;
use std::fs::{self, File};



pub fn solve (size: Integer, max_glass: [i32; 256]) -> (){
    //println!{"Solving for size: {}", size};
    let mut config = Config::new(size, max_glass); //Creates the necessary starting plate.
    //println!("Config: {:?}", config);
    decompose(&mut config, 1);
}

pub fn solve_cc(send : &Sender<Message>, size: Integer, max_glass: [i32; 256]) ->(){
    //println!{"Solving for size: {}", size};
    let mut config = Config::new(size, max_glass); //Creates the necessary starting plate.
    //println!("Config: {:?}", config);
    double_nest_init(send, &mut config);
    // init_bottom_corners(send, &mut config);
    //initial_SOLVE_decompose_cc(send, rcv, &mut config);
}

pub fn solve_glasses(max_glass: usize) -> ([i32; 256]) {
    let mut check_glass_size: i32 = 0;
    let mut max_glass_precalc: [i32; 256] = [0; 256];
    for glass in 1..max_glass {
        let start = std::time::Instant::now();
        check_glass_size = glass as i32;
        let mut config = Config::new(256, max_glass_precalc);
        config.plates[1].width = glass as i32;
        config.plates[2].width = (256 - glass) as i32;
        let mut i = 1;
        config.net_squares = 2;
        decompose_glasses(&mut config, 1, check_glass_size, &mut max_glass_precalc);
        let finish = std::time::Instant::now();
        println!("==>    Time for glasses of size {}: {} ms -> max glass {}", 
            glass, (finish - start).as_millis(), max_glass_precalc[glass]);
    }

    check_glass_size = 0;
    for glass in max_glass..256 {
        max_glass_precalc[glass] = 256;
    }

    return max_glass_precalc;
}

fn next_plate(config: &mut Config) -> () { //find smallest delimited plate, and decompose it
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    let mut p_min_i : usize = 0;
    //find the minimum delimited plate
    //if, in the meantime, we identify that there is only three plates, we have found a square or rectangle and should return them
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height && plate.width < l_min{
            l_min = plate.width;
            p_min_i = i;
        }
    }
    //print width and index of minimum delimited plate
    // println!("[next_plate] l_min: {}, p_min_i: {}", l_min, p_min_i);
    if l_min == config.size {
        let mut f = File::options().append(true).open("OUTPUT.txt").unwrap();
        if config.plates[p_min_i].height == config.size {
            //we have found a square
            //return the square and 
            //create a string of ("Found a square, width: {}, height: {}, squares: ", config.size, config.plates[p_min_i].height + config.squares:
            let mut s = String::new();
            s += "S, W: ";
            s += &config.size.to_string();
            s += ", H: ";
            s += &config.plates[p_min_i].height.to_string();
            s += ", ORDER: ";
            s += &config.order().to_string();
            s += ", SET: ";
            s += &config.squares_to_string();
            writeln!(&mut f, "{}", s).unwrap();
        }
        else {
            //we have found a rectangle
            //return the rectangle}
            //create a string of ""Found a rectangle, width: {}, height: {}, squares: ", config.size, config.plates[p_min_i].height, );
            // config.print_squares();
            let mut s = String::new();
            s += "R, W: ";
            s += &config.size.to_string();
            s += ", H: ";
            s += &config.plates[p_min_i].height.to_string();
            s += ", ORDER: ";
            s += &config.order().to_string();
            s += ", SET: ";
            s += &config.squares_to_string();
            writeln!(&mut f, "{}", s).unwrap();
            //println!("continuing search...");
            decompose(config, p_min_i);
        }
    }
    else {
        decompose(config, p_min_i);
    }
}

fn next_plate_cc(send : &Sender<Message>, rcv: &Receiver<()>, config: &mut Config) -> () { //find smallest delimited plate, and decompose it
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    let mut p_min_i : usize = 0;
    //find the minimum delimited plate
    //if, in the meantime, we identify that there is only three plates, we have found a square or rectangle and should return them
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height && plate.width < l_min{
            l_min = plate.width;
            p_min_i = i;
        }
    }
    //print width and index of minimum delimited plate
    ////eprintln!("l_min: {}, p_min_i: {}", l_min, p_min_i);
    if l_min == config.size {
        if config.plates[p_min_i].height == config.size {
            //we have found a square
            //return the square and 
            //println!("Found a square: {:?}", config);
        }
        else {
            //we have found a rectangle
            //return the rectangle}
            //println!("Found a rectangle: {:?}", config);
            //println!("continuing search...");
            decompose_cc(send, rcv, config, p_min_i);
        }
    }
    else {
        decompose_cc(send, rcv, config, p_min_i);
    }
}

fn next_plate_for_glasses(config: &mut Config, check_glass_size: Integer, mut max_glass_precalc: &mut [i32; 256]) -> () { //find smallest delimited plate, and decompose it
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    let mut p_min_i : usize = 0;
    //find the minimum delimited plate
    //if, in the meantime, we identify that there is only three plates, we have found a square or rectangle and should return them
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height && plate.width < l_min{
            l_min = plate.width;
            p_min_i = i;
        }
    }

    decompose_glasses(config, p_min_i, check_glass_size, max_glass_precalc);
}
pub fn decompose_glasses(mut config: &mut Config, plate_id: usize, check_glass_size: Integer, mut max_glass_precalc: &mut [i32; 256]) -> () { 
    //given a plate, decompose it by adding squares, then select the next plate if the plates change
    // if filling the plate with a square does not make the height greater than the size, add the square and then next plate
    // println!("decomposing, config: {}, plate_id: {}, net_squares: {}", config, plate_id, config.net_squares);
    if plate_id > 0 && plate_id < config.plates.len() - 1 {
        let glass_width = check_glass_size;
        let old_glass = max_glass_precalc[glass_width as usize];
        let new_glass = 
            std::cmp::min(config.plates[plate_id].height,
            std::cmp::min(config.plates[1].height,
                        config.plates[(config.plates.len() - 2) as usize].height));
        if new_glass > old_glass {
            // println!("max_glass_precalc[{}] {} -> {} min({},{},{})", 
            //     check_glass_size,
            //     old_glass, new_glass, 
            //     config.plates[1].height,
            //     config.plates[plate_id].height,
            //     config.plates[(config.plates.len() - 2) as usize].height);
            max_glass_precalc[glass_width as usize] = new_glass;
        }
    } 

    if config.can_use(config.plates[plate_id].width) && 
       config.plates[plate_id].height + config.plates[plate_id].width <= config.size
        // &&  
    //    (if plate_id == config.plates.len()-2 {config.plates[plate_id].width >= 5} else {true})
       {
        config.net_squares += 1;
        //eprintln!("a+ {}", config);
        
        let mut config_backup = config.clone();
        config.vertical_extension(plate_id);
        //println!("{:?}", config);
        next_plate_for_glasses(&mut config, check_glass_size, max_glass_precalc);
        //undo it
        config_backup.net_squares = config.net_squares;
        // println!("v-");
        *config = config_backup;
        
    }
    else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    let mut min_for_corner = 1;
    if config.plates[plate_id - 1].height - config.plates[plate_id].height < 
       config.plates[plate_id].width && config.can_use(config.plates[plate_id - 1].height - config.plates[plate_id].height) {
        config.net_squares += 1;
        //eprintln!("b+ {}", config);
        //let mut config_backup = config.clone();

        config.horizontal_extension(plate_id);
        //println!("{:?}", config);
        decompose_glasses(config, plate_id, check_glass_size, max_glass_precalc);
        //remove the square
        config.reverse_horizontal_extension(plate_id);
        //config_backup.net_squares = config.net_squares;
        //*config = config_backup;
        //eprintln!("b- {}", config);

    }

    for s in 1..(std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height)+1) {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && s != config.plates[plate_id-1].height - config.plates[plate_id].height{
            config.net_squares += 1;
            //print number of plates:        
            //let mut config_backup = config.clone();

            config.add_square_quick(s, plate_id);
            //println!("{:?}", config);
            next_plate_for_glasses(&mut config, check_glass_size, max_glass_precalc);
            config.remove_square(plate_id);
        }
        else{
            ////eprintln!("{} is not a valid square size", s)
        }
    }

}

pub fn decompose(mut config: &mut Config, plate_id: usize) -> () { //given a plate, decompose it by adding squares, then select the next plate if the plates change
    // if filling the plate with a square does not make the height greater than the size, add the square and then next plate
    // println!("decomposing, config: {}, plate_id: {}, net_squares: {}", config, plate_id, config.net_squares);
    if plate_id > 0 && plate_id < config.plates.len() - 1 {
        let glass_depth = std::cmp::min(
            config.plates[plate_id - 1].height - config.plates[plate_id].height,
            config.plates[plate_id + 1].height - config.plates[plate_id].height,
        );
        let glass_width = config.plates[plate_id].width;
        if config.max_glass[glass_width as usize] -
           (if config.can_use(glass_width) {0} else {glass_width}) < glass_depth {
            if glass_width > 35 {
                println!(
                    "skipping GLASS glass_depth={} w={} {}",
                    glass_depth,
                    config.plates[plate_id].width,
                    if config.can_use(glass_width) {""} else {"(w used)"}
                );
                }
            return;
        }
    } 

    let square = config.plates[plate_id].width;
    if config.can_use(square) && 
       config.plates[plate_id].height + square <= config.size
        // &&  
    //    (if plate_id == config.plates.len()-2 {square >= 5} else {true})
       {
        config.net_squares += 1;
        let orig_left_plate_width = config.plates[plate_id - 1].width;
        let new_plate_id = config.vertical_extension(plate_id);
        //println!("{:?}", config);
        next_plate(&mut config);
        //undo it
        if new_plate_id == plate_id {
            // we were potentially merging with right plate only
            config.reverse_vertical_extension(plate_id, square, 0);
        } else {
            // we were mergeing with left plate and potentially with right plate
            config.reverse_vertical_extension(new_plate_id, square, orig_left_plate_width);
        }
    }
    else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    let mut min_for_corner = 1;
    if config.plates[plate_id - 1].height - config.plates[plate_id].height < 
       config.plates[plate_id].width && config.can_use(config.plates[plate_id - 1].height - config.plates[plate_id].height) {
        config.net_squares += 1;
        //eprintln!("b+ {}", config);
        //let mut config_backup = config.clone();

        config.horizontal_extension(plate_id);
        //println!("{:?}", config);
        decompose(config, plate_id);
        //remove the square
        config.reverse_horizontal_extension(plate_id);
        //config_backup.net_squares = config.net_squares;
        //*config = config_backup;
        //eprintln!("b- {}", config);

    }
    else {
        //////eprintln!("b.");
        let corner_to_box: [i32; 21] = [0, 1, 2, 3, 3, 3, 4, 4, 5, 5, 5, 5, 5, 6, 7, 7, 7, 7, 7, 7, 8];
        let corner_size = std::cmp::min(config.plates[plate_id - 1].height - config.plates[plate_id].height, config.plates[plate_id].width);
        min_for_corner = if corner_size < 21 { corner_to_box[corner_size as usize] } else { 8 };
    }
    // iterate over all possible square sizes that can be added to the bottom left corner.
    //println!("{} to {}", 2, std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height) + 1);

    // let min = if (config.plates[plate_id].height == 0 || plate_id == 1) {5} else {2};
    // let min = if (config.plates[plate_id].height == 0 || plate_id == 1) {std::cmp::max(5, min_for_corner)} else {min_for_corner};
    let min = min_for_corner;
    // println!("min_for_corner={} ", min_for_corner); 
    for s in min..(std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height)+1) {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && s != config.plates[plate_id-1].height - config.plates[plate_id].height{
            config.net_squares += 1;
            config.add_square_quick(s, plate_id);
            //println!("{:?}", config);
            next_plate(&mut config);
            config.remove_square(plate_id);
            //println!("{} checked", s);
        }
        else{
            ////eprintln!("{} is not a valid square size", s)
            // println!("min_for_corner={} ", min_for_corner);        
        }
    }

}

fn decompose_cc(send : &Sender<Message>, rcv : &Receiver<()>, mut config: &mut Config, plate_id: usize) -> (){
    /* reimplement if needed
    match rcv.try_recv() {
        Ok(_) => {
            //println!("Work unit produced: {}, plate: {}", config, plate_id);
            send.send(Message::WorkUnit((config.clone(), plate_id))).unwrap();
        },
        Err(_) => {

            // if filling the plate with a square does not make the height greater than the size, add the square and then next plate
            if config.can_use(config.plates[plate_id].width) && config.plates[plate_id].height + config.plates[plate_id].width <= config.size && (if plate_id == config.plates.len()-2 {config.plates[plate_id].width >= 5} else {true}){
                //eprintln!("a+ {}", config);
                let config_backup = config.clone();
                config.vertical_extension(plate_id);
                //println!("{:?}", config);
                next_plate_cc(send, rcv, &mut config);
                //undo it
                *config = config_backup;
            }
            else{
                ////eprintln!("a.");
            }
            // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
            if config.can_use(config.plates[plate_id - 1].height - config.plates[plate_id].height) && config.plates[plate_id - 1].height - config.plates[plate_id].height < config.plates[plate_id].width{
                //eprintln!("b+ {}", config);

                config.horizontal_extension(plate_id);
                //println!("{:?}", config);
                decompose_cc(send, rcv, config, plate_id);
                //remove the square
                config.reverse_horizontal_extension(plate_id);
                //eprintln!("b- {}", config);

            }
            else{
                //////eprintln!("b.");
            }
            // iterate over all possible square sizes that can be added to the bottom left corner.
            //println!("{} to {}", 2, std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height) + 1);
        
            let min = if (config.plates[plate_id].height == 0 || plate_id == 1) {5} else {2};

            for s in min..(std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height)+1) {
                // if the square can be added to the bottom left corner, add it and then decompose the new plate)
                if config.can_use(s) && s != config.plates[plate_id-1].height - config.plates[plate_id].height{
                    //print number of plates:
                    config.add_square_quick(s, plate_id);
                    //println!("{:?}", config);
                    decompose_cc(send, rcv, config, plate_id + 1);
                    //remove the square
                    config.remove_square(plate_id);
                    //println!("{} checked", s);
                }
                else{
                    ////eprintln!("{} is not a valid square size", s)
                }
            }
        }
    }
    */
}

pub fn initial_decompose_cc(send : &Sender<Message>, rcv : &Receiver<()>, config: &mut Config, plate_id: usize) -> (){

    { 
    //given a plate, decompose it by adding squares, then select the next plate if the plates change
    // if filling the plate with a square does not make the height greater than the size, add the square and then next plate

    if config.can_use(config.plates[plate_id].width) && config.plates[plate_id].height + config.plates[plate_id].width <= config.size && (if plate_id == config.plates.len()-2 {config.plates[plate_id].width >= 5} else {true}){
        //eprintln!("a+ {}", config);
        let config_backup = config.clone();
        config.vertical_extension(plate_id);
        next_plate_cc(send, rcv, config);
        //undo it
        *config = config_backup;
        //eprintln!("a- {}", config);
    }
    else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    if config.can_use(config.plates[plate_id - 1].height - config.plates[plate_id].height) && config.plates[plate_id - 1].height - config.plates[plate_id].height < config.plates[plate_id].width{
        //eprintln!("b+ {}", config);
        config.horizontal_extension(plate_id);
        decompose_cc(send, rcv, config, plate_id);
        //remove the square
        config.reverse_horizontal_extension(plate_id);
        //eprintln!("b- {}", config);
    }
    else{
        ////eprintln!("b.");
    }
    // iterate over all possible square sizes that can be added to the bottom left corner.
    //println!("{} to {}", 2, std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height) + 1);
    
    let min = if (config.plates[plate_id].height == 0 || plate_id == 1) {5} else {2};

    for s in min..(std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height)+1) {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && s != config.plates[plate_id-1].height - config.plates[plate_id].height{
            //print number of plates:
            config.add_square_quick(s, plate_id);
            decompose_cc(send, rcv, config, plate_id + 1);
            //remove the square
            config.remove_square(plate_id);
            //eprintln!("{} checked", s);
        }
        else{
        }
    }
    //println!("Initial decompose finished");

}
}

pub fn initial_SOLVE_decompose_cc(send : &Sender<Message>, rcv : &Receiver<()>, config: &mut Config) -> (){

    { 
    
    // iterate over all possible square sizes that can be added to the bottom left corner.
    //println!("{} to {}", 2, std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height) + 1);
    for s in (9..(config.plates[1].width/2 +1)) {
        config.add_square_quick(s, 1);
        send.send(Message::WorkUnit((config.clone(), 2))).unwrap();
        config.remove_square(1);
        //eprintln!("{} checked", s);
    }
    //println!("Initial decompose finished");

}
}

pub fn double_nest_init(send: &Sender<Message>,  config: &mut Config) -> () {
    let mut i = 1;
    //start time:
    //let mut start: std::time::Instant = std::time::Instant::now();
    config.net_squares = 2;
    for s1 in (9..(config.plates[1].width/2+1)){
        config.add_square_quick(s1, 1);
            for s2 in 5..((config.plates[2].width - 9) + 1){
                if s2 != s1 {
                    config.add_square_quick(s2, 2);
                        if s2 > s1 && s1 < config.plates[3].width {
                            send.send(Message::WorkUnit((config.clone(), 1))).unwrap();
                        }
                        else {
                            send.send(Message::WorkUnit((config.clone(), 3))).unwrap();
                        }
                    i+=1;
                    config.remove_square(2);
                }
            }
            //MY BOUND, to implement comment out 
            /*
            let s2 = config.plates[2].width;
            if s2 != s1 {
                let mut c = config.clone();
                c.vertical_extension(2);
                    if s1 > s2 {
                        send.send(Message::WorkUnit((c, 2))).unwrap();
                    }
                    else{
                        send.send(Message::WorkUnit((c, 1))).unwrap();
                    }
                    i+=1;
            }*/
            

        config.remove_square(1);
    }
    //end time:
    //println!("{} work units produced", i);
    ////let end = std::time::Instant::now();
    //println!("Time elapsed producing units: {}ms", (end - start).as_millis());
}


// pub fn init_bottom_corners(send: &Sender<Message>,  config: &mut Config) -> () {
//     let mut i = 1;
//     //start time:
//     //let mut start: std::time::Instant = std::time::Instant::now();
//     config.net_squares = 2;
//     for s1 in (9..(config.size-8)){
//         // println!("init_bottom_corners - start, config: {}, net_squares: {}", config, config.net_squares);
//         config.add_square_quick(s1, 1);
//         for s2 in 8..std::cmp::min(s1, config.plates[2].width){
//                 config.add_square_quick_right(s2, 2);
//                 println!("init_bottom_corners - send, config: {}, net_squares: {}", config, config.net_squares);
//                 send.send(Message::WorkUnit((config.clone(), 2))).unwrap();
//                 i+=1;
//                 config.remove_square_right(3);
//         }
//         config.remove_square(1);
//     }
// }