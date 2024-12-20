macro_rules! function_item_matcher {
    (

        $( #[$meta:meta] )*
    //  ^~~~attributes~~~~^
        $vis:vis fn $name:ident ( $( $arg_name:ident : $arg_ty:ty ),* $(,)? )
    //                          ^~~~~~~~~~~~~~~~argument list!~~~~~~~~~~~~~~^
            $( -> $ret_ty:ty )?
    //      ^~~~return type~~~^
            { $($tt:tt)* }
    //      ^~~~~body~~~~^
    ) => {
        $( #[$meta] )*
        $vis fn $name ( $( $arg_name : $arg_ty ),* ) $( -> $ret_ty )? { $($tt)* }
    }
}

function_item_matcher!(
    #[inline]
    #[cold]
    pub fn foo(bar: i32, baz: i32) -> String {
        format!("{} {}", bar, baz)
    }
);

macro_rules! struct_item_matcher {
    // Unit-Struct
    (
        $( #[$meta:meta] )*
    //  ^~~~attributes~~~~^
        $vis:vis struct $name:ident;
    ) => {
        $( #[$meta] )*
        $vis struct $name;
    };

    // Tuple-Struct
    (
        $( #[$meta:meta] )*
    //  ^~~~attributes~~~~^
        $vis:vis struct $name:ident (
            $(
                $( #[$field_meta:meta] )*
    //          ^~~~field attributes~~~~^
                $field_vis:vis $field_ty:ty
    //          ^~~~~~a single field~~~~~~^
            ),*
        $(,)? );
    ) => {
        $( #[$meta] )*
        $vis struct $name (
            $(
                $( #[$field_meta] )*
                $field_vis $field_ty
            ),*
        );
    };

    // Named-Struct
    (
        $( #[$meta:meta] )*
    //  ^~~~attributes~~~~^
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
    //          ^~~~field attributes~~~!^
                $field_vis:vis $field_name:ident : $field_ty:ty
    //          ^~~~~~~~~~~~~~~~~a single field~~~~~~~~~~~~~~~^
            ),*
        $(,)? }
    ) => {
        $( #[$meta] )*
        $vis struct $name {
            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_ty
            ),*
        }
    }
}
struct_item_matcher!(
    #[allow(dead_code)]
    #[derive(Copy, Clone)]
    pub(crate) struct Foo {
        pub bar: i32,
        baz: &'static str,
        qux: f32,
    }
);
struct_item_matcher!(
    #[derive(Copy, Clone)]
    pub(crate) struct Bar;
);
struct_item_matcher!(
    #[derive(Clone)]
    pub(crate) struct Baz(i32, pub f32, String);
);
macro_rules! enum_item_matcher {
    // tuple variant
    (@variant $variant:ident (
        $(
            $( #[$field_meta:meta] )*
    //      ^~~~field attributes~~~~^
            $field_vis:vis $field_ty:ty
    //      ^~~~~~a single field~~~~~~^
        ),* $(,)?
    //∨~~rest of input~~∨
    ) $(, $($tt:tt)* )? ) => {

        // process rest of the enum
        $( enum_item_matcher!(@variant $( $tt )*); )?
    };

    // named variant
    (@variant $variant:ident {
        $(
            $( #[$field_meta:meta] )*
    //      ^~~~field attributes~~~!^
            $field_vis:vis $field_name:ident : $field_ty:ty
    //      ^~~~~~~~~~~~~~~~~a single field~~~~~~~~~~~~~~~^
        ),* $(,)?
    //∨~~rest of input~~∨
    } $(, $($tt:tt)* )? ) => {
        // process rest of the enum
        $( enum_item_matcher!(@variant $( $tt )*); )?
    };

    // unit variant
    (@variant $variant:ident $(, $($tt:tt)* )? ) => {
        // process rest of the enum
        $( enum_item_matcher!(@variant $( $tt )*); )?
    };

    // trailing comma
    (@variant ,) => {};
    // base case
    (@variant) => {};

    // entry point
    (
        $( #[$meta:meta] )*
        $vis:vis enum $name:ident {
            $($tt:tt)*
        }
    ) => {
        enum_item_matcher!(@variant $($tt)*);
    };
}

enum_item_matcher!(
    #[derive(Copy, Clone)]
    pub(crate) enum Foo {
        Bar,
        Baz,
    }
);
enum_item_matcher!(
    #[derive(Copy, Clone)]
    pub(crate) enum Bar {
        Foo(i32, f32),
        Bar,
        Baz(),
    }
);
enum_item_matcher!(
    #[derive(Clone)]
    pub(crate) enum Baz {}
);

fn main() {
    assert_eq!(foo(13, 37), "13 37");
    let _: Foo = Foo {
        bar: 42,
        baz: "macros can be nice",
        qux: 3.14,
    };
    let _: Bar = Bar;
    let _: Baz = Baz(2, 0.1234, String::new());
}
