#[macro_export]
#[cfg_attr(doctest, doc = " ````no_test")]
/// Prints a debugging message with call trace
///
/// Uses full format, i.e. `{:#?}`
///
/// ---
///
/// Does nothing if:
/// ```
/// cfg!(debug_assertions) == false
/// ```
macro_rules! printdbf {
    ($($args: expr),*) => {
        // do nothing on release
        if cfg!(debug_assertions) {
            // print trace details in yellow, set bold
            eprint!("{}{}:{}:{}{}", "\x1b[93m", file!(), line!(), column!(), "\x1b[0m");

            let mut i = 0;
            // for each arg
            $(
                i += 1;
                match i {
                    1 => {
                        // print first arg in italice, without binding name
                        let s = format!("{:?}", $args);
                        // shitty workaround to not print quotes for strings
                        let s = &s[1..s.len()-1];
                        // print, white letters & italic
                        eprint!(" >\x1b[1m \x1b[3m{}\x1b[23m", s);
                    },
                    2 => {
                        // stringify to print binding / varname, e.g. x = val
                        eprint!("\n\x1b[93m> \x1b[0m\x1b[36m\x1b[22m{} = {:#?}\x1b[93m;", stringify!($args), $args);
                    },
                    _ => {
                        // print white ";" for arg #2+
                        eprint!("\n\x1b[93m> \x1b[0m\x1b[36m{} = {:#?}\x1b[93m;", stringify!($args), $args);
                    }
                }
            )*

            // end bold && flush with newline
            eprintln!("\x1b[0m\n");
        }
    }
}

#[macro_export]
#[cfg_attr(doctest, doc = " ````no_test")]
/// Prints a debugging message with trace
///
/// Uses compact format, i.e. `{:?}`
///
/// ---
///
/// Does nothing if:
/// ```
/// cfg!(debug_assertions) == false
/// ```
macro_rules! printdb {
    ($($args: expr),*) => {
        // do nothing on release
        if cfg!(debug_assertions) {
            // print trace details in yellow, set bold
            eprint!("{}{}:{}:{}{}", "\x1b[93m", file!(), line!(), column!(), "\x1b[0m");

            let mut i = 0;
            // for each arg ..
            $(
                i += 1;
                match i {
                    1 => {
                        // print first arg in italice, without binding name
                        let s = format!("{:?}", $args);
                        // shitty workaround to not print quotes for strings
                        let s = &s[1..s.len()-1];
                        // print, white letters & italic
                        eprint!(" >\x1b[1m \x1b[3m{}\x1b[23m", s);
                    },
                    2 => {
                        // stringify to print binding / varname, e.g. x = val
                        eprint!("\x1b[0m {}=\x1b[35m\x1b[22m{:?}", stringify!($args), $args);
                    },
                    _ => {
                        // print white "," for arg #2+
                        eprint!("\x1b[37m,\x1b[0m {}=\x1b[35m{:?}", stringify!($args), $args);
                    }
                }
            )*

            // end bold && flush with newline
            eprintln!("\x1b[0m");
        }
    }
}



#[macro_export]
#[cfg_attr(doctest, doc = " ````no_test")]
/// Prints an error/warning message with call trace
///
/// Uses compact format, i.e. `{:?}`
///
/// Does not call a panic unless an error occured, e.g. with [`println`]
///
/// ---
///
/// Does nothing if:
/// ```
/// cfg!(debug_assertions) == false
/// ```
///
macro_rules! printerr {
    ($($args: expr),*) => {
        // do nothing on release
        if cfg!(debug_assertions) {
            // print trace details in yellow, set bold
            eprint!("{}{}:{}:{}{}", "\x1b[41m", file!(), line!(), column!(), "\x1b[0m");

            let mut i = 0;
            // for each arg
            $(
                i += 1;
                match i {
                    1 => {
                        // print first arg in italice, without var name
                        // let s = &format!($args);
                        let s = format!("{:?}", $args);
                        // shitty workaround to not print quotes for strings
                        // print, white letters & italic
                        eprint!(" >\x1b[1m \x1b[3m{}\x1b[23m", &s[1..s.len()-1]);
                    },
                    2 => {
                        // stringify to print binding / varname, e.g. x = val
                        eprint!("\x1b[35m\x1b[22m {}={:#?}", stringify!($args), $args);
                    },
                    _ => {
                        // print white "," for arg #2+
                        eprint!("\x1b[37m,\x1b[35m {}={:#?}", stringify!($args), $args);
                    }
                }
            )*

            // end bold && flush with newline
            eprintln!("\x1b[0m");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_printdb() {
        printdb!("only msg");

        let a: &str = "this is a str";
        let b = String::from("this is a String");
        printdb!("msg, slice, string:", a, b);

        let x = 42;
        let v = vec![0, 1, 2];
        printdb!("msg, int, vector:", x, v);
    }

    #[test]
    fn example_printdbf() {
        let a: &str = "this is a str";
        let b = String::from("this is a String");
        printdbf!("msg, slice, string:", a, b);

        let x = 42;
        let v = vec![0, 1, 2];
        printdbf!("msg, int, vector:", x, v);
    }

    #[test]
    fn example_printerr() {
        printerr!("only errmsg");

        let a: &str = "this is a str";
        let b = String::from("this is a String");
        printerr!("errmsg, slice, string:", a, b);

        let x = 42;
        let v = vec![0, 1, 2];
        printerr!("errmsg, int, vector:", x, v);
    }
}
