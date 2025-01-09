use std::mem::size_of;
static B: [u8; 10] = [99, 97, 114, 114, 121, 116, 111, 119, 101, 108];
static C: [u8; 11] = [116, 104, 97, 110, 107, 115, 102, 105, 115, 104, 0];

use std::env;
use std::fs::File;
use std::io::prelude::*;

const BYTES_PER_LINE: usize = 16; // <1>

fn main() {
    let arg1 = env::args().nth(1);

    let fname = arg1.expect("usage: fview FILENAME");

    let mut f = File::open(&fname).expect("Unable to open file.");
    let mut pos = 0;
    let mut buffer = [0; BYTES_PER_LINE];

    while let Ok(_) = f.read_exact(&mut buffer) {
        print!("[0x{:08x}] ", pos);
        for byte in &buffer {
            match *byte {
                0x00 => print!(".  "),
                0xff => print!("## "),
                _ => print!("{:02x} ", byte),
            }
        }

        println!("");
        pos += BYTES_PER_LINE;
    }
}

/*fn main() {
    let a: i64 = 42;
    let a_ptr = &a as *const i64;
    let a_addr: usize = unsafe { std::mem::transmute(a_ptr) };
    println!("a: {} ({:p}...0x{:x})", a, a_ptr, a_addr + 7);
    /*
        let a: usize = 42;
        let b: &[u8; 10] = &B;
        let c: Box<[u8]> = Box::new(C);

        println!("a (an unsigned integer):");
        println!(" location: {:p}", &a);
        println!(" size:{:?} bytes", size_of::<usize>());
        println!(" value:{:?}", a);
        println!();

        println!("b (a reference to B):");
        println!(" location: {:p}", &b);
        println!(
            " size:
    {:?} bytes",
            size_of::<&[u8; 10]>()
        );
        println!(" points to: {:p}", b);
        println!();
        println!("c (a \"box\" for C):");
        println!(" location: {:p}", &c);
        println!(
            " size:
    {:?} bytes",
            size_of::<Box<[u8]>>()
        );
        println!(" points to: {:p}", c);
        println!();
        println!("B (an array of 10 bytes):");
        println!(" location: {:p}", &B);
        println!(
            " size:
    {:?} bytes",
            size_of::<[u8; 10]>()
        );
        println!(
            " value:
    {:?}",
            B
        );
        println!();
        println!("C (an array of 11 bytes):");
        println!(" location: {:p}", &C);
        println!(
            " size:
    {:?} bytes",
            size_of::<[u8; 11]>()
        );
        println!(
            " value:
    {:?}",
            C
        );*/
}
*/
