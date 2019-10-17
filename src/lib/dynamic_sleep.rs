//! The dynamic sleep module implements a timer
//! to try and maintain a given timining interval
//! that adheres as close as possible to a
//! specified update rate in a loop

use std::thread;
use std::time::{Duration, Instant};

use log::Level::Warn;

/// Dynamic Sleep
/// The dynamic sleep struct
/// provides a sleep method that
/// does its best to allow a loop
/// to run with the specified frequency
pub struct DynamicSleep {
  /// Tag, used for logging
  tag: String,
  /// Update frequency in Hz
  _freq: u32,
  /// Period in microseconds
  _period_nanos: u32,
  /// Last instand
  last_instant: Option<Instant>,
  /// How many cycles
  cycles: u32,
  /// How many slips / second
  slips: u32,
}

impl DynamicSleep {
  /// Create a new instance with default frequency of 30hz.
  /// If logging is enabled, tag is used to mark the log messages
  pub fn new<S>(tag: S) -> DynamicSleep
  where
    S: Into<String>,
  {
    let hz = 30;
    DynamicSleep {
      tag: tag.into(),
      _freq: hz,
      _period_nanos: hz_to_nanos(hz),
      last_instant: None,
      cycles: 0,
      slips: 0,
    }
  }

  /// Create a new instance with a custom frequency.
  /// If logging is enabled, tag is used to mark the log messages
  pub fn with_freq<S>(tag: S, freq_in_hz: u32) -> DynamicSleep
  where
    S: Into<String>,
  {
    DynamicSleep {
      tag: tag.into(),
      _freq: freq_in_hz,
      _period_nanos: hz_to_nanos(freq_in_hz),
      last_instant: None,
      cycles: 0,
      slips: 0,
    }
  }

  /// Get the freq
  pub fn freq_hz(&self) -> u32 {
    self._freq
  }

  /// Get frequency period in nanos
  pub fn period_nanos(&self) -> u32 {
    self._period_nanos
  }

  /// When called repeatedly in a loop, tries to
  /// sleep the proper amount needed to ensure the loop
  /// runs at the given frequency
  pub fn sleep(&mut self) {
    match self.last_instant {
      None => self.last_instant = Some(Instant::now()),
      Some(last) => {
        self.cycles += 1;
        let now = Instant::now();
        let dur = now.duration_since(last);
        let subsec_nanos = dur.subsec_nanos();
        self.last_instant = Some(now);
        if dur.as_secs() > 0 || subsec_nanos > self._period_nanos {
          self.slips += 1;
          return;
        } else {
          thread::sleep(Duration::new(0, self._period_nanos - subsec_nanos));
        }
        if self.cycles == self._freq && self.slips > 0 {
          // If we have had slippage within the last second ( approx )
          // then we log it.
          if log_enabled!(Warn) {
            warn!("'{}' slipped {} times in last second", self.tag, self.slips);
          }
          self.slips = 0;
          self.cycles = 0;
        }
      }
    }
  }
}

// Convert frequency to nanos
fn hz_to_nanos(freq: u32) -> u32 {
  (1_000_000_000) / freq
}
