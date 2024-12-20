#[macro_export]
macro_rules! call_with_larch {
    ($callback:ident) => {
        $callback!(larch); // not work
    };
}
#[macro_export]
macro_rules! expand_to_larch {
    () => {
        larch
    };
}
#[macro_export]
macro_rules! recognize_tree {
    (larch) => {
        println!("#1, the Larch.")
    };
    (redwood) => {
        println!("#2, the Mighty Redwood.")
    };
    (fir) => {
        println!("#3, the Fir.")
    };
    (chestnut) => {
        println!("#4, the Horse Chestnut.")
    };
    (pine) => {
        println!("#5, the Scots Pine.")
    };
    ($($other:tt)*) => {
        println!("I don't know; some kind of birch maybe?")
    };
}

#[macro_export]
macro_rules! callback {
    ($callback:ident( $($args:tt)* )) => {
        $callback!( $($args)* )
    };
}
