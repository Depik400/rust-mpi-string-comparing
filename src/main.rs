use clap::Parser;
use mpi::point_to_point::Status;
use mpi::traits::*;

use rand::distributions::{Alphanumeric, DistString};

#[derive(Parser)]
#[command(
    author = "Pavel Kononov, thedepik400@yandex.ru",
    version = "1.0.0",
    long_about = "
Two processes fill their lines (dimensions of line1 and line2 user-defined) random characters.
Program operation ends with outputting both lines to a file if both lines contain the same
and the same characters in any quantity and without regard to their order and none of the strings
does not contain characters that are not in the second line. Otherwise both processes randomly generate strings again, etc.
"
)]
struct Arguments {
    ///Length of first string
    #[arg(short = 'o', long, default_value_t = 10)]
    length_one: i64,
    ///Length of second string
    #[arg(short = 't', long, default_value_t = 10)]
    length_two: i64,
    #[clap(short, long, default_value_t = false)]
    debug: bool,
}

fn generate_random_str_with_length(length: &i64) -> String {
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), (*length) as usize);
    string
}

fn is_good_strings(first: &Vec<u8>, second: &Vec<u8>) -> bool {
    let mut not_contains: bool;
    let mut contains = false;
    for f in first.iter() {
        not_contains = false;
        for s in second.iter() {
            if *f == *s {
                contains = true;
                not_contains = true;
            }
        }
        if !not_contains {
            return false;
        }
    }
    return contains;
}

fn main() {
    let args = Arguments::parse();
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();
    let in_debug = cfg!(feature = "debug") || args.debug;
    if size != 3 {
        eprintln!("Size of mpi processes must be 3. Shutdown");
        std::process::exit(1);
    }
    let string_lengths = [-1, args.length_one, args.length_two];
    let mut string: String;
    let mut string_bytes: Vec<u8> = vec![];
    let mut passed = true;
    while passed {
        if rank == 0 {
            if in_debug {
                println!("[Root] waiting for children messages");
            }
            let (message_from_first, _status): (Vec<u8>, Status) =
                world.process_at_rank(1).receive_vec();
            let (message_from_second, _status): (Vec<u8>, Status) =
                world.process_at_rank(2).receive_vec();
            if in_debug {
                println!(
                    "[Root] received strings first: {:?}, string second: {:?}",
                    String::from_utf8(message_from_first.clone()).unwrap(),
                    String::from_utf8(message_from_second.clone()).unwrap()
                );
            }
            passed = !is_good_strings(&message_from_first, &message_from_second);
            if in_debug {
                println!("[Root] check strings and got {passed}")
            }
            world.process_at_rank(1).send(&passed);
            world.process_at_rank(2).send(&passed);
            if !passed {
                println!(
                    "[Root]: passed string is {:?} and {:?}",
                    String::from_utf8(message_from_first.clone()).unwrap(),
                    String::from_utf8(message_from_second.clone()).unwrap()
                )
            }
        } else {
            string = generate_random_str_with_length(&string_lengths[rank as usize]);
            let mut original_string_bytes = string.as_bytes().to_vec();
            string_bytes.clear();
            string_bytes.append(&mut original_string_bytes);
            if in_debug {
                println!("[Child {rank}] generated string {string} and send it to 0");
            }
            world.process_at_rank(0).send(&string_bytes);
            world.process_at_rank(0).receive_into(&mut passed);
            if in_debug {
                println!(
                    "[Child {rank}]: continue generation because root send {}",
                    (passed).clone()
                )
            }
        }
    }
}
