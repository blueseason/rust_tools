use my_macro::{count_exprs, hello_marco, recurrence, *};

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
}
