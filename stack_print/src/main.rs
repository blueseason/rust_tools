use std::arch::asm;

const X86_USER_MAX_VA: u64 = 0x00007FFFFFFFFFFF;

fn main() {
    let mut fp: u64;
    unsafe {
        asm!("mov rbp, {0}",out(reg) fp,);
        loop {
            if fp > 0 && fp < X86_USER_MAX_VA {
                let p = fp as *mut u64;
                println!("Return address: {:p} ", p.wrapping_sub(8));
                println!("Old Stack pointer: {:p}", p.wrapping_sub(16));
                println!("\n");

                fp = *(p.wrapping_sub(16));
            } else {
                break;
            }
        }

        println!("=== End ===");
    }
}
