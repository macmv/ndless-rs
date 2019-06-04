use core::cmp::Ordering;
use core::convert::TryInto;
use core::hash::{Hash, Hasher};
use core::time::Duration;

use crate::libc;

pub use self::inner::{Instant, SystemTime, UNIX_EPOCH};

const NSEC_PER_SEC: u64 = 1_000_000_000;

#[derive(Copy, Clone)]
struct Timespec {
	t: libc::timespec,
}

impl Timespec {
	const fn zero() -> Timespec {
		Timespec {
			t: libc::timespec {
				tv_sec: 0,
				tv_nsec: 0,
			},
		}
	}

	fn sub_timespec(&self, other: &Timespec) -> Result<Duration, Duration> {
		if self >= other {
			Ok(if self.t.tv_nsec >= other.t.tv_nsec {
				Duration::new(
					(self.t.tv_sec - other.t.tv_sec) as u64,
					(self.t.tv_nsec - other.t.tv_nsec) as u32,
				)
			} else {
				Duration::new(
					(self.t.tv_sec - 1 - other.t.tv_sec) as u64,
					self.t.tv_nsec as u32 + (NSEC_PER_SEC as u32) - other.t.tv_nsec as u32,
				)
			})
		} else {
			match other.sub_timespec(self) {
				Ok(d) => Err(d),
				Err(d) => Ok(d),
			}
		}
	}

	fn checked_add_duration(&self, other: &Duration) -> Option<Timespec> {
		let mut secs = other
			.as_secs()
			.try_into() // <- target type would be `libc::time_t`
			.ok()
			.and_then(|secs| self.t.tv_sec.checked_add(secs))?;

		// Nano calculations can't overflow because nanos are <1B which fit
		// in a u32.
		let mut nsec = other.subsec_nanos() + self.t.tv_nsec as u32;
		if nsec >= NSEC_PER_SEC as u32 {
			nsec -= NSEC_PER_SEC as u32;
			secs = secs.checked_add(1)?;
		}
		Some(Timespec {
			t: libc::timespec {
				tv_sec: secs,
				tv_nsec: libc::c_long::from(nsec as i32),
			},
		})
	}

	fn checked_sub_duration(&self, other: &Duration) -> Option<Timespec> {
		let mut secs = other
			.as_secs()
			.try_into() // <- target type would be `libc::time_t`
			.ok()
			.and_then(|secs| self.t.tv_sec.checked_sub(secs))?;

		// Similar to above, nanos can't overflow.
		let mut nsec = self.t.tv_nsec as i32 - other.subsec_nanos() as i32;
		if nsec < 0 {
			nsec += NSEC_PER_SEC as i32;
			secs = secs.checked_sub(1)?;
		}
		Some(Timespec {
			t: libc::timespec {
				tv_sec: secs,
				tv_nsec: libc::c_long::from(nsec),
			},
		})
	}
}

impl PartialEq for Timespec {
	fn eq(&self, other: &Timespec) -> bool {
		self.t.tv_sec == other.t.tv_sec && self.t.tv_nsec == other.t.tv_nsec
	}
}

impl Eq for Timespec {}

impl PartialOrd for Timespec {
	fn partial_cmp(&self, other: &Timespec) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Timespec {
	fn cmp(&self, other: &Timespec) -> Ordering {
		let me = (self.t.tv_sec, self.t.tv_nsec);
		let other = (other.t.tv_sec, other.t.tv_nsec);
		me.cmp(&other)
	}
}

impl Hash for Timespec {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.t.tv_sec.hash(state);
		self.t.tv_nsec.hash(state);
	}
}

mod inner {
	use core::fmt;

	use crate::file_io::sys::cvt;
	use crate::libc;
	use crate::time::Duration;

	use super::Timespec;

	#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Instant {
		t: Timespec,
	}

	#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SystemTime {
		t: Timespec,
	}

	pub const UNIX_EPOCH: SystemTime = SystemTime {
		t: Timespec::zero(),
	};

	impl Instant {
		pub fn now() -> Instant {
			use core::ptr;

			let mut s = libc::timeval {
				tv_sec: 0,
				tv_usec: 0,
			};
			cvt(unsafe { libc::gettimeofday(&mut s, ptr::null_mut()) }).unwrap();
			Instant {
				t: Timespec {
					t: libc::timespec {
						tv_sec: s.tv_sec,
						tv_nsec: s.tv_usec * 1000,
					},
				},
			}
		}

		pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
			self.t.sub_timespec(&other.t).ok()
		}

		pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
			Some(Instant {
				t: self.t.checked_add_duration(other)?,
			})
		}

		pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
			Some(Instant {
				t: self.t.checked_sub_duration(other)?,
			})
		}
	}

	impl fmt::Debug for Instant {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Instant")
				.field("tv_sec", &self.t.t.tv_sec)
				.field("tv_nsec", &self.t.t.tv_nsec)
				.finish()
		}
	}

	impl SystemTime {
		pub fn now() -> SystemTime {
			use core::ptr;

			let mut s = libc::timeval {
				tv_sec: 0,
				tv_usec: 0,
			};
			cvt(unsafe { libc::gettimeofday(&mut s, ptr::null_mut()) }).unwrap();
			SystemTime::from(s)
		}

		pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
			self.t.sub_timespec(&other.t)
		}

		pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
			Some(SystemTime {
				t: self.t.checked_add_duration(other)?,
			})
		}

		pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
			Some(SystemTime {
				t: self.t.checked_sub_duration(other)?,
			})
		}
	}

	impl From<libc::timeval> for SystemTime {
		fn from(t: libc::timeval) -> SystemTime {
			SystemTime {
				t: Timespec {
					t: libc::timespec {
						tv_sec: t.tv_sec,
						tv_nsec: t.tv_usec * 1000,
					},
				},
			}
		}
	}

	impl From<libc::timespec> for SystemTime {
		fn from(t: libc::timespec) -> SystemTime {
			SystemTime { t: Timespec { t } }
		}
	}

	impl From<libc::c_uint> for SystemTime {
		fn from(t: libc::c_uint) -> SystemTime {
			libc::timespec {
				tv_sec: t as libc::c_long,
				tv_nsec: 0,
			}
			.into()
		}
	}

	impl fmt::Debug for SystemTime {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("SystemTime")
				.field("tv_sec", &self.t.t.tv_sec)
				.field("tv_nsec", &self.t.t.tv_nsec)
				.finish()
		}
	}
}
