macro_rules! mixed_rules {
    () => {};
    (trace $name:ident;$($tail:tt)*) => {
        {
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };
    (trace $name:ident = $init:expr;$($tail:tt)*) => {
        {
            let $name = $init;
            println!(concat!(stringify($name)," = {:?}", $name));
            mixed_rules!($($tail)*)
        }
    }
}
