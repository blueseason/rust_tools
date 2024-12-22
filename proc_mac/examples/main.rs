use proc_mac::*;
make_answer!();

#[derive(AnswerFn)]
struct A;

#[derive(HelperAttr)]
struct B {
    #[helper]
    b: (),
}

#[derive(HelperAttr)]
struct C(#[helper] ());

#[show_streams]
fn invoke1() {}

#[show_streams(bar)]
fn invoke2() {}

#[show_streams(multiple => tokens)]
fn invoke3() {}

#[show_streams { delimiters }]
fn invoke4() {}

#[derive(HelloMacro)]
struct Pancakes;

pub trait HelloMacro {
    fn hello_macro();
}

struct MyType {
    s: String,
}

impl MyType {
    pub fn with_label_values(s: &str) -> Self {
        MyType { s: s.to_string() }
    }
}
fn main() {
    println!("{}", answer());
    println!("{}", answer_derive());
    Pancakes::hello_macro();

    make_metrics! {
     pub struct MyStaticMetric {
         foo, bar,
     }
    };

    let metrics = MyStaticMetric::new();
    println!("{}", metrics.foo.s); // 输出 "foo"
    println!("{}", metrics.bar.s); // 输出 "bar"
}
