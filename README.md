![travis](https://travis-ci.org/A1-Triard/dyn-fmt.svg?branch=master)

# dyn-fmt

Provides dynamic string format.

```rust
use std::io::{stdin};
use utf8_chars::{BufReadCharsExt};

fn main() {
    for c in stdin().lock().io_chars().map(|x| x.unwrap()) {
        println!("{}", c);
    }
}
```
