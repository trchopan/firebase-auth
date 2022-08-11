//! [Firebase](https://firebase.google.com) authentication layer for [Actix Web](https://actix.rs).
//!
//! Provides:
//! - Extractor [FirebaseAuth] for decode and verify user according to [Firebase Document](https://firebase.google.com/docs/auth/admin/verify-id-tokens#verify_id_tokens_using_a_third-party_jwt_library)
//!

pub mod firebase_auth;
