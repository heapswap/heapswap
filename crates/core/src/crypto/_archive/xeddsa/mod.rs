// SPDX-FileCopyrightText: 2023 Dominik George <nik@naturalnet.de>
//
// SPDX-License-Identifier: Apache-2.0

//! # Implementation of the XEdDSA signature algorithm
//!
//! [XEdDSA] is an Elliptic-Curve signature algorithm designed by the
//! [Signal] project. [XEdDSA] is designed to use the same Elliptic-Curve
//! keys both for Diffie-Hellman key exchange and for EdDSA signatures.
//!
//! ## Features
//!
//! The following features of the specification are implemented:
//!
//! - Utility functions to implement specific algorithms
//! - Concrete implementation for XEd25519 (on Curve25519, compatible with [`curve25519-dalek`])
//!
//!
//! [XEdDSA]: https://www.signal.org/docs/specifications/xeddsa/
//! [Signal]: https://signal.org/
//! [`curve25519-dalek`]: curve25519-dalek

pub mod xeddsa;
pub use xeddsa::*;

//#[cfg(feature = "xed25519")]
pub mod xed25519;

mod util;

#[cfg(test)]
mod tests;
