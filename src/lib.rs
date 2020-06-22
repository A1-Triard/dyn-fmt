#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
pub(crate) mod std {
    pub use core::*;
}

use std::fmt::{self, Display};
use std::hint::{unreachable_unchecked};

#[cfg(feature = "std")]
pub trait AsStrFormatExt: AsRef<str> + Sized {
    /// Creates a [`String`](std::string::String) replacing the {}s within `self` using provided parameters in the order given.
    /// A runtime analog of [`format!`](std::format) macro. In contrast with the macro format string have not be a string literal.
    /// # Examples:
    /// ```rust
    /// use dyn_fmt::AsStrFormatExt;
    /// assert_eq!("{}a{}b{}c".format(&[1, 2, 3]), "1a2b3c");
    /// assert_eq!("{}a{}b{}c".format(&[1, 2, 3, 4]), "1a2b3c"); // extra arguments are ignored
    /// assert_eq!("{}a{}b{}c".format(&[1, 2]), "1a2bc"); // missing arguments are replaced by empty string
    /// assert_eq!("{{}}{}".format(&[1, 2]), "{}1");
    fn format<'a, T: Display + ?Sized + 'a>(self, args: impl IntoIterator<Item=&'a T> + Clone) -> String {
        format!("{}", Arguments::new(self, args))
    }
}

#[cfg(feature = "std")]
impl<T: AsRef<str> + Sized> AsStrFormatExt for T { }

/// This structure represents a format string combined with its arguments.
/// In contrast with [`fmt::Arguments`](std::fmt::Arguments) this structure can be easily and safely created at runtime.
#[derive(Clone, Debug)]
pub struct Arguments<'a, F: AsRef<str>, T: Display + ?Sized + 'a, I: IntoIterator<Item=&'a T> + Clone> {
    fmt: F,
    args: I
}

impl<'a, F: AsRef<str>, T: Display + ?Sized + 'a, I: IntoIterator<Item=&'a T> + Clone> Arguments<'a, F, T, I> {
    /// Creates a new instance of a [`Display`](std::fmt::Display)able structure, representing formatted arguments.
    /// A runtime analog of [`format_args!`](std::format_args) macro.
    /// Extra arguments are ignored, missing arguments are replaced by empty string.
    /// # Examples:
    /// ```rust
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3]); // "1a2b3c"
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3, 4]); // "1a2b3c"
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2]); // "1a2bc"
    /// dyn_fmt::Arguments::new("{{}}{}", &[1, 2]); // "{}1"
    /// ```
    pub fn new(fmt: F, args: I) -> Self { Arguments { fmt, args } }
}

impl<'a, F: AsRef<str>, T: Display + ?Sized + 'a, I: IntoIterator<Item=&'a T> + Clone> Display for Arguments<'a, F, T, I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Eq, PartialEq)]
        enum Brace { Left, Right };
        const LEFT_BRACE: u8 = '{' as u8;
        const RIGHT_BRACE: u8 = '}' as u8;
        fn as_brace(c: u8) -> Option<Brace> {
            match c {
                LEFT_BRACE => Some(Brace::Left),
                RIGHT_BRACE => Some(Brace::Right),
                _ => None
            }
        }
        let mut args = self.args.clone().into_iter();
        let mut fmt = self.fmt.as_ref();
        let mut piece_end = 0;
        enum State { Piece, Arg };
        let mut state = State::Piece;
        loop {
            match state {
                State::Piece => match fmt.as_bytes()[piece_end ..].first() {
                    None => {
                        fmt.fmt(f)?;
                        break;
                    },
                    Some(&b) => match as_brace(b) {
                        Some(b) => {
                            fmt[.. piece_end].fmt(f)?;
                            fmt = &fmt[(piece_end + 1) ..];
                            if fmt.is_empty() { break; }
                            match b {
                                Brace::Left => {
                                    piece_end = 0;
                                    state = State::Arg;
                                },
                                Brace::Right => {
                                    piece_end = 1;
                                    state = State::Piece;
                                }
                            };
                        },
                        None => {
                            piece_end += 1;
                        }
                    }
                },
                State::Arg => match fmt.as_bytes().first() {
                    None => unsafe { unreachable_unchecked() },
                    Some(&RIGHT_BRACE) => {
                        if let Some(arg) = args.next() {
                            arg.fmt(f)?;
                        }
                        fmt = &fmt[1 ..];
                        state = State::Piece;
                    },
                    Some(_) => {
                        piece_end = 1;
                        state = State::Piece;
                    }
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate as dyn_fmt;
    use std::fmt::{self, Write};
    use std::str::{self};

    struct Writer<'a> {
        buf: &'a mut str,
        len: usize,
    }
    
    impl<'a> fmt::Write for Writer<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let buf = &mut self.buf[self.len ..];
            assert!(buf.len() >= s.len());
            let buf = &mut buf[.. s.len()];
            unsafe { buf.as_bytes_mut() }.copy_from_slice(s.as_bytes());
            self.len += s.len();
            Ok(())
        }
    }
    
    #[test]
    fn write_args() {
        let args_format = dyn_fmt::Arguments::new("{}{}{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("123", &buf[.. len]);
    }

    #[test]
    fn write_str() {
        let args_format = dyn_fmt::Arguments::new("abcd{}абвгд{}{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("abcd1абвгд23", &buf[.. len]);
    }

    #[test]
    fn complex_case_1() {
        let args_format = dyn_fmt::Arguments::new("{{}}x{{}{}}y{", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{}x{{}y", &buf[.. len]);
    }

    #[test]
    fn complex_case_2() {
        let args_format = dyn_fmt::Arguments::new("{{{}}}x{y}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{1}xy", &buf[.. len]);
    }

    #[test]
    fn complex_case_3() {
        let args_format = dyn_fmt::Arguments::new("{{{}}}x{{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{1}x{", &buf[.. len]);
    }

    #[test]
    fn fmt_lifetime() {
        fn display<'a, 'b>(f: &'a str, i: &'a [u8], buf: &'b mut str) -> &'b str {
            let args_format = dyn_fmt::Arguments::new(f, i);
            let mut writer = Writer { buf, len: 0 };
            write!(&mut writer, "{}", args_format).unwrap();
            let len = writer.len;
            &buf[.. len]
        }
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let res = display("{}", &[0], buf);
        assert_eq!("0", res);
    }
}