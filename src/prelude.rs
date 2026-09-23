pub use log::{error, warn, info, debug, trace};


macro_rules! build_regex {
    (crate $reg_crate:ident, $i:ident = $r:expr) => {

        use $reg_crate::Regex;
        use std::sync::LazyLock;

        // https://docs.rs/regex/latest/regex/#avoid-re-compiling-regexes-especially-in-a-loop
        static $i: LazyLock<Regex> = LazyLock::new(|| Regex::new($r).unwrap());

    };
    ($i:ident = $r:expr) => {
        build_regex!(crate regex, $i = $r)
    };

    (fancy $i:ident = $r:expr) => {
        build_regex!(crate fancy_regex, $i = $r)
    };
}

pub(crate) use build_regex;
