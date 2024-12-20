use my_macro::{count_exprs, hello_marco, recurrence, *};
//#![feature(trace_macros)]

fn main() {
    hello_marco!();
    let fib = recurrence![a[n]: u64 = 0, 1;...; a[n-1] + a[n-2]];
    //  let fib = recurrence! [a[n]: u64 = 0,1 ;...; a[n-1]+a[n-2]];
    for e in fib.take(10) {
        println!("{}", e);
    }

    for e in recurrence!(f[i]: f64 = 1.0; ...; f[i-1] * i as f64).take(10) {
        println!("{}", e)
    }

    print_idents! {
        // _ <- `_` 不是标识符，而是一种模式
        foo
        async
        O_________O
        _____O_____
    }

    print_items! {
        struct Foo;
        enum Bar {
            Baz
        }
        impl Foo {}
        /*...*/
    }

    print_metas! {
        ASimplePath
        super::man
        path = "home"
        foo(bar)
    }

    macro_rules! dead_rule {
    ($e:expr) => { ... };
        ($i:ident +) => { ... };
    }
    //dead_rule!(x+);

    macro_rules! double_method {
    ($self_:ident, $body:expr) => {
        fn double(mut $self_) -> Dummy {
            $body
        }
    };
}

    struct Dummy(i32);

    impl Dummy {
        double_method! {self, {
            self.0 *= 2;
            self
        }}
    }

    println!("{:?}", Dummy(4).double().0);

    recognize_tree!(expand_to_larch!()); // 无法直接使用 `expand_to_larch!` 的展开结果
    call_with_larch!(recognize_tree); // 回调就是给另一个宏传入宏的名称 (`ident`)，而不是宏的结果

    callback!(callback(println("Yes, this *was* unnecessary.")));

    let strings: [String; 3] = init_array![String::from("hi!"); 3];
    println!("{:?}", strings);

    //  trace_macros!(true);
    let array: [usize; 4] = init_array1![0; 3; first 0];
    println!("{:?}", array);

    //    let array: [usize; 64] = init_array_r![0;63; first 0];
    assert_eq!(
        tuple_default!(i32, bool, String),
        (i32::default(), bool::default(), String::default())
    );
}
