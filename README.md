![travis](https://travis-ci.org/A1-Triard/dyn-fmt.svg?branch=master)

# dyn-fmt

Provides dynamic string format.
Features:
1. [Positional parameters](https://doc.rust-lang.org/std/fmt/#positional-parameters)
2. Formatting parameters [width](https://doc.rust-lang.org/std/fmt/#width) and [precision](https://doc.rust-lang.org/std/fmt/#precision)

```rust
use dyn_fmt::AsStrFormatExt;

fn main() {
    assert_eq!("{}a{}b{}c".format(&[1, 2, 3]), "1a2b3c");
    assert_eq!("{}a{}b{}c".format(&[1, 2, 3, 4]), "1a2b3c");
    assert_eq!("{}a{}b{}c".format(&[1, 2]), "1a2bc");
    assert_eq!("{{}}{}".format(&[1, 2]), "{}1");

    // Positional parameters
    ssert_eq!("{2}a{1}b{0}c".format(&[1, 2, 3]), "3a2b1c");
    assert_eq!("{1}a{2}b{0}c".format(&[1, 2, 3, 4]), "2a3b1c");
    assert_eq!("{1}a{}b{0}c".format(&[1, 2]), "2a1b1c");
    assert_eq!("{1}a{}b{}c".format(&[1, 2, 3]), "2a1b2c");

    // Width formatting parameters
    assert_eq!("{2:04}a{1}b{0}c".format(&[1, 2, 3]), "0003a2b1c");
    assert_eq!("{1:05}a{2}b{0:02}c".format(&[1, 2, 3, 4]), "00002a3b01c");
    assert_eq!("{1}a{:4}b{0}c".format(&[1, 2]), "2a   1b1c");

    // Precision formatting parameters
    assert_eq!("{1:.3}a{:4.3}b{0:.2}c".format(&[1.0, 2.123456]),"2.123a1.000b1.00c");

}
```
## Comparision

|                                           | [dyn-fmt](https://crates.io/crates/dyn-fmt) | [strfmt](https://crates.io/crates/strfmt) | [dynfmt](https://crates.io/crates/dynfmt) |
|:-----------------------------------------:|:-------------------------------------------:|:-----------------------------------------:|:-----------------------------------------:|
|                 no_std                    |                      +                      |                       -                   |                      -                    |
|Easy but powerfull API that you enjoy using|                      +                      |                      +/-                  |                      -                    |
|               Nice license                |                      +                      |                      +/-                  |                      +/-                  |
