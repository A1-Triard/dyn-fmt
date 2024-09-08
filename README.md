![maintenance: passively maintained](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)

# dyn-fmt

Provides dynamic string format.

```rust
use dyn_fmt::AsStrFormatExt;

fn main() {
    assert_eq!("{}a{}b{}c".format(&[1, 2, 3]), "1a2b3c");
    assert_eq!("{}a{}b{}c".format(&[1, 2, 3, 4]), "1a2b3c");
    assert_eq!("{}a{}b{}c".format(&[1, 2]), "1a2bc");
    assert_eq!("{{}}{}".format(&[1, 2]), "{}1");
}
```
## Comparision

|                                           | [dyn-fmt](https://crates.io/crates/dyn-fmt) | [strfmt](https://crates.io/crates/strfmt) | [dynfmt](https://crates.io/crates/dynfmt) |
|:-----------------------------------------:|:-------------------------------------------:|:-----------------------------------------:|:-----------------------------------------:|
|                 no_std                    |                      +                      |                       -                   |                      -                    |
|Easy but powerfull API that you enjoy using|                      +                      |                      +/-                  |                      -                    |
|               Nice license                |                      +                      |                      +/-                  |                      +/-                  |
