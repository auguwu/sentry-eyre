// 🐻‍❄️👀 sentry-eyre: Sentry integration for `eyre`.
// Copyright (c) 2023-2026 Noel Towa <cutie@floofy.dev>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
//! **sentry-eyre** is a integration to capture [`eyre::Report`](https://docs.rs/eyre/latest/eyre/struct.Report.html)s. This crate
//! was inspired by the `sentry-anyhow` integration, and does a similar API but distinct enough to not create any issues.
//!
//! ## Usage
//! ```no_run
//! use eyre::Result;
//! use sentry::{ClientOptions, init, types::Dsn};
//! use sentry_eyre::capture_report;
//!
//! fn method_that_might_fail() -> Result<()> {
//!     Err(eyre::eyre!("this method has failed."))
//! }
//!
//! if let Err(e) = method_that_might_fail() {
//!     capture_report(&e);
//! }
//! ```

#![doc(html_logo_url = "https://cdn.floofy.dev/images/Noel.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/Noel.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]

use eyre::Report;
use sentry_core::{protocol::Event, types::Uuid, Hub};
use std::error::Error;

/// Captures a [`Report`] and sends it to Sentry. Refer to the top-level
/// module documentation on how to use this method.
pub fn capture_report(report: &Report) -> Uuid {
    Hub::with_active(|hub| hub.capture_report(report))
}

/// Utility function to represent a Sentry [`Event`] from a [`Report`]. This shouldn't
/// be consumed directly unless you want access to the created [`Event`] from a [`Report`].
pub fn event_from_report(report: &Report) -> Event<'static> {
    let err: &dyn Error = report.as_ref();

    // It's not mutated for not(feature = "stable-backtrace")
    #[allow(unused_mut)]
    let mut event = sentry_core::event_from_error(err);

    #[cfg(feature = "stable-backtrace")]
    {
        // exception records are sorted in reverse
        if let Some(exc) = event.exception.iter_mut().last() {
            use stable_eyre::BacktraceExt;
            if let Some(backtrace) = report.backtrace() {
                exc.stacktrace = sentry_backtrace::parse_stacktrace(&format!("{backtrace:#?}"));
            }
        }
    }

    event
}

/// Extension trait to implement a `capture_report` method on any implementations.
pub trait CaptureReportExt: private::Sealed {
    /// Captures a [`Report`] and sends it to Sentry. Refer to the top-level
    /// module documentation on how to use this method.
    fn capture_report(&self, report: &Report) -> Uuid;
}

impl CaptureReportExt for Hub {
    fn capture_report(&self, report: &Report) -> Uuid {
        self.capture_event(event_from_report(report))
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for sentry_core::Hub {}
}

#[cfg(all(feature = "stable-backtrace", test))]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_environment() {
        INIT.call_once(|| {
            std::env::set_var("RUST_BACKTRACE", "1");
            stable_eyre::install().unwrap();
        });
    }

    #[test]
    fn test_event_from_report_with_backtrace() {
        init_test_environment();

        let event = event_from_report(&eyre::eyre!("Oh jeez"));

        let stacktrace = event.exception[0].stacktrace.as_ref().unwrap();
        let found_test_fn = stacktrace
            .frames
            .iter()
            .find(|frame| match &frame.function {
                Some(f) => f.contains("test_event_from_report_with_backtrace"),
                None => false,
            });

        assert!(found_test_fn.is_some());
    }

    #[test]
    fn test_capture_eyre_uses_event_from_report_helper() {
        init_test_environment();

        let err = &eyre::eyre!("Oh jeez");

        let event = event_from_report(err);
        let events = sentry::test::with_captured_events(|| {
            capture_report(err);
        });

        assert_eq!(event.exception, events[0].exception);
    }
}
