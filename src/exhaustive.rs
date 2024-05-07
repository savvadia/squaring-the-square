use crate::squares::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, Once};
use crate::coordinator::Message;
use std::io::Write;

//import config
use crate::squares::Config;
use std::fs::{self, File};



pub fn solve (size: Integer, max_glass: [i32; 256], max_square_for_glass: [i32; 256]) -> (){
    //println!{"Solving for size: {}", size};
    let mut config = Config::new(size, max_glass, max_square_for_glass); //Creates the necessary starting plate.
    //println!("Config: {:?}", config);
    decompose(&mut config, 1);
}

pub fn solve_cc(send : &Sender<Message>, size: Integer, max_glass: [i32; 256], max_square_for_glass: [i32; 256]) ->(){
    //println!{"Solving for size: {}", size};
    let mut config = Config::new(size, max_glass, max_square_for_glass); //Creates the necessary starting plate.
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
        let mut config = Config::new(256, max_glass_precalc, max_glass_precalc);
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

pub fn solve_glasses_wo_half(max_glass: usize) -> ([i32; 256]) {
    let mut check_glass_size: i32 = 0;
    let mut max_square_for_glass: [i32; 256] = [0; 256]; // rounddown(width/2)+1 is missing
    for glass in 1..max_glass {
        let start = std::time::Instant::now();
        check_glass_size = glass as i32;
        let mut config = Config::new(256, max_square_for_glass, max_square_for_glass);
        config.plates[1].width = glass as i32;
        config.plates[2].width = (256 - glass) as i32;
        let mut i = 1;
        config.net_squares = 2;
        config.squares[(1) as usize] = true;
        decompose_glasses(&mut config, 1, check_glass_size, &mut max_square_for_glass);
        config.squares[(1) as usize] = false;
        let finish = std::time::Instant::now();
        println!("==>    Time for glasses wo half of size {} without {} : {} ms -> max glass {}", 
            glass, glass/2+1, (finish - start).as_millis(), max_square_for_glass[glass]);
    }

    check_glass_size = 0;
    for glass in max_glass..256 {
        max_square_for_glass[glass] = 256;
    }

    return max_square_for_glass;
}
fn next_plate(config: &mut Config) -> () { //find smallest delimited plate, and decompose it
    let mut l_min : Integer = config.size + Integer::from(1); //equiv to infinity
    let mut walls_ratio : f32 = -1000.0;
    let mut p_min_i : usize = 0;
    // let mut p_max_ratio : usize = 0;
    //find the minimum delimited plate
    //if, in the meantime, we identify that there is only three plates, we have found a square or rectangle and should return them
    for i in 1..config.num_plates()-1 {
        let plate = &config.plates[i];
        let plate_prev = &config.plates[i-1];
        let plate_next = &config.plates[i+1];
        if plate_prev.height > plate.height && plate_next.height > plate.height {
            if is_unfillable_glass(config, i, plate_prev.height - plate.height, plate.width, plate_next.height - plate.height) {
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

pub fn is_unfillable_glass(config: & Config, plate_id: usize, left_wall: Integer, width: Integer, right_wall: Integer) -> bool {
    let depth = std::cmp::min(left_wall, right_wall);
    if depth > 0 && depth > config.max_glass[width as usize] - (if config.can_use(width) {0} else {width}) {
        // println!("skip glass: plate_id: {} depth: {}, width/square: {}, config: {}, net_squares: {}", plate_id, depth, width, config, config.net_squares);
        return true;
    }
    // println!("---- glass: plate_id: {} depth: {}, width/square: {}, config: {}, net_squares: {}", plate_id, depth, width, config, config.net_squares);
    return false;
}

pub fn decompose(mut config: &mut Config, plate_id: usize) -> () { //given a plate, decompose it by adding squares, then select the next plate if the plates change
    // if filling the plate with a square does not make the height greater than the size, add the square and then next plate
    // println!("decomposing, config: {}, plate_id: {}, net_squares: {}", config, plate_id, config.net_squares);
    // let glass_depth = std::cmp::min(
    //     config.plates[plate_id - 1].height - config.plates[plate_id].height,
    //     config.plates[plate_id + 1].height - config.plates[plate_id].height,
    // );
    // let glass_width = config.plates[plate_id].width;
    let h = config.plates[plate_id].height;
    let w = config.plates[plate_id].width;
    let last_plate_id = config.plates.len() - 2;
    let lh = config.plates[plate_id-1].height;
    let lw = config.plates[plate_id-1].width;
    let rh = config.plates[plate_id+1].height;
    let rw = config.plates[plate_id+1].width;
    
    // this could happen if the square equal to the width became used while jumping between plates
    if is_unfillable_glass(config, plate_id,
        config.plates[plate_id - 1].height - h,
        w,
        config.plates[plate_id + 1].height - h) {
        // if glass_width > 15 {
            // println!(
            //     "skipping GLASS h={} w={} h={} can_use({})={} config: {}",
            //     config.plates[plate_id - 1].height - h,
            //     w,
            //     config.plates[plate_id + 1].height - h,
            //     w,
            //     config.can_use(w), config);
        //     }
        return;
    }

    // if config.can_use(glass_width) == false &&
    // glass_width > 15 &&
    // glass_depth > glass_width{
        // println!(
        //     "deep GLASS glass_depth={} w={} {} config: {}",
        //     glass_depth,
        //     w,
        //     if config.can_use(glass_width) {""} else {"(w used)"},
        //     config
        // );
    // }

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
        // else if plate_id==1 &&                                                  // first plate
        //      h + square == config.size &&     // touches the ceiling
        //      square < config.first_corner  {                                // smaller than the first 
                // println!("bottom line: SKIP vertical top-left: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, square, config, config.net_squares);
            // } else if plate_id==1 &&                                                  // first plate
            //  h + square < config.size &&       // not touching ceiling
            //  config.size - h - square < config.first_corner { // not leaving enough space
            //     println!("bottom line: SKIP vertical left: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, square, config, config.net_squares);
            } else if plate_id >= 2 && 
                is_unfillable_glass(config, plate_id, 
                    config.plates[plate_id-2].height - lh, 
                    lw,
                    h + square - lh) {

            } else if plate_id <= last_plate_id - 1 && 
                is_unfillable_glass(config, plate_id, 
                    h + square - rh,
                    rw,
                    config.plates[plate_id+2].height - rh
            ) {

            } else {
        
        // if h > 0  ||  // bottom line
        //    square > config.first_corner ||         // bigger than the first 
        //    plate_id < last_plate_id      // the last plate on the row
        //    {
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
        }
    else{
        ////eprintln!("a.");
    }
    // if the height separating the plate from the one to it's left is less than the length, extend the left plate horizontally by adding the square
    let square = config.plates[plate_id - 1].height - h;
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
    }
        //////eprintln!("b.");
    // should be ok even for corners where we do not know the excat shape size as we will never have less than 8 in corners
    let corner_to_box: [i32; 21] = [0, 1, 2, 3, 3, 3, 4, 4, 5, 5, 5, 5, 5, 6, 7, 7, 7, 7, 7, 7, 8];
    let corner_size = std::cmp::min(config.plates[plate_id - 1].height - h, w);
    let min = if corner_size < 21 { corner_to_box[corner_size as usize] } else { 8 };

        // iterate over all possible square sizes that can be added to the bottom left corner.
    //println!("{} to {}", 2, std::cmp::min(w - 1, config.size - h) + 1);

    // let min = if (h == 0 || plate_id == 1) {5} else {2};
    // let min = if (h == 0 || plate_id == 1) {std::cmp::max(5, min_for_corner)} else {min_for_corner};
    let mut max = std::cmp::min(w - 1, config.size - h)+1;
    // if h == 0 { 
    //     if plate_id == last_plate_id {
    //         let with_space_for_corner = w - config.first_corner - 1 + 1; // -1 for bigger than first_corner; +1 to get max range
    //         if with_space_for_corner < max {
    //             // println!("bottom line: SKIP custom bottom right {}-{}: plate_id: {}, first={}, width: {}, config: {}, net_squares: {}", with_space_for_corner-1, max, plate_id, config.first_corner, w, config, config.net_squares);
    //             max = with_space_for_corner;
    //         }
    //     }    
        // we cannot filter out the top left corner by size as we do not know the width of the rectanle (we try to cover all sizes up to a square)
    // } else if plate_id == 1 {
    //     let with_space_for_corner = (config.size - h) - config.first_corner - 1 + 1; // -1 for bigger than first_corner; +1 to get max range
    //     if with_space_for_corner < max {
            // println!("bottom line: SKIP custom left {}-{}: plate_id: {}, first={}, width: {}, config: {}, net_squares: {}", with_space_for_corner-1, max, plate_id, config.first_corner, w, config, config.net_squares);
            // max = with_space_for_corner;
        // }
    // }

    // after adding a new square, we might get a new glass on the right
    let right_height = config.plates[plate_id + 1].height - h;
    if right_height >= w {
        // if config.max_square_for_glass[glass_width as usize] + 1 < max && glass_width > 5 {
            // println!("SKIP GLASS custom glass {}-{}: plate_id: {}, glass width: {}, right_height: {}, max_for_glass: {}, config: {}, net_squares: {}", 
            //     config.max_square_for_glass[glass_width as usize]+1, max-1, plate_id, glass_width, right_height, config.max_square_for_glass[glass_width as usize], config, config.net_squares);
        // }
        max = std::cmp::min(max, config.max_square_for_glass[w as usize] + 1);
    }

    let right_height = config.plates[plate_id + 1].height - h;
    for s in min..max {
        // if the square can be added to the bottom left corner, add it and then decompose the new plate)
        if config.can_use(s) && 
           s != lh - h {
            // let right_glass_width = glass_width - s;
            // let right_glass_height = std::cmp::min(s, right_height);
            // // s becomes the lowest wall of the new glass on the right of s square
            // if right_glass_height > config.max_glass[right_glass_width as usize] - if config.can_use(right_glass_width) && s != right_glass_width {0} else {right_glass_width} {
                    // if glass_width > 15 {
                    //     println!("SKIP custom square {}: plate_id: {}, glass_width: {}, right_wall: {}, right_g_h: {}, right_g_w: {}, can_use({})={}, max_glass: {}, config: {}, net_squares: {}", 
                    //         s, plate_id, glass_width, right_height, right_glass_height, right_glass_width, right_glass_width, config.can_use(right_glass_width), config.max_glass[right_glass_width as usize], config, config.net_squares);
                    // }
            // } else if plate_id == 1 && config.size - s <= config.first_corner   { //s < config.first_corner{
                // if s > 2 {
                    // println!("SKIP custom TOP LEFT square {}: plate_id: {}, first: {}, space_left_after_s: {}, config: {}, net_squares: {}", 
                    //     s, plate_id, config.first_corner, config.size - s, config, config.net_squares);
                // }

            if plate_id > 1 && 
                is_unfillable_glass(config, plate_id, 
                    config.plates[plate_id-2].height - lh, 
                    lw,
                    h + s - lh) {
        
            } else if plate_id <= last_plate_id && 
                is_unfillable_glass(config, plate_id, 
                    s,
                    w - s,
                    rh - h) {

            } else  if h == 0  && plate_id == last_plate_id && w - s <= config.first_corner {
                // the ritghr bottom corner should be bigger than the first corner
            } 
            else if plate_id==1 &&                                                  // first plate
                 h + s == config.size &&     // touches the ceiling
                 s < config.first_corner  {                                // smaller than the first 
                    // println!("bottom line: SKIP custom top-left: plate_id: {} width/square: {}, config: {}, net_squares: {}", plate_id, s, config, config.net_squares);
        } else {

            // if we are trying to put the square in the bottom right corner, it must be greater than the first corner
            // if h == 0  &&  // bottom line
            // w - s <= config.first_corner && // leaving enough space for the square bigger than the first corner
            // plate_id == last_plate_id      // the last plate on the row
            // {
            // } else {
                    config.net_squares += 1;
                    config.add_square_quick(s, plate_id);
                    //println!("{:?}", config);
                    next_plate(&mut config);
                    config.remove_square(plate_id);
                    //println!("{} checked", s);
            // }
                }
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
        config.first_corner = s1;
        config.add_square_quick(s1, 1);
            // expect at least 3 squares on bottom line
            for s2 in 5..((config.plates[2].width - 9) + 1){
                if s2 != s1 {
                    config.add_square_quick(s2, 2);
                    if s2 > s1 && s1 < config.plates[3].width {
                        // if s2-s1 > config.max_glass[s1 as usize] {
                            // println!("init_bottom_corners glass drop plane1  - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                        // } else {
                            if !is_unfillable_glass(config, 1, config.size - s1, s1, s2 - s1) {
                                send.send(Message::WorkUnit((config.clone(), 1))).unwrap();
                            }
                        // }
                        
                    }
                    else {
                        // if s2 > config.max_glass[(config.size - s1 - s2) as usize] {
                            // println!("init_bottom_corners glass drop plane 3 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                        // } else {
                            if !is_unfillable_glass(config, 3, s2, config.size - s1 - s2, config.size) {
                                send.send(Message::WorkUnit((config.clone(), 3))).unwrap();
                            }
                        // }
                        
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

pub fn tripple_nest_init(send: &Sender<Message>,  config: &mut Config) -> () {
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
                            let glass_width = s1;
                            if glass_width > 0 && s2-s1 > config.max_glass[glass_width as usize] {
                                // println!("init_bottom_corners glass drop plane1  - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                            } else {
                                send.send(Message::WorkUnit((config.clone(), 1))).unwrap();
                            }
                        }
                        else {
                            let glass_width = config.plates[3].width;
                            if glass_width > 0 && s2 > config.max_glass[glass_width as usize] {
                                // println!("init_bottom_corners glass drop plane 3 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                            } else {
                                for s3 in 5..config.plates[3].width {
                                    if s3 == config.plates[3].width {
                                        let new_plate = config.vertical_extension(3);
                                        if s3 > s2 {
                                            send.send(Message::WorkUnit((config.clone(), 2))).unwrap();
                                        } else {
                                            send.send(Message::WorkUnit((config.clone(), 3))).unwrap();
                                        }
                                        config.reverse_vertical_extension(3, s3, s2);
                                    } else {
                                        config.add_square_quick(s3, 3);
                                        let glass_width = config.plates[4].width;
                                        if s3 > config.max_glass[glass_width as usize] {
                                            // println!("init_bottom_corners glass drop plane 4 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                                        } else if s3 > s1 {
                                            let glass_width = s1 + s2;
                                            let glass_height = s3 - s2;
                                            if glass_height > config.max_glass[glass_width as usize] {
                                                // println!("init_bottom_corners glass drop plane 1+2 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                                            } else {
                                                if s3 > s2 {
                                                    let glass_width = s2;
                                                    let glass_height = std::cmp::min(s1,s3);
                                                    if glass_height > config.max_glass[glass_width as usize] {
                                                        // println!("init_bottom_corners glass drop plane 2 between 1/3 - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                                                    } else {
                                                        if s2 < config.plates[4].width {
                                                            send.send(Message::WorkUnit((config.clone(), 2))).unwrap();
                                                        } else {
                                                            send.send(Message::WorkUnit((config.clone(), 4))).unwrap();

                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        config.remove_square(3);
                                    }
                                }
                                send.send(Message::WorkUnit((config.clone(), 3))).unwrap();
                            }
                            
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

pub fn init_bottom_corners(send: &Sender<Message>,  config: &mut Config) -> () {
    let mut i = 1;
    config.net_squares = 2;
    // println!("init_bottom_corners - START size: {}, config: {}, net_squares: {}", config.size, config, config.net_squares);
    // 8 is min for s2, +1 to go to the boundary of size
    for s1 in (9..(config.plates[1].width/2+1)){
        // s1 is the smallest corner box
        if s1+s1+1 > config.size {
            continue
        }
        // println!("init_bottom_corners - start s1: {}, config: {}, net_squares: {}", s1, config, config.net_squares);
        config.add_square_quick(s1, 1);
        for s2 in (s1+1)..(config.plates[2].width+1){
            let glass_width = config.size - s1 - s2;
            if glass_width > 0 && s2 > config.max_glass[glass_width as usize] {
                // println!("init_bottom_corners glass drop  - send, size: {}, s1: {}, s2: {}, glass_width: {}", config.size, s1, s2, glass_width);
                continue
            }
            // println!("init_bottom_corners - start s1: {}, s2: {} config: {}, net_squares: {}", s1, s2, config, config.net_squares);
            if s2 == config.plates[2].width {
                let square = config.plates[2].width;
                config.squares[square as usize] = true;
                config.plates[2].height = square;
                // println!("init_bottom_corners vertical     - send, config: {}, net_squares: {}, plate_id: 1 ", config, config.net_squares);
                // s2 is always less than s1
                send.send(Message::WorkUnit((config.clone(), 1))).unwrap();
                config.squares[square as usize] = false;
                config.plates[2].height = 0;
            } else {
                config.add_square_quick_right(s2, 2);
                // println!("init_bottom_corners right corner - send, config: {}, net_squares: {}, plate_id: 2", config, config.net_squares);
                send.send(Message::WorkUnit((config.clone(), 2))).unwrap();
                i+=1;
                config.remove_square_right(3);
            }
        }
        config.remove_square(1);
    }
}