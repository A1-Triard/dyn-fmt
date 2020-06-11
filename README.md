![travis](https://travis-ci.org/A1-Triard/dyn-fmt.svg?branch=master)

# dyn-fmt

Provides dynamic string format.

```rust
assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3])), "1a2b3c");
assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3, 4])), "1a2b3c");
assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2])), "1a2bc");
assert_eq!(format!("{}", dyn_fmt::Arguments::new("{{}}{}", &[1, 2])), "{}1");
```
## Comparision

|                                           | [dyn-fmt](https://crates.io/crates/dyn-fmt) | [strfmt](https://crates.io/crates/strfmt) | [dynfmt](https://crates.io/crates/dynfmt) |
|:-----------------------------------------:|:-------------------------------------------:|:-----------------------------------------:|:-----------------------------------------:|
|                 no_std                    |                      +                      |                       -                   |                      -                    |
|Easy but powerfull API that you enjoy using|                      +                      |                      +/-                  |                      -                    |
|               Nice license                |                      +                      |                      +/-                  |                      +/-                  |
