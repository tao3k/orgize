//! Request-local diagnostic projection. Disabled builds contain no clocks/TLS.
#[cfg(feature = "runtime-profile")]
mod collector;
#[cfg(feature = "runtime-profile")]
pub use collector::measure;
mod dispatch;
mod transport;
mod worker;
pub(crate) use dispatch::{is_active, stage};
#[cfg(all(feature = "runtime-profile", any(feature = "runtime-scheme", test)))]
pub(crate) use transport::record_transport;
#[cfg(not(feature = "runtime-scheme"))]
pub(crate) use transport::{OwnerReply, Stamp};
pub(crate) use worker::{query_batch_completed, worker};
#[cfg(feature = "runtime-profile")]
mod wire;
#[cfg(feature = "runtime-profile")]
pub(crate) use dispatch::{native_tape, record_native_batch};
#[cfg(feature = "runtime-profile")]
pub(crate) use wire::decode_batch as decode_native_batch;

#[cfg(all(test, feature = "runtime-profile"))]
#[path = "../../tests/unit/runtime_profile.rs"]
mod tests;
