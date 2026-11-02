//! The signatures a license key needs, both of them: Ed25519 and ML-DSA-65, each with the
//! product's context. The portal signs them with Go's `crypto/ed25519` and `crypto/mldsa`.

use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use ml_dsa::{EncodedSignature, EncodedVerifyingKey, MlDsa65, Signature, VerifyingKey};
use sha2::{Digest, Sha512};

/// The length of an Ed25519 public key, in bytes.
pub const ED25519_PUBLIC: usize = 32;
/// The length of an ML-DSA-65 public key, in bytes.
pub const MLDSA65_PUBLIC: usize = 1952;

/// Whether `signature` is `public`'s of `message` in Ed25519ctx (RFC 8032, 5.1), which
/// ed25519-dalek has no way to ask for. As Go's: the group equation without the cofactor,
/// `s` below the group's order. Keys and points of small order are refused too.
pub fn ed25519ctx(public: &[u8; 32], message: &[u8], context: &[u8], signature: &[u8]) -> bool {
    let (Ok(signature), Ok(length)) = (
        <&[u8; 64]>::try_from(signature),
        u8::try_from(context.len()),
    ) else {
        return false;
    };
    let (r, s) = signature.split_at(32);
    let Some(a) = CompressedEdwardsY(*public).decompress() else {
        return false;
    };
    let Ok(r) = <[u8; 32]>::try_from(r) else {
        return false;
    };
    let Some(point) = CompressedEdwardsY(r).decompress() else {
        return false;
    };
    let Ok(s) = <[u8; 32]>::try_from(s) else {
        return false;
    };
    let Some(s) = Option::<Scalar>::from(Scalar::from_canonical_bytes(s)) else {
        return false;
    };
    if a.is_small_order() || point.is_small_order() || context.is_empty() {
        return false;
    }
    let mut hash = Sha512::new();
    hash.update(b"SigEd25519 no Ed25519 collisions");
    hash.update([0, length]);
    hash.update(context);
    hash.update(r);
    hash.update(public);
    hash.update(message);
    let mut wide = [0; 64];
    wide.copy_from_slice(&hash.finalize());
    let k = Scalar::from_bytes_mod_order_wide(&wide);
    // [s]B - [k]A == R
    EdwardsPoint::vartime_double_scalar_mul_basepoint(&k, &-a, &s)
        .compress()
        .to_bytes()
        == r
}

/// Whether `signature` is `public`'s of `message` in ML-DSA-65, with `context`.
pub fn mldsa65(public: &[u8; 1952], message: &[u8], context: &[u8], signature: &[u8]) -> bool {
    let Ok(signature) = <[u8; 3309]>::try_from(signature) else {
        return false;
    };
    let public = VerifyingKey::<MlDsa65>::decode(&EncodedVerifyingKey::<MlDsa65>::from(*public));
    let Some(signature) = Signature::decode(&EncodedSignature::<MlDsa65>::from(signature)) else {
        return false;
    };
    public.verify_with_context(message, context, &signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
            .collect()
    }

    /// RFC 8032, 7.2: "TEST 1" of Ed25519ctx, and what must fail next to it.
    #[test]
    fn the_rfcs_example_verifies() {
        let public: [u8; 32] =
            bytes("dfc9425e4f968f7f0c29f0259cf5f9aed6851c2bb4ad8bfb860cfee0ab248292")
                .try_into()
                .expect("32 bytes");
        let message = bytes("f726936d19c800494e3fdaff20b276a8");
        let signature = bytes(
            "55a4cc2f70a54e04288c5f4cd1e45a7bb520b36292911876cada7323198dd87a\
             8b36950b95130022907a7fb7c4e9b2d5f6cca685a587b4b21f4b888e4e7edb0d",
        );
        assert!(ed25519ctx(&public, &message, b"foo", &signature));
        assert!(!ed25519ctx(&public, &message, b"bar", &signature));
        assert!(!ed25519ctx(&public, b"other", b"foo", &signature));
        assert!(!ed25519ctx(&public, &message, b"foo", &signature[..63]));
    }

    #[test]
    fn s_at_or_above_the_order_is_refused() {
        let public: [u8; 32] =
            bytes("dfc9425e4f968f7f0c29f0259cf5f9aed6851c2bb4ad8bfb860cfee0ab248292")
                .try_into()
                .expect("32 bytes");
        let mut signature = bytes(
            "55a4cc2f70a54e04288c5f4cd1e45a7bb520b36292911876cada7323198dd87a\
             8b36950b95130022907a7fb7c4e9b2d5f6cca685a587b4b21f4b888e4e7edb0d",
        );
        signature[32..].fill(0xff);
        assert!(!ed25519ctx(&public, b"", b"foo", &signature));
    }
}
