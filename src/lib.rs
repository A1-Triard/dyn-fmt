#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

//! |        Static format macro         |           Dynamic analog           |
//! |:----------------------------------:|:----------------------------------:|
//! |      [`format!`](std::format )     | [`format`](AsStrFormatExt::format) |
//! | [`format_args!`](std::format_args) | [`Arguments::new`](Arguments::new) |
//! |       [`write!`](std::write)       |      [`dyn_write!`](dyn_write)     |
//!
//! **Crate features**
//!
//! * `"std"`
//! Enabled by default. Disable to make the library `#![no_std]`.

#[cfg(feature = "std")]
extern crate core;

use core::fmt::{self, Display};

#[doc(hidden)]
pub use core::write as std_write;

/// Extends strings with the `format` method, which is a runtime analog of the [`format!`](std::format) macro.
/// Unavailable in `no_std` environment.
#[cfg(feature = "std")]
pub trait AsStrFormatExt: AsRef<str> {
    /// Creates a [`String`] replacing the {}s within `self` using provided parameters in the order given.
    /// A runtime analog of [`format!`](std::format) macro. In contrast with the macro format string have not be a string literal.
    /// # Examples:
    /// ```rust
    /// use dyn_fmt::AsStrFormatExt;
    /// assert_eq!("{}a{}b{}c".format(&[1, 2, 3]), "1a2b3c");
    /// assert_eq!("{}a{}b{}c".format(&[1, 2, 3, 4]), "1a2b3c"); // extra arguments are ignored
    /// assert_eq!("{}a{}b{}c".format(&[1, 2]), "1a2bc"); // missing arguments are replaced by empty string
    /// assert_eq!("{{}}{}".format(&[1, 2]), "{}1");
    fn format<'a, T: Display>(&self, args: &[T]) -> String {
        format!("{}", Arguments::new(self, args))
    }
}

#[cfg(feature = "std")]
impl<T: AsRef<str>> AsStrFormatExt for T {}

/// Writes formatted data into a buffer. A runtime analog of [`write!`](std::write) macro.
/// In contrast with the macro format string have not be a string literal.
///
/// This macro accepts three arguments: a writer, a format string, and an arguments iterator.
/// Arguments will be formatted according to the specified format string by calling `Arguments::new(fmt, args)`,
/// and the result will be passed to the writer.
///
/// The writer may be any value with a `write_fmt` method; generally this comes from an implementation of either
/// the [`fmt::Write`](std::fmt::Write) or the [`Write`](std::io::Write) trait.
/// The macro returns whatever the `write_fmt` method returns;
/// commonly a [`fmt::Result`](std::fmt::Result), or an [`io::Result`](std::io::Result).
///
/// # Examples:
/// ```rust
/// use dyn_fmt::dyn_write;
/// use std::fmt::Write;
/// let mut buf = String::new();
/// dyn_write!(buf, "{}a{}b{}c", &[1, 2, 3]);
/// assert_eq!(buf, "1a2b3c");
/// ```
#[macro_export]
macro_rules! dyn_write {
    ($dst:expr, $fmt:expr, $args:expr $(,)?) => {
        $crate::std_write!($dst, "{}", $crate::Arguments::new($fmt, $args))
    };
}

/// This structure represents a format string combined with its arguments.
/// In contrast with [`fmt::Arguments`] this structure can be easily and safely created at runtime.
#[derive(Clone, Debug)]
pub struct Arguments<'a, F: AsRef<str>, T: Display> {
    fmt: F,
    args: &'a [T],
}

impl<'a, F: AsRef<str>, T: Display> Arguments<'a, F, T> {
    /// Creates a new instance of a [`Display`]able structure, representing formatted arguments.
    /// A runtime analog of [`format_args!`](std::format_args) macro.
    /// Extra arguments are ignored, missing arguments are replaced by empty string.
    /// # Examples:
    /// ```rust
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3]); // "1a2b3c"
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3, 4]); // "1a2b3c"
    /// dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2]); // "1a2bc"
    /// dyn_fmt::Arguments::new("{{}}{}", &[1, 2]); // "{}1"
    /// ```
    pub fn new(fmt: F, args: &'a [T]) -> Self {
        Arguments { fmt, args }
    }
}

impl<'a, F: AsRef<str>, T: Display> Display for Arguments<'a, F, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Eq, PartialEq)]
        enum Brace {
            Left,
            Right,
        }
        fn as_brace(c: u8) -> Option<Brace> {
            match c {
                b'{' => Some(Brace::Left),
                b'}' => Some(Brace::Right),
                _ => None,
            }
        }

        #[derive(Eq, PartialEq)]
        enum Width {
            Zero(usize),
            Space(usize),
        }

        #[derive(Eq, PartialEq)]
        enum Pos {
            None,
            Some(usize),
            Error,
        }

        #[derive(Eq, PartialEq)]
        enum State {
            Piece,
            Arg,
            ArgPos,
            ArgWidth,
            ArgPrecision,
        }

        #[inline(always)]
        fn parse_pos(s: &str) -> Pos {
            let s = s.trim();
            if s.is_empty() {
                Pos::None
            } else if let Ok(pos) = s.parse::<usize>() {
                Pos::Some(pos)
            } else {
                Pos::Error
            }
        }

        #[inline(always)]
        fn parse_width(s: &str) -> Width {
            if let Ok(w) = s.parse::<usize>() {
                if *s.as_bytes().first().unwrap() == b'0' {
                    Width::Zero(w)
                } else {
                    Width::Space(w)
                }
            } else {
                Width::Space(0)
            }
        }

        let mut state = State::Piece;
        let mut pos_range = (0, 0);
        let mut width_range = (0, 0);
        let mut precision_range = (0, 0);

        let mut args = self.args.into_iter();
        let mut fmt = self.fmt.as_ref();
        let mut piece_end = 0;

        let mut i = 0;
        loop {
            match state {
                State::Piece => match fmt.as_bytes()[piece_end..].first() {
                    None => {
                        fmt.fmt(f)?;
                        break;
                    }
                    Some(&b) => match as_brace(b) {
                        Some(b) => {
                            fmt[..piece_end].fmt(f)?;
                            let step = piece_end + 1;
                            i += step;
                            fmt = &fmt[step..];
                            if fmt.is_empty() {
                                break;
                            }
                            match b {
                                Brace::Left => {
                                    piece_end = 0;
                                    // let pos = None;
                                    state = State::ArgPos;
                                    pos_range = (i, i);
                                    width_range = (0, 0);
                                    precision_range = (0, 0);
                                }
                                Brace::Right => {
                                    piece_end = 1;
                                    state = State::Piece;
                                }
                            };
                        }
                        None => {
                            piece_end += 1;
                        }
                    },
                },
                State::Arg => {
                    let buf = self.fmt.as_ref();
                    let _pos = parse_pos(&buf[pos_range.0..pos_range.1]);
                    let _width = parse_width(&buf[width_range.0..width_range.1]);
                    let _precision = buf[precision_range.0..precision_range.1]
                        .parse::<usize>()
                        .ok();

                    if let Some(arg) = match _pos {
                        Pos::Some(i) => self.args.get(i),
                        Pos::None => args.next(),
                        Pos::Error => None,
                    } {
                        match (_width, _precision) {
                            (Width::Space(0), None) => {
                                write!(f, "{}", arg)?;
                            }
                            (Width::Zero(w), Some(p)) => {
                                write!(f, "{:01$.2$}", arg, w, p)?;
                            }
                            (Width::Space(w), Some(p)) => {
                                write!(f, "{:1$.2$}", arg, w, p)?;
                            }
                            (Width::Zero(w), None) => {
                                write!(f, "{:01$}", arg, w)?;
                            }
                            (Width::Space(w), None) => {
                                write!(f, "{:1$}", arg, w)?;
                            }
                        }
                    }

                    state = State::Piece;
                }

                State::ArgPos | State::ArgWidth | State::ArgPrecision => {
                    match fmt.as_bytes().first() {
                        Some(b'}') => {
                            i += 1;
                            fmt = &fmt[1..];
                            state = State::Arg;
                        }

                        Some(b'{') => {
                            state = State::Piece;
                            piece_end = 1;
                        }

                        Some(b':') if state == State::ArgPos => {
                            i += 1;
                            fmt = &fmt[1..];
                            width_range = (i, i);
                            state = State::ArgWidth;
                        }

                        Some(b'.') if state == State::ArgWidth => {
                            i += 1;
                            fmt = &fmt[1..];
                            precision_range = (i, i);
                            state = State::ArgPrecision;
                        }

                        Some(_) => {
                            match state {
                                State::ArgPos => {
                                    pos_range.1 += 1;
                                }
                                State::ArgWidth => {
                                    width_range.1 += 1;
                                }
                                State::ArgPrecision => {
                                    precision_range.1 += 1;
                                }
                                _ => unreachable!(),
                            }
                            i += 1;
                            fmt = &fmt[1..];
                        }
                        None => unreachable!(),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate as dyn_fmt;
    #[cfg(feature = "std")]
    use crate::AsStrFormatExt;
    use core::fmt::{self, Write};
    use core::str::{self};

    #[cfg(feature = "std")]
    #[test]
    fn test_format() {
        assert_eq!("{}a{}b{}c".format(&[1, 2, 3]), "1a2b3c");
        assert_eq!("{}a{}b{}c".format(&[1, 2, 3, 4]), "1a2b3c");
        assert_eq!("{}a{}b{}c".format(&[1, 2]), "1a2bc");
        assert_eq!("{{}}{}".format(&[1, 2]), "{}1");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_format_pos() {
        assert_eq!("{2}a中文内{1}b{0}c".format(&[1, 2, 3]), "3a中文内2b1c");
        assert_eq!("{1 }a{ 2}b{0}c".format(&[1, 2, 3, 4]), "2a3b1c");
        assert_eq!("{1}a{ }b{0}c".format(&[1, 2]), "2a1b1c");
        assert_eq!("{1}a{}b{}c".format(&[1, 2, 3]), "2a1b2c");
        assert_eq!("{{}}{}".format(&[1, 2]), "{}1");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_format_width() {
        assert_eq!("{2:04}a{1}b{0}c".format(&[1, 2, 3]), "0003a2b1c");
        assert_eq!("{1:05}a{2}b{0:02}c".format(&[1, 2, 3, 4]), "00002a3b01c");
        assert_eq!("{1 }a{:4}b{0}c".format(&[1, 2]), "2a   1b1c");
        assert_eq!("{ 1}a{:02}b{:03}c".format(&[1, 2, 3]), "2a01b002c");
        assert_eq!("{{:01}}{:04}".format(&[1, 2]), "{:01}0001");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_format_precision() {
        assert_eq!("{2:04.1}a{1}b{0}c".format(&[1, 2, 3]), "0003a2b1c");
        assert_eq!(
            "{1:05}a{2}абвгдb{0:02}c".format(&[1, 2, 3, 4]),
            "00002a3абвгдb01c"
        );
        assert_eq!(
            "{  1:.3}a{:4.3}b{0:.2}c".format(&[1.0, 2.123456]),
            "2.123a1.000b1.00c"
        );
        assert_eq!("{1}a{:02}b{:03}c".format(&[1.0, 2.0, 3.0]), "2a01b002c");
        assert_eq!("{{:01.2}}{:04.2}".format(&[1.0, 2.5677]), "{:01.2}1.00");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_format_with_string_format() {
        let format: String = "{}a{}b{}c".into();
        assert_eq!(format.format(&[1, 2, 3]), "1a2b3c");
        assert_eq!(format.format(&[2, 3, 4]), "2a3b4c");
    }

    struct Writer<'a> {
        buf: &'a mut str,
        len: usize,
    }

    impl<'a> fmt::Write for Writer<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let buf = &mut self.buf[self.len..];
            assert!(buf.len() >= s.len());
            let buf = &mut buf[..s.len()];
            unsafe { buf.as_bytes_mut() }.copy_from_slice(s.as_bytes());
            self.len += s.len();
            Ok(())
        }
    }

    #[test]
    fn test_write() {
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        dyn_write!(&mut writer, "{}a{}b{}c", &[1, 2, 3]).unwrap();
        let len = writer.len;
        assert_eq!("1a2b3c", &buf[..len]);
    }

    #[test]
    fn write_args() {
        let args_format = dyn_fmt::Arguments::new("{}{}{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("123", &buf[..len]);
    }

    #[test]
    fn write_unsized_args() {
        let args = &[&1, &2, &3];
        let args_format = dyn_fmt::Arguments::new("{}{}{}", args);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("123", &buf[..len]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn format_unsized_args() {
        let args = &[&1, &2, &3];
        let args_format = "{}{}{}".format(args);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("123", &buf[..len]);
    }

    #[test]
    fn write_str() {
        let args_format = dyn_fmt::Arguments::new("abcd{}абвгд{}{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("abcd1абвгд23", &buf[..len]);
    }

    #[test]
    fn complex_case_1() {
        let args_format = dyn_fmt::Arguments::new("{{}}x{{}{}}y{", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{}x{{}y", &buf[..len]);
    }

    #[test]
    fn complex_case_2() {
        let args_format = dyn_fmt::Arguments::new("{{{}}}x{y}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{1}x", &buf[..len]);
    }

    #[test]
    fn complex_case_3() {
        let args_format = dyn_fmt::Arguments::new("{{{}}}x{{}", &[1, 2, 3]);
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        write!(&mut writer, "{}", args_format).unwrap();
        let len = writer.len;
        assert_eq!("{1}x{", &buf[..len]);
    }

    #[test]
    fn fmt_lifetime() {
        fn display<'a, 'b>(f: &'a str, i: &'a [u8], buf: &'b mut str) -> &'b str {
            let args_format = dyn_fmt::Arguments::new(f, i);
            let mut writer = Writer { buf, len: 0 };
            write!(&mut writer, "{}", args_format).unwrap();
            let len = writer.len;
            &buf[..len]
        }
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let res = display("{}", &[0], buf);
        assert_eq!("0", res);
    }

    #[test]
    fn write_macros() {
        let mut buf = [0u8; 128];
        let buf = str::from_utf8_mut(&mut buf).unwrap();
        let mut writer = Writer { buf, len: 0 };
        dyn_write!(&mut writer, "abcd{}абвгд{}{}", &[1, 2, 3]).unwrap();
        let len = writer.len;
        assert_eq!("abcd1абвгд23", &buf[..len]);
    }
}
