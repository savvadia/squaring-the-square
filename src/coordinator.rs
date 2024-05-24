use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
//use std::thread::available_parallelism;
use crate::squares::Config;
use crate::exhaustive::*;
use crate::squares::*;
use std::fs::File;
use std::io::Write;
use num_cpus;
use crate::CONFIG_SIZE;

#[derive(Debug)]
pub enum Message {
    ThreadDeath(usize, u128, u128), // first, placed_squares, spent_time_in_ms 
    WorkUnit((Config, usize)),
}

pub fn coordinator_continuous(min_size : Integer, max_size : Integer, max_glass: &[i32; CONFIG_SIZE], max_square_for_glass: &[i32; CONFIG_SIZE]) -> u128{
    let start = std::time::Instant::now();
    let mut size = min_size;
    let mut total_squares = 0;
    let (to_coord, rcv_coord) = channel();
    let nthreads = num_cpus::get() * 2; // works much faster than just number of cores
    println!("will work on {} threads", nthreads);
    let mut file = File::create("timings-".to_owned() + &min_size.to_string()+ " to " + &max_size.to_string() + ".txt").unwrap();
    //println!("Number of threads: {}", nthreads);
    //create an hashmap that contains tuples of threads and senders:
    // let mut threads: HashMap<usize, Integer> = HashMap::new();
    let mut no_of_units_per_first : Vec<Integer>  = vec![0; max_size as usize];
    let mut no_of_threads_per_first : Vec<Integer>  = vec![0; max_size as usize];
    let mut no_of_threads_done_per_first : Vec<Integer>  = vec![0; max_size as usize];
    let mut squares_placed_per_first : Vec<u128> = vec![0; max_size as usize];
    let mut time_spent_per_first : Vec<u128>  = vec![0; max_size as usize];
    let mut size_start_time: [std::time::Instant; CONFIG_SIZE] = [std::time::Instant::now(); CONFIG_SIZE]; // Initialize with default values

    //queue:
    let mut queue: Vec<(Config, usize)> = Vec::new();
    let mut queue_pos = 0;
    //threadcount:
    let mut i = 0;
    let mut f = File::options().append(true).open("timings-".to_owned() + &min_size.to_string()+ " to " + &max_size.to_string() + ".txt").unwrap();
    let mut no_of_started_initial_threads = 0;

    for first in 8 .. (max_size +1)/2+1 {
    
        //random number generator:
        // let mut rng = rand::thread_rng();

        let max_glass_cloned = *max_glass;
        let max_square_for_glass_cloned = *max_square_for_glass;
    
        //create the first thread:
        let to_co = to_coord.clone();
        let new_thread = thread::spawn(move || {
            let start = std::time::Instant::now();
            solve_cc( &to_co, first, max_size, max_glass_cloned, max_square_for_glass_cloned);
            let end = std::time::Instant::now();
            //println!("Time elapsed: {}ms", (end - start).as_millis());
            // println!("Thread {} disconnecting...", i);
            to_co.send(Message::ThreadDeath(i, 0, (end - start).as_millis())).unwrap();
        });
        no_of_units_per_first[first as usize] = 0;
        no_of_threads_per_first[first as usize] = 0;
        no_of_threads_done_per_first[first as usize] = 0;
        squares_placed_per_first[first as usize] = 0;
        time_spent_per_first[first as usize] = 0;
        no_of_started_initial_threads += 1;
    }
    // threads.insert(i, first);
    //println!("Solve called, number of threads: {}", threads.len());
    

    //sleep for a bit:
    thread::sleep(std::time::Duration::from_millis(10));
    
    while no_of_started_initial_threads > 0 {

        let m = rcv_coord.recv().unwrap();
        //println!("Message recieved: {:?}", m);
        match m {
            Message::ThreadDeath(id, squares_placed, time_spent) => {
                //println!("Thread {} disconnected", id);
                no_of_started_initial_threads -= 1;
            },
            Message::WorkUnit(unit) => {
                //add to queue:
                no_of_units_per_first[unit.0.first_corner as usize] += 1;
                queue.push(unit);
                //println!("Work unit recieved, queue length: {}", queue.len());
            },
        }
    }
    let mut running_threads = 0;

    while queue_pos < queue.len() {
        // println!("Number of threads: {}, workunits: {}", running_threads, queue.len() - queue_pos);
        //if there could be more threads:
        while queue_pos < queue.len() && running_threads < nthreads {
            //println!("Number of threads: {}, workunits: {}", threads.len(), queue.len());
            //println!("Queue length: {}", queue.len());
            let (mut config, plate_id) = queue[queue_pos].clone();
            let first : usize = config.first_corner as usize;
            if no_of_threads_per_first[first] == 0 && no_of_threads_done_per_first[first] == 0 {
                size_start_time[first] = std::time::Instant::now();
                println!("{} start at {} ms", config.first_corner, (size_start_time[first] - start).as_millis());
                writeln!(&mut f, "{} start at {} ms", config.first_corner, (size_start_time[first] - start).as_millis()).unwrap();
            }
            no_of_threads_per_first[first] += 1;
            no_of_units_per_first[first] -= 1;
            queue_pos += 1;
            let to_co: std::sync::mpsc::Sender<Message> = to_coord.clone();
            let new_thread = thread::spawn(move || {
                let start = std::time::Instant::now();
                decompose(&mut config, plate_id);
                let end = std::time::Instant::now();
                
                // println!("Thread {} disconnecting...", i);
                // println!("thread stop... : first: {}, squares: {}, time: {} ms", config.first_corner, config.net_squares, (end - start).as_millis());
                to_co.send(Message::ThreadDeath(first, config.net_squares, (end - start).as_millis())).unwrap();
            });
            running_threads += 1;
                        //println!("New thread: {}, threads: {:?}", i, threads.keys());
        }
        //Process incoming messages:
        //It is possible a thread kills if a message is sent after it's last split opportunity.
        //That thread will still send a message, to die.
        //therefore it is safe to wait for a message.

        //little sleep:
        //thread::sleep(std::time::Duration::from_millis(10));
        let m = rcv_coord.recv().unwrap();
        //println!("Message recieved: {:?}", m);
        match m {
            Message::ThreadDeath(index, squares_placed, time_spent) => {
                // println!("Thread {} disconnected", index);
                //println!("Threads {:?}", threads);
                total_squares += squares_placed;
                running_threads -= 1;
                squares_placed_per_first[index as usize] += squares_placed;
                time_spent_per_first[index as usize] += time_spent;
                no_of_threads_per_first[index as usize] -= 1;
                no_of_threads_done_per_first[index as usize] += 1;
                if no_of_units_per_first[index as usize] == 0  && no_of_threads_per_first[index as usize] == 0{
                    writeln!(&mut f, "{} end, placed squares: {} -> {}", index, squares_placed_per_first[index as usize], time_spent_per_first[index as usize]);
                    println!("{:3} end, placed squares: {:13} ongoing_threads: {} -> spent {:12}", index, squares_placed_per_first[index as usize], running_threads, time_spent_per_first[index as usize]);
                }
            },
            Message::WorkUnit(unit) => {
                //add to queue:
                queue.push(unit);
                println!("Work unit recieved at unexpected moment, queue length: {}", queue.len());
            },
            _ => {
                println!("Message recieved: unknown");
            }
        }
    } 

    let end = std::time::Instant::now();
    writeln!(&mut f, "{} to {} squares: {}, total wall-clock time: {}", min_size, max_size, total_squares, (end - start).as_millis()).unwrap();
    println!("{} to {} squares: {}, total wall-clock time: {}", min_size, max_size, total_squares, (end - start).as_millis());
    total_squares
}

pub fn SingleSizeCoordinator(size : Integer, max_glass: &[i32; CONFIG_SIZE], max_square_for_glass: &[i32; CONFIG_SIZE]) -> u128 {

    let start = std::time::Instant::now();
    let (to_coord, rcv_coord) = channel();
    let nthreads = 75; //available_parallelism().unwrap().get();
    println!("SingleSizeCoordinator: Number of threads: {}", nthreads);
    //create an hashmap that contains tuples of threads and senders:
    let mut threads: HashMap<usize, thread::JoinHandle<()>> = HashMap::new();

    //queue:
    let mut queue: Vec<(Config, usize)> = Vec::new();

    //threadcount:
    let mut i = 0;
    let mut total_squares = 0;

    //random number generator:
    let mut rng = rand::thread_rng();

    let max_glass_cloned = *max_glass;
    let max_square_for_glass_cloned = *max_glass;

    //create the first thread:
    
    for first in 8..(size+1)/2 {
        let to_co = to_coord.clone();
        let new_thread = thread::spawn(move || {
            let start = std::time::Instant::now();
            solve_cc( &to_co, first, size, max_glass_cloned, max_square_for_glass_cloned);
            let end = std::time::Instant::now();
            //println!("Time elapsed: {}ms", (end - start).as_millis());
            // println!("Thread {} disconnecting...", i);
            to_co.send(Message::ThreadDeath(i, 0, (end - start).as_millis())).unwrap();
        });
        threads.insert(i, new_thread);
    }
    //println!("Solve called, number of threads: {}", threads.len());
    
    //sleep for a bit:
    thread::sleep(std::time::Duration::from_millis(10));

    let m = rcv_coord.recv().unwrap();
        match m {
            Message::ThreadDeath(index, squares_placed, _time_spent) => {
                //println!("Thread {} disconnected", index);
                threads.remove(&index);
                total_squares += squares_placed;
                //println!("Number of threads: {}, work units: {}", threads.len(), queue.len());
            },
            Message::WorkUnit(unit) => {
                //add to queue:
                queue.push(unit);
                //println!("Work unit recieved, queue length: {}", queue.len());
            },
        }
        
        for received in rcv_coord.try_iter() {
            match received {
                Message::ThreadDeath(index, squares_placed, _time_spent) => {
                    //println!("Thread {} disconnected", index);
                    threads.remove(&index);
                    total_squares += squares_placed;
                    //println!("Number of threads: {}, work units: {}", threads.len(), queue.len());
                },
                Message::WorkUnit(unit) => {
                    //add to queue:
                    queue.push(unit);
                    //println!("Work unit recieved, queue length: {}", queue.len());
                },
                _ => {
                    //println!("Message recieved: unknown");
                }
            }
        }
    println!("Work units: {}", queue.len());

    //While there is more than one thread:
    while queue.len() > 0 {
        //println!("Number of threads: {}, workunits: {}", threads.len(), queue.len());
        //if there could be more threads:
        if threads.len() < nthreads {
            //println!("Number of threads: {}, workunits: {}", threads.len(), queue.len());
            for _ in 0..(nthreads - threads.len()) {
            //println!("Queue length: {}", queue.len());
                match queue.pop() { //TODO: change this to finite length
                    //if queue is not empty:
                    Some(unit) => {
                        //create a new thread:
                        i += 1;
                        let u = unit.clone();
                        let (to_thread, rcv_thread) = channel::<()>();
                        let to_co: std::sync::mpsc::Sender<Message> = to_coord.clone();
                        let new_thread = thread::spawn(move || {
                            //time the lifetime of the thread:
                            let start = std::time::Instant::now();
                            let (mut config, plate_id) = u;
                            //initial_decompose_cc(&to_co, &rcv_thread, &mut config, plate_id);
                            decompose(&mut config, plate_id);
                            let end = std::time::Instant::now();
                            //println!("Time elapsed: {}ms", (end - start).as_millis());
                            println!("Thread {} disconnecting...", i);
                            to_co.send(Message::ThreadDeath(i, config.net_squares, (end - start).as_millis())).unwrap();
                        });
                        threads.insert(i, new_thread);
                        //println!("New thread: {}, threads: {:?}", i, threads.keys());
                    },
                    //if queue is empty:
                    //   randomly select thread, and send it a message to produce a work unit
                    None => {} /*
                        //randomly select a key from threads:
                        let k = threads.keys().cloned().collect::<Vec<usize>>();
                        if k.len() > 0 {
                            let key = k.iter().min().unwrap();//k.choose(&mut rand::thread_rng()).unwrap();
                            //send a message to the thread:
                            match threads.get(&key){
                                Some((_, to_thread)) => {
                                    //println!("Sending request to thread {}", key);
                                    match to_thread.send(()) {
                                        Ok(_) => {/*//println!("Request sent to: {}", key)*/},
                                                
                                        Err(_) => {/*//println!("Request failed, thread {} disconnected", key);
                                                            threads.remove(&key);*/}
                                    }
                                },
                                None => {
                                    //Thread killed in between selection and sending
                                    //println!("Thread {} disconnected", key);
                                    threads.remove(&key);
                                }
                            }
                        }
                    },*/
                }
            }
        }
        //Process incoming messages:
        //It is possible a thread kills if a message is sent after it's last split opportunity.
        //That thread will still send a message, to die.
        //therefore it is safe to wait for a message.

        //little sleep:
        //thread::sleep(std::time::Duration::from_millis(10));



        if queue.len() != 0{
            let m = rcv_coord.recv().unwrap();
            match m {
                Message::ThreadDeath(index, squares_placed, _time_spent) => {
                    //println!("Thread {} disconnected", index);
                    total_squares += squares_placed;
                    threads.remove(&index);
                    //println!("Number of threads: {}, work units: {}", threads.len(), queue.len());
                },
                Message::WorkUnit(unit) => {
                    //add to queue:
                    queue.push(unit);
                    //println!("Work unit recieved, queue length: {}", queue.len());
                },
            }
            for received in rcv_coord.try_iter() {
                match received {
                    Message::ThreadDeath(index, squares_placed, _time_spent) => {
                        //println!("Thread {} disconnected", index);
                        total_squares += squares_placed;
                        threads.remove(&index);
                        //println!("Number of threads: {}, work units: {}", threads.len(), queue.len());
                    },
                    /*Message::WorkUnit(unit) => {
                        //add to queue:
                        queue.push(unit);
                        println!("Work unit recieved, queue length: {}", queue.len());
                    }*/
                    _ => {
                        println!("Message recieved: unknown");
                    }
                }
            }
        }
    }
    let mut f = File::options().append(true).open("timings-".to_owned() + &size.to_string() + ".txt").unwrap();
    write!(&mut f, "{} {}", size, (std::time::Instant::now() - start).as_millis()).unwrap();
    
    if threads.len() != nthreads {
        for _ in 0..(nthreads - threads.len()) {
            write!(&mut f, " {}", (std::time::Instant::now() - start).as_millis()).unwrap();
        }
    }
    while threads.len() > 0 {
        match rcv_coord.recv().unwrap() {
            Message::ThreadDeath(index, squares_placed, _time_spent) => {
                //println!("Thread {} disconnected", index);
                threads.remove(&index);
                total_squares += squares_placed;
            },
            _ => {
                println!("Message recieved: unknown");
            }
        }
        write!(&mut f, " {}", (std::time::Instant::now() - start).as_millis()).unwrap();

    }
    writeln!(&mut f, "").unwrap();
    //println!("sp {} {}", size, total_squares);
    total_squares
}
