#[macro_export]
macro_rules! foo {
    //内建宏定义，匹配模式由 as_expr 组成（和命名），然后附加上宏的输入参数 $e:expr ； 在展开里填写这个宏被匹配到时具体的内容。
    (@as_expr $e:expr) => {$e};

    ($($tts:tt)*) => {
        foo!(@as_expr $($tts)*)
    };
}
