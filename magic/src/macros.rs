#[macro_export]
macro_rules! import_all {
    ( $( $x:ident ),+ $(,)?) => {
        $(
            mod $x;
            use $x::*;
        )+
    };
}

#[macro_export]
macro_rules! pub_import_all {
    ( $( $x:ident ),+ $(,)?) => {
        $(
            pub mod $x;
            pub use $x::*;
        )+
    };
}
