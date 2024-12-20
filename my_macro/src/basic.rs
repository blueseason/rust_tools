#![allow(dead_code)]

macro_rules! as_expr {
    ($e:expr) => {
        $e
    };
}
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}
macro_rules! as_pat {
    ($p:pat) => {
        $p
    };
}
macro_rules! as_stmt {
    ($s:stmt) => {
        $s
    };
}
macro_rules! as_ty {
    ($t:ty) => {
        $t
    };
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! count_tts_r {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts_r!($($tail)*)};
}

macro_rules! count_tts_r_impr {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count_tts_r_impr!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count_tts_r_impr!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count_tts_r_impr!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count_tts_r_impr!($($tail)*)};
    () => {0usize};
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts_array {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}
// 互不相同的标识符 的数量时， 可以利用枚举体的 numeric cast 功能来达到统计成员（即标识符）个数
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $($idents,)* __CountIdentsLast }
            const COUNT: u32 = Idents::__CountIdentsLast as u32;
            COUNT
        }
    };
}

macro_rules! count_tts_bit {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count_tts_bit!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count_tts_bit!($($a)*) << 1 };
}

//rustc src/basic.rs && ./basic
fn main() {
    as_item! {struct Dummy;}

    as_stmt!(let as_pat!(_): as_ty!(_) = as_expr!(42));

    assert_eq!(count_tts!(0 1 2), 3);
    assert_eq!(count_tts_r!(0 1 2 4), 4);

    assert_eq!(
        700,
        count_tts_r_impr!(
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,

            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
            ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,, ,,,,,,,,,,
        )
    );

    const N: usize = count_tts!(0 1 2);
    let array = [0; N];
    println!("{:?}", array);

    const COUNT: u32 = count_idents!(A, B, C);
    assert_eq!(COUNT, 3);
}
