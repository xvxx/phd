use std::fmt;

macro_rules! color {
    ($t:ident, $code:expr) => {
        pub struct $t;
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "\x1b[{}m", $code)
            }
        }
    };
}

color!(Black, 90);
color!(Red, 91);
color!(Green, 92);
color!(Yellow, 93);
color!(Blue, 94);
color!(Magenta, 95);
color!(Cyan, 96);
color!(White, 97);

color!(Reset, 0);
color!(Bold, 1);
color!(Underline, 4);
