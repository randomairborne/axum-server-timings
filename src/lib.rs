#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    warnings
)]
#![cfg_attr(
    hide_server_timings,
    allow(
        unused_variables,
        unreachable_code,
        unused_mut,
        dead_code,
        clippy::needless_pass_by_ref_mut,
        clippy::unused_self,
        clippy::needless_pass_by_value
    )
)]
#![doc = include_str!("../README.md")]

use std::{
    borrow::Cow,
    convert::Infallible,
    fmt::{Display, Write},
    time::Instant,
};

use axum_core::response::{IntoResponseParts, ResponseParts};
use http::{HeaderName, HeaderValue};

type TimingString = Cow<'static, str>;

/// Tracker for server timings.
/// Implements [`IntoResponseParts`], so it can be returned at the start of an axum response tuple.
pub struct ServerTimings {
    timings: Vec<Timing>,
    last_timing: Instant,
}

impl ServerTimings {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a raw timing to the internal timing list. The internal duration tracker is **not** updated.
    pub fn add_timing(&mut self, timing: Timing) {
        #[cfg(not(hide_server_timings))]
        self.timings.push(timing);
    }

    /// Record a timing event, with a name and description. These can be string literals or normal strings.
    /// The time since the last event is recorded automatically.
    pub fn record(&mut self, name: impl Into<TimingString>, desc: impl Into<TimingString>) {
        let dur = self.advance_duration();
        self.record_all(name.into(), Some(desc.into()), Some(dur));
    }

    /// Like [`Self::record`], but without a description. Useful for conserving bandwidth.
    pub fn record_name(&mut self, name: impl Into<TimingString>) {
        let dur = self.advance_duration();
        self.record_all(name.into(), None, Some(dur));
    }

    /// Add a timing to the internal timing list. The internal duration tracker is **not** updated.
    pub fn record_all(&mut self, name: TimingString, desc: Option<TimingString>, dur: Option<f64>) {
        let timing = Timing { name, desc, dur };
        self.add_timing(timing);
    }

    #[cfg(hide_server_timings)]
    fn advance_duration(&mut self) -> f64 {
        0.0
    }

    #[cfg(not(hide_server_timings))]
    fn advance_duration(&mut self) -> f64 {
        let now = Instant::now();
        // Browsers want timings in ms
        let dur_since = now.duration_since(self.last_timing).as_secs_f64() * 1000.0;
        self.last_timing = now;
        dur_since
    }
}

impl Default for ServerTimings {
    fn default() -> Self {
        Self {
            timings: Vec::new(),
            last_timing: Instant::now(),
        }
    }
}

/// A representation of a server timing object.
#[derive(Debug, Clone, PartialEq)]
pub struct Timing {
    pub name: Cow<'static, str>,
    pub desc: Option<Cow<'static, str>>,
    /// Time (in ms) operation took
    pub dur: Option<f64>,
}

impl Display for Timing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        if let Some(desc) = &self.desc {
            f.write_str(";desc=\"")?;
            for char in desc.chars() {
                if char == '"' {
                    f.write_str("\\\"")?;
                } else {
                    f.write_char(char)?;
                }
            }
            f.write_char('"')?;
        }
        if let Some(dur) = self.dur {
            write!(f, ";dur={dur}")?;
        }
        Ok(())
    }
}

static TIMINGS_HEADER: HeaderName = HeaderName::from_static("server-timing");

impl IntoResponseParts for ServerTimings {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        #[cfg(hide_server_timings)]
        return Ok(res);
        let mut timing_string = String::new();
        let mut had_error = false;
        for timing in self.timings {
            // we don't want to crash the server because there are no timings.
            // thus, we have manual handling here.
            had_error = had_error || write!(timing_string, "{timing},").is_err();
            if had_error {
                break;
            }
        }
        if let Ok(header) = HeaderValue::from_str(timing_string.trim_end_matches(',')) {
            res.headers_mut().append(TIMINGS_HEADER.clone(), header);
        }
        Ok(res)
    }
}
