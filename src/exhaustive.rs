use crate::squares::*;
use std::sync::mpsc::{Receiver, Sender};
use crate::coordinator::Message;
use std::io::Write;

//import config
use crate::squares::Config;
use crate::CONFIG_SIZE;
use std::fs::{File};

pub fn solve_cc(send : &Sender<Message>, first: Integer, size: Integer, max_glass: [i32; CONFIG_SIZE], max_square_for_glass: [i32; CONFIG_SIZE]) ->(){
    //println!{"Solving for size: {}", size};
    let mut config = Config::new(Some(send.clone()), size, max_glass, max_square_for_glass); //Creates the necessary starting plate.
    //println!("Config: {:?}", config);
    double_nest_init(send, first, &mut config);
 }

pub fn solve_glasses(max_glass: usize) -> [i32; CONFIG_SIZE] {
    let mut check_glass_size: i32 = 0;
    let mut max_glass_precalc: [i32; CONFIG_SIZE] = [0; CONFIG_SIZE];
    for glass in 1..max_glass {
        let start = std::time::Instant::now();
        check_glass_size = glass as i32;
        let mut config = Config::new(None::<Sender<Message>>, CONFIG_SIZE as Integer, max_glass_precalc, max_glass_precalc);
        config.plates[1].width = glass as i32;
        config.plates[2].width = (CONFIG_SIZE - glass) as i32;
        config.net_squares = 2;
        decompose_glasses(&mut config, 1, check_glass_size, &mut max_glass_precalc);
        let finish = std::time::Instant::now();
        println!("==>    Time for glasses of size {}: {} ms -> max glass {}", 
            glass, (finish - start).as_millis(), max_glass_precalc[glass]);
    }

    for glass in max_glass..CONFIG_SIZE {
        max_glass_precalc[glass] = CONFIG_SIZE as Integer;
    }

    return max_glass_precalc;
}

fn next_plane_id (config: &mut Config) -> usize { 
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    let mut p_min_i : usize = 0;
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height {
            if plate.width < l_min {
                l_min = plate.width;
                p_min_i = i;
            }
        }
    }
    return p_min_i;
}
fn next_plate(config: &mut Config) -> () { //find smallest delimited plate, and decompose it
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    // let mut walls_ratio : f32 = -1000.0;
    let mut p_min_i : usize = 0;
    // let mut p_max_ratio : usize = 0;
    //find the minimum delimited plate
    //if, in the meantime, we identify that there is only three plates, we have found a square or rectangle and should return them
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height {
            if plate.height > 0 && is_unfillable_glass(config, i, plate_prev.height - plate.height, plate.width, plate_next.height - plate.height) {
                // println!("[next_plate] GO BACK for plate_id: {}, prev: {}, w: {} can_use: {}, next:{}, config: {}", i, plate_prev.height - plate.height, plate.width, config.can_use(plate.width), plate_next.height - plate.height, config);
                return;
            }
            // let depth = std::cmp::min(plate_prev.height - plate.height, plate_next.height - plate.height);
            // let new_ratio : f32 =  ((depth - (if config.can_use(plate.width) {0} else {plate.width}) ) as f32 / plate.width as f32) as f32;
            // if new_ratio > walls_ratio {
            //     // println!("[next_plate] selected plate_id: {} depth: {}, width: {}, can_use: {}, ratio {} -> {}, config: {}",
            //         // i, depth, plate.width, config.can_use(plate.width), walls_ratio, new_ratio, config);
            //     walls_ratio = new_ratio;
            //     p_max_ratio = i;
            // }
            // println!("[next_plate] selected plate_id: {} depth: {}, width: {}, can_use: {}, ratio {}, config: {}",
            // i, depth, plate.width, config.can_use(plate.width), walls_ratio, config);
            if plate.width < l_min {
                l_min = plate.width;
                p_min_i = i;
            }
        }
    }
    // if p_max_ratio > 0 {
    //     p_min_i = p_max_ratio;
    // }
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
            // println!("FOUND {}", s);
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
            // println!("FOUND {}", s);
            //println!("continuing search...");
            decompose(config, p_min_i);
        }
    }
    else {
        decompose(config, p_min_i);
    }
}

fn next_plate_for_glasses(config: &mut Config, check_glass_size: Integer, max_glass_precalc: &mut [i32; CONFIG_SIZE]) -> () { //find smallest delimited plate, and decompose it
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
pub fn decompose_glasses(mut config: &mut Config, plate_id: usize, check_glass_size: Integer, mut max_glass_precalc: &mut [i32; CONFIG_SIZE]) -> () { 
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

    let square = config.plates[plate_id].width;
    if config.can_use(square) && 
       config.plates[plate_id].height + square <= config.size {
        config.net_squares += 1;
        //eprintln!("a+ {}", config);
        let orig_left_plate_width = config.plates[plate_id - 1].width;
        let new_plate_id = config.vertical_extension(plate_id);
        //println!("{:?}", config);
        next_plate_for_glasses(&mut config, check_glass_size, max_glass_precalc);
        //undo it
        if new_plate_id == plate_id {
            // we were potentially merging with right plate only
            config.reverse_vertical_extension(plate_id, square, 0);
        } else {
            // we were mergeing with left plate and potentially with right plate
            config.reverse_vertical_extension(new_plate_id, square, orig_left_plate_width);
        }  
    } else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    if config.plates[plate_id - 1].height - config.plates[plate_id].height < 
       config.plates[plate_id].width && config.can_use(config.plates[plate_id - 1].height - config.plates[plate_id].height) {
        config.net_squares += 1;
        //eprintln!("b+ {}", config);

        config.horizontal_extension(plate_id);
        //println!("{:?}", config);
        decompose_glasses(config, plate_id, check_glass_size, max_glass_precalc);
        //remove the square
        config.reverse_horizontal_extension(plate_id);
    }

    for s in 1..(std::cmp::min(config.plates[plate_id].width - 1, config.size - config.plates[plate_id].height)+1) {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && s != config.plates[plate_id-1].height - config.plates[plate_id].height{
            config.net_squares += 1;
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

pub fn is_unfillable_glass(config: & Config, _plate_id: usize, left_wall: Integer, width: Integer, right_wall: Integer) -> bool {
    let depth = std::cmp::min(left_wall, right_wall);
    if depth > 0 && depth > config.max_glass[width as usize] - (if config.can_use(width) {0} else {width}) {
        // println!("skip glass: plate_id: {} depth: {}, width/square: {}, config: {}, net_squares: {}", _plate_id, depth, width, config, config.net_squares);
        return true;
    }
    // println!("---- glass: plate_id: {} depth: {}, width/square: {}, config: {}, net_squares: {}", _plate_id, depth, width, config, config.net_squares);
    return false;
}

pub fn decompose(mut config: &mut Config, plate_id: usize) -> () { //given a plate, decompose it by adding squares, then select the next plate if the plates change
    let plate = &config.plates[plate_id];
    let h = plate.height;
    let w = plate.width;
    let last_plate_id = config.plates.len() - 2;
    let lh = config.plates[plate_id - 1].height;
    let lw = config.plates[plate_id - 1].width;
    let rh = config.plates[plate_id + 1].height;
    let rw = config.plates[plate_id + 1].width;
    
    // println!("decomposing, config: {}, plate_id: {}, net_squares: {}", config, plate_id, config.net_squares);
    
    // this could happen if the square equal to the width became used while jumping between plates
    if is_unfillable_glass(config, plate_id,
        lh - h,
        w,
        rh - h) {
        // if glass_width > 15 {
            // println!(
            //     "skipping GLASS h={} w={} h={} can_use({})={} config: {}",
            //     lh - h,
            //     w,
            //     rh - h,
            //     w,
            //     config.can_use(w), config);
        //     }
        return;
    }

    let square = w;
    if config.can_use(square) && 
       h + square <= config.size
        // &&  
    //    (if plate_id == last_plate_id {square >= 5} else {true})
       {
        // the smallest corner block is on bottom left, the first square
        // if we are trying to put the square in the bottom right corner, it must be greater than the first corner
            if h == 0  &&  // bottom line
                square < config.first_corner &&         // smaller than the first 
                plate_id == last_plate_id { // last plate
                // println!("bottom line: SKIP vertical bottom right: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, square, config, config.net_squares);
        // } 
            } else if plate_id >= 2 && 
                is_unfillable_glass(config, plate_id, 
                    config.plates[plate_id-2].height - lh, 
                    lw,
                    h + square - lh) {

            } else if plate_id <= last_plate_id - 1 && 
                is_unfillable_glass(config, plate_id, 
                    h + square - rh,
                    rw,
                    config.plates[plate_id+2].height - rh ) {
            // } else if h == 0  && plate_id == last_plate_id && square < config.first_corner {
                // the right bottom corner should be bigger than the first corner
                // println!("SKIP vertical bottom right: plate_id: {} width/square: {}, first: {} config: {}, net_squares: {}", plate_id, square, config.first_corner, config, config.net_squares);
            } else if plate_id==1 &&                                                  // first plate
                    h + square >= config.size - config.first_corner &&     // top square
                    square < config.first_corner  {                                // smaller than the first 
                // the right bottom corner should be bigger than the first corner
                // println!("SKIP vertical top left: plate_id: {} width/square: {}, first: {} config: {}, net_squares: {}", plate_id, square, config.first_corner, config, config.net_squares);
            } else {
        
        // if h > 0  ||  // bottom line
        //    square > config.first_corner ||         // bigger than the first 
        //    plate_id < last_plate_id      // the last plate on the row
        //    {
                config.net_squares += 1;
                let orig_left_plate_width = lw;
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
        }
    else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    let square = lh - h;
    if square < w && config.can_use(square) {
        if plate_id <= last_plate_id - 1 && 
            is_unfillable_glass(config, plate_id, 
                h + square - rh,
                rw,
                config.plates[plate_id+2].height - rh) {
                // } 
                // else if plate_id==1 &&                                                  // first plate
                //      h + square == config.size &&     // touches the ceiling
                //      square < config.first_corner  {                                // smaller than the first 
                //         println!("bottom line: SKIP horizontal top-left: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, square, config, config.net_squares);
                    } else {

                config.net_squares += 1;

                config.horizontal_extension(plate_id);
                //println!("{:?}", config);
                decompose(config, plate_id);
                //remove the square
                config.reverse_horizontal_extension(plate_id);
                //config_backup.net_squares = config.net_squares;
                //*config = config_backup;
                //eprintln!("b- {}", config);
        }
    }
        //////eprintln!("b.");
    // should be ok even for corners where we do not know the excat shape size as we will never have less than 8 in corners
    let corner_to_box: [i32; 21] = [0, 1, 2, 3, 3, 3, 4, 4, 5, 5, 5, 5, 5, 6, 7, 7, 7, 7, 7, 7, 8];
    let corner_size = std::cmp::min(lh - h, w);
    let min = if corner_size < 21 { corner_to_box[corner_size as usize] } else { 8 };

        // iterate over all possible square sizes that can be added to the bottom left corner.
    //println!("{} to {}", 2, std::cmp::min(w - 1, config.size - h) + 1);

    // let min = if (h == 0 || plate_id == 1) {5} else {2};
    // let min = if (h == 0 || plate_id == 1) {std::cmp::max(5, min_for_corner)} else {min_for_corner};
    let mut max = std::cmp::min(w - 1, config.size - h)+1;

    // after adding a new square, we might get a new glass on the right
    let right_height = rh - h;
    if h > 0 && right_height >= w {
        // if config.max_square_for_glass[glass_width as usize] + 1 < max && glass_width > 5 {
            // println!("SKIP GLASS custom glass {}-{}: plate_id: {}, glass width: {}, right_height: {}, max_for_glass: {}, config: {}, net_squares: {}", 
            //     config.max_square_for_glass[glass_width as usize]+1, max-1, plate_id, glass_width, right_height, config.max_square_for_glass[glass_width as usize], config, config.net_squares);
        // }
        max = std::cmp::min(max, config.max_square_for_glass[w as usize] + 1);
    }

    let right_height = rh - h;
    for s in min..max {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && 
           s != lh - h {

            if plate_id > 1 && 
                is_unfillable_glass(config, plate_id, 
                    config.plates[plate_id-2].height - lh, 
                    lw,
                    h + s - lh) {
        
            } else if h > 0 && 
                is_unfillable_glass(config, plate_id, 
                    s,
                    w - s,
                    rh - h) {

            } else  if h == 0  && plate_id == last_plate_id && s < config.first_corner && w - s <= config.first_corner {
                // the right bottom corner should be bigger than the first corner
            } 
            else if plate_id == 1 &&                                                  // first plate
                 h + s >= config.size - config.first_corner &&     // top square
                 s < config.first_corner  {                                // smaller than the first 
                // println!("bottom line: SKIP custom top-left: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, s, config, config.net_squares);
            } else {

                    config.net_squares += 1;
                    config.add_square_quick(s, plate_id);

                    if h == 0 && s > config.first_corner && config.send.is_some() {
                        let removed_w = w-s;
                        // println!("COMPACT BEFORE for first: {}, size: {}, config: {}", config.first_corner, config.size, config);
                        config.size -= removed_w;
                        config.plates.remove(last_plate_id+1);
                        config.plates[0].height = config.size+1;
                        config.plates[last_plate_id+1].height = config.size+1;
                        // println!("COMPACT READY  for first: {}, size: {}->{}, config: {}", config.first_corner, config.size + removed_w, config.size, config);
                        next_plate(&mut config);

                        config.size += removed_w;
                        config.plates[0].height = config.size+1;
                        config.plates[last_plate_id+1].height = config.size+1;
                        config.plates.insert(last_plate_id+1, Plate{height: 0, width: removed_w});
                        // println!("COMPACT AFTER  for first: {}, size: {}, config: {}", config.first_corner, config.size, config);
                    }

                    //println!("{:?}", config);
                    let next_plate_to_go = next_plane_id(config);
                    if plate_id > 1 && s > lh-h && lw < w - s && config.plates[plate_id - 2].height > lh {
                        decompose(config, plate_id - 1);
                    } else {
                        decompose(config, plate_id + 1); 
                    }

                    // next_plate(&mut config);
                    config.remove_square(plate_id);
                    //println!("{} checked", s);
                }
        }
        else{
            ////eprintln!("{} is not a valid square size", s)
            // println!("min_for_corner={} ", min_for_corner);        
        }
    }

}

pub fn double_nest_init(send: &Sender<Message>,  first: Integer, config: &mut Config) -> () {
    let mut i = 1;
    //start time:
    //let mut start: std::time::Instant = std::time::Instant::now();
    config.net_squares = 2;
    // for s1 in 9..(config.plates[1].width/2+1){
        let s1 = first;
        config.first_corner = s1;
        config.add_square_quick(s1, 1);
            // expect at least 3 squares on bottom line
            for s2 in 5..((config.plates[2].width - 9) + 1){
                if s2 != s1 {
                    config.add_square_quick(s2, 2);
                    if s2 > s1 {
                        if s2-s1 > config.max_glass[s1 as usize] {
                            // println!("init_bottom_corners glass drop plane1  - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                        } else {
                            if s1 < config.plates[3].width {
                                if !is_unfillable_glass(config, 1, config.size - s1, s1, s2 - s1) {
                                    send.send(Message::WorkUnit((config.clone(), 1))).unwrap();
                                }
                            } 
                            // println!("COMPACT BEFORE for first: {}, size: {}, config: {}", config.first_corner, config.size, config);
                            let removed_w = config.size - s1 - s2;
                            let last_plate_id = 2;
                            config.size -= removed_w;
                            config.plates.remove(last_plate_id+1);
                            config.plates[0].height = config.size+1;
                            config.plates[last_plate_id+1].height = config.size+1;
                            // println!("COMPACT READY  for fors  t: {}, size: {}->{}, config: {}", config.first_corner, config.size + removed_w, config.size, config);
                            send.send(Message::WorkUnit((config.clone(), 1))).unwrap();

                            config.size += removed_w;
                            config.plates[0].height = config.size+1;
                            config.plates[last_plate_id+1].height = config.size+1;
                            config.plates.insert(last_plate_id+1, Plate{height: 0, width: removed_w});
                            // println!("COMPACT AFTER for first: {}, size: {}, config: {}", config.first_corner, config.size, config);
                        }
                    }
                    else {
                        if s2 > config.max_glass[(config.size - s1 - s2) as usize] {
                            // println!("init_bottom_corners glass drop plane 3 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                        } else {
                            if !is_unfillable_glass(config, 3, s2, config.size - s1 - s2, config.size) {
                                send.send(Message::WorkUnit((config.clone(), 3))).unwrap();
                            }

                            // if s1 > s2, then s2 is more than a half of the reduced config size of s1+s2. safely skip it


    
                        }
                        
                    }
                    i+=1;
                    config.remove_square(2);
                }
            }
        config.remove_square(1);
    // }
    //end time:
    //println!("{} work units produced", i);
    ////let end = std::time::Instant::now();
    //println!("Time elapsed producing units: {}ms", (end - start).as_millis());
}
