#![no_std]

#[cfg(test)]
mod no_std_tests {
    use core::fmt::{self, Write};
    use core::str::{self};

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