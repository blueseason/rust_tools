#[macro_export]
macro_rules! init_array {
    (@accum (0, $_e:expr) -> ($($body:tt)*))
        => {init_array!(@as_expr [$($body)*])};
    (@accum (1, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (0, $e) -> ($($body)* $e,))};
    (@accum (2, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (1, $e) -> ($($body)* $e,))};
    (@accum (3, $e:expr) -> ($($body:tt)*))
        => {init_array!(@accum (2, $e) -> ($($body)* $e,))};
    (@as_expr $e:expr) => {$e};
    [$e:expr; $n:tt] => {
        {
            let e = $e;
            init_array!(@accum ($n, e.clone()) -> ())
        }
    };
}

#[macro_export]
macro_rules! init_array1 {
    (@accum (0, $_e:expr) -> ($($body:tt)*))
        => {init_array1!(@as_expr [$($body)*])};
    (@accum (1, $e:expr) -> ($($body:tt)*))
        => {init_array1!(@accum (0, $e) -> ($($body)* $e+3,))};
    (@accum (2, $e:expr) -> ($($body:tt)*))
        => {init_array1!(@accum (1, $e) -> ($($body)* $e+2,))};
    (@accum (3, $e:expr) -> ($($body:tt)*))
        => {init_array1!(@accum (2, $e) -> ($($body)* $e+1,))};
    (@as_expr $e:expr) => {$e};
    [$e:expr; $n:tt $(; first $init:expr)?] => {
        {
            let e = $e;
            init_array1!(@accum ($n, e.clone()) -> ($($init)?,))
        }
    };
}

//#![recursion_limit = "256"]
/*
#[macro_export]
macro_rules! init_array_r {
    // Base case: when n == 0, just return the accumulated array
    (@accum (0, $_e:expr) -> ($($body:tt)*)) => {
        init_array_r!(@as_expr [$($body)*])
    };

    // Recursive case: decrement n and add the next element
    (@accum ($n:expr, $e:expr) -> ($($body:tt)*)) => {
        init_array_r!(@accum ($n - 1, $e) -> ($($body)* $e + $n,))
    };

    // Helper for treating an expression as a single entity
    (@as_expr $e:expr) => { $e };

    // Public entry point for the macro
    [$e:expr; $n:expr $(; first $init:expr)?] => {{
        let e = $e;
        init_array_r!(@accum ($n, e.clone()) -> ($($init)?,))
    }};
}
*/
