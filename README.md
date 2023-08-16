# 🐻‍❄️👀 sentry-eyre
> *Sentry integration for [`eyre`](https://crates.io/crates/eyre)*

**sentry-eyre** is a integration to capture [`eyre::Report`](https://docs.rs/eyre/latest/eyre/struct.Report.html)s. This crate
was inspired by the `sentry-anyhow` integration, and does a similar API but distinct enough to not create any issues.

## Usage
```toml
[dependencies]
sentry-eyre = "0.1"
sentry = "*"
```

```rs
use eyre::Result;
use sentry_eyre::capture_report;
use sentry::{ClientOptions, init, types::Dsn};
use std::io::{Error, ErrorKind};

fn some_method_that_fails() -> Result<()> {
    Err(Error::new(ErrorKind::Other, "this should fail"))
}

fn main() {
    // init the client guard, which will be dropped at the end
    // of the scope.
    let _guard = init(ClientOptions::default());
    let func = some_method_that_fails();

    match func {
        Ok(()) => panic!("expected this to fail")
        Err(report) => {
            capture_report(&report);
        }
    }
}
```

## License
**sentry-eyre** is released under the [MIT License](https://github.com/auguwu/sentry-eyre/blob/master/LICENSE) with love by Noel.
