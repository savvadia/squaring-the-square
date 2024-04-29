use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use std::mem;

pub type Integer = i32;

#[derive(Clone)]
pub struct Config {
    pub squares: [bool; 256],
    pub size: Integer,
    pub plates: Vec<Plate>,
    pub net_squares: u128,
    pub max_glass: [i32; 256]
}

impl Config{
    pub fn new(size: Integer, max_glass: [i32; 256]) -> Self {
        let mut s = [false; 256];
        let mut p = Vec::new();
        //First plate: height size + 1, width 1
        p.push(Plate{height: size + 1, width: 1});
        //Second plate: height 0, width size
        p.push(Plate{height: 0, width: size});
        //Third plate: height size + 1, width 1
        p.push(Plate{height: size + 1, width: 1});
        Self {
            squares: s,
            size: size,
            plates: p,
            net_squares: 0,
            max_glass: max_glass
        }
    }

    pub fn num_plates(&self) -> usize {
        self.plates.len()
    }

    pub fn can_use(&self, square: Integer) -> bool {
        !self.squares[square as usize]
    }

    pub fn add_square_quick(&mut self, square: Integer, plate_id: usize) -> () {
        //add_square without merge checks.
        self.squares[square as usize] = true;
        let original_plate_height = self.plates[plate_id].height;
        self.plates.insert(plate_id, Plate{height: square + original_plate_height, width: square});
        //take the width of the square from the original plate
        self.plates[plate_id + 1].width -= square;
        // eprintln!("c+ {}", self);
        // println!("c+ {}", self);
    }

    pub fn horizontal_extension(&mut self, plate_id: usize) -> () { //extends the plate on the left by adding a square
        let square = self.plates[plate_id - 1].height - self.plates[plate_id].height;
        self.squares[square as usize] = true;
        self.plates[plate_id - 1].width += square;
        self.plates[plate_id].width -= square;
        ////eprintln!("b+ {}", self);
        // println!("h+ {}", self);
    }

    pub fn reverse_horizontal_extension(&mut self, plate_id: usize) -> () { //remove extension
        let square = self.plates[plate_id - 1].height - self.plates[plate_id].height;
        self.squares[square as usize] = false;
        self.plates[plate_id - 1].width -= square;
        self.plates[plate_id].width += square;
        ////eprintln!("b- {}", self);
        // println!("h- {}", self);
    }

    // returns new plate id
    pub fn vertical_extension(&mut self,  plate_id: usize) -> usize {
        let square = self.plates[plate_id].width;
        let mut new_plate_id = plate_id;
        self.squares[square as usize] = true;
        self.plates[plate_id].height += square;
        if self.plates[plate_id].height == self.plates[plate_id + 1].height {
            //merge the two plates:
            self.plates[plate_id].width += self.plates[plate_id + 1].width;
            self.plates.remove(plate_id + 1);
        }
        if self.plates[plate_id].height == self.plates[plate_id - 1].height {
            //merge the two plates:
            self.plates[plate_id - 1].width += self.plates[plate_id].width;
            self.plates.remove(plate_id);
            new_plate_id = plate_id - 1;
        }
        ////eprintln!("a+ {}", self);
        // println!("v+ {} {}", square, self);
        return new_plate_id;
    }     

    pub fn reverse_vertical_extension(&mut self,  mut plate_id: usize, square: Integer, orig_left_plate_width: Integer) -> () {
        self.squares[square as usize] = false;
        let mut plate_width = self.plates[plate_id].width;
        let plate_height = self.plates[plate_id].height;
        // println!("v- start {} plate: {} orig_left={} {}", square, plate_id, orig_left_plate_width, self);

        if orig_left_plate_width > 0 {
            // height will be adjusted below
            self.plates.insert(plate_id+1, Plate{height: plate_height, width: square});
            self.plates[plate_id].width = orig_left_plate_width;
            plate_id += 1;
            plate_width -= orig_left_plate_width;
            // println!("v- left {} {}", square, self);
        }
        if plate_width > square {
            self.plates.insert(plate_id+1, Plate{height: plate_height, width: plate_width - square});
            self.plates[plate_id].width = square;
            // println!("v- right {} {}", square, self);
        }
        self.plates[plate_id].height -= square;

        // println!("v- {} {}", square, self);
    }     

    pub fn remove_square(&mut self, plate_id: usize) -> () {
        //remove_square, which makes it's entire own plate, and merges with the next plate
        let square = self.plates[plate_id].width;
        self.squares[square as usize] = false;
        self.plates.remove(plate_id);
        self.plates[plate_id].width += square;
        ////eprintln!("c- {}", self);
        // println!("c- {}", self);

    }

    pub fn add_square_quick_right(&mut self, square: Integer, plate_id: usize) -> () {
        //add_square without merge checks.
        self.squares[square as usize] = true;
        let original_plate_height = self.plates[plate_id].height;
        self.plates.insert(plate_id+1, Plate{height: square + original_plate_height, width: square});
        //take the width of the square from the original plate
        self.plates[plate_id].width -= square;
        // eprintln!("c+ {}", self);
        // println!("c+ {}", self);
    }

    pub fn remove_square_right(&mut self, plate_id: usize) -> () {
        //remove_square, which makes it's entire own plate, and merges with the next plate
        let square = self.plates[plate_id].width;
        self.squares[square as usize] = false;
        self.plates.remove(plate_id);
        self.plates[plate_id-1].width += square;
        ////eprintln!("c- {}", self);
        // println!("c- {}", self);

    }


    pub fn print_squares(&self) -> () {
        let mut s = String::new();
        s += "{";
        for i in 0..256 {
            if self.squares[i] {
                s += &i.to_string();
                s += ", ";
            }
        }
        //remove last two elements in squares_out:
        s.pop();
        s.pop();
        s += "}";
        println!("{}", s);
    }

    pub fn squares_to_string(&self) -> String {
        let mut s = String::new();
        s += "{";
        for i in 0..256 {
            if self.squares[i] {
                s += &i.to_string();
                s += ", ";
            }
        }
        //remove last two elements in squares_out:
        if s.len()>2 {
            s.pop();
            s.pop();
        }

        s += "}";
        s
    }

    pub fn order(&self) -> usize {
        let mut order = 0;
        for i in 0..256{
            if self.squares[i]{
                order += 1;
            }
        }
        order
    }
}

impl Debug for Config{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut squares_out : String = "{".to_string();
        for i in 0..256 {
            if self.squares[i] {
                squares_out += &i.to_string();
                squares_out += ", ";
            }
        }
        //remove last two elements in squares_out:

        if squares_out.len() > 2 {
            squares_out.pop();
            squares_out.pop();
        }
        squares_out += "}";

        write!(f, "Config: size: {}, squares: {:?}, plates: {:?}", self.size, squares_out, self.plates)
    }
}

impl Display for Config{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut squares_out : String = "{".to_string();
        for i in 0..256 {
            if self.squares[i] {
                squares_out += &i.to_string();
                squares_out += ", ";
            }
        }
        //remove last two elements in squares_out:
        if squares_out.len() > 2 {
            squares_out.pop();
            squares_out.pop();
        }
        squares_out += "}";

        write!(f, "Config: size: {}, squares: {:?}, plates: {:?}", self.size, squares_out, self.plates)
    }
}

#[derive(Clone)] //useful later for threads
pub struct Square {
    squares: HashSet<Integer>,
    // todo: bouwkcamp / table code
}

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = Vec::new();
        for i in &self.squares {
            s.push(i);
        }
        write!(f, "Square: {:?}", s)
    }
}

#[derive(Clone)]
pub struct Plate {
    pub height: Integer,
    pub width: Integer,
}
impl Debug for Plate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}