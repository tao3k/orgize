//! Compile-time ownership selection; both lanes use the same Gerbil parser.

#[cfg(all(feature = "runtime-rust", feature = "runtime-scheme"))]
compile_error!("select one runtime: runtime-rust or runtime-scheme, not both");
#[cfg(not(any(feature = "runtime-rust", feature = "runtime-scheme")))]
compile_error!("select runtime-rust or runtime-scheme (runtime-rust is the default)");
#[cfg(all(feature = "runtime-scheme", not(unix)))]
compile_error!("runtime-scheme currently requires POSIX threads; no implicit fallback");

/// Execution/lifecycle owner, not a parser algorithm selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBackend {
    /// Rust owns the request queue and thread-affine native handle.
    Rust,
    /// A native service owns setup and its single Scheme actor consumer.
    Scheme,
}

/// The execution owner selected when compiling this library.
pub const fn runtime_backend() -> RuntimeBackend {
    if cfg!(feature = "runtime-scheme") {
        RuntimeBackend::Scheme
    } else {
        RuntimeBackend::Rust
    }
}

impl RuntimeBackend {
    /// Stable receipt/benchmark label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "runtime-rust",
            Self::Scheme => "runtime-scheme",
        }
    }
}
