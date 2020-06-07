![travis](https://travis-ci.org/A1-Triard/dyn-fmt.svg?branch=master)

# dyn-fmt

Provides dynamic string format.

```rust
use dyn_fmt;

fn main() {
    assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3])), "1a2b3c");
    assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2, 3, 4])), "1a2b3c");
    assert_eq!(format!("{}", dyn_fmt::Arguments::new("{}a{}b{}c", &[1, 2])), "1a2bc");
    assert_eq!(format!("{}", dyn_fmt::Arguments::new("{{}}{}", &[1, 2])), "{}1");
}
```
