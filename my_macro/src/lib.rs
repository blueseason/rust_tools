mod callback;

#[macro_export]
macro_rules! hello_marco {
    () => {
        ""; //add stat
        println!(" hello from my macro");
    };
}

#[macro_export]
macro_rules! print_idents {
    ($($ide:ident)*) => {
        $(
            println!("{}", stringify!($ide));
        )*
    };
}

#[macro_export]
macro_rules! print_items {
    ($($item:item)*) => {
        $(
            println!("{}", stringify!($item));
        )*
    };
}

#[macro_export]
macro_rules! print_metas {
    ($($meta:meta)*) => {
        $(
            println!("{}", stringify!($meta));
        )*
    };
}

#[macro_export]
macro_rules! count_exprs {
    () => {
        1
    };
    ($head:expr) => {
        1
    };
    ($head:expr,$($tail:expr),*) => {
        1 + count_exprs!($($tail),*)
    };
}

#[macro_export]
macro_rules! recurrence {
    //分隔符,不被识别,编译器无法决定是该将它解析成 inits 中的又一个表达式， 还是解析成 ...
    // , ... ,替换成 ; 或者 ; ... ;
    ($seq:ident [$ind:ident]: $sty:ty = $($inits:expr),+ ;...; $recur:expr) => {
     {
         use std::ops::Index;
         const MEM_SIZE: usize = count_exprs!($($inits),+);
        struct Recurrence {
            mem: [$sty; MEM_SIZE],
            pos: usize,
        }

        struct IndexOffset<'a> {
            slice: &'a [$sty; MEM_SIZE],
            offset: usize,
        }

        //  IndexOffset
        //  简化带偏移的数组访问逻辑
        //  支持环形缓冲区的索引计算，Wrapping 是 Rust 提供的工具，用于防止整数溢出
        //  real_index.0  是为了从 Wrapping 中提取实际的值（usize 类型的索引值），需要访问它的字段，而字段名是 .0

        impl<'a> Index<usize> for IndexOffset<'a> {
            type Output = $sty;
            #[inline(always)]
            fn index<'b>(&'b self, index: usize) -> &'b $sty {
                use std::num::Wrapping;
                let index = Wrapping(index);
                let offset = Wrapping(self.offset);
                let window = Wrapping(MEM_SIZE);
                let real_index = index - offset + window;
                &self.slice[real_index.0]
            }
        }

        impl Iterator for Recurrence {
            type Item = $sty;
            #[inline]
            fn next(&mut self) -> Option<$sty> {
                if self.pos < MEM_SIZE {
                    let next_val = self.mem[self.pos];
                    self.pos += 1;
                    Some(next_val)
                }else {
                    let next_val = {
                        let $ind = self.pos;
                        let $seq = IndexOffset { slice: &self.mem, offset: $ind};
                        $recur
                    };
                    {
                        use std::mem::swap;
                        let mut swap_temp = next_val;
                        //.rev()反向迭代
                        for i in (0..MEM_SIZE).rev() {
                            swap(&mut swap_temp,&mut self.mem[i]);
                        }
                    }
                    self.pos += 1;
                    Some(next_val)
                }
            }
        }

         Recurrence {mem: [$($inits),+], pos: 0}
     }
    };
}
// how to test
// rustup install nightly
// cargo install cargo-expand
// cargo expand
// rustc +nightly -Zunpretty=expanded src/lib.rs(deprecated)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_macro_test() {
        hello_marco!();
    }

    #[test]
    fn recur_macto_test() {
        for e in recurrence!(f[i]: f64 = 1.0; ...; f[i-1] * i as f64).take(10) {
            println!("{}", e)
        }
    }
}

fn main() {
    let fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];

    for e in fib.take(10) {
        println!("{}", e)
    }
}
