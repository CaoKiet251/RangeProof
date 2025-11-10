use num_bigint::BigInt;
use sha2::{Digest, Sha256};

pub fn fiat_shamir(inputs: &[&BigInt]) -> BigInt {
    let mut hasher = Sha256::new();
    for i in inputs {
        hasher.update(i.to_str_radix(10).as_bytes());
    }
    let hash = hasher.finalize();
    BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    // Purpose: ensure Fiat–Shamir is deterministic and sensitive to input ordering/content
    // Params: small BigInt inputs
    // Output: equality/inequality assertions
    // Usage: `cargo test -- src::fiat_shamir` or `cargo test`
    #[test]
    fn fs_deterministic_and_order_sensitive() {
        let a = BigInt::from(123);
        let b = BigInt::from(456);
        let h1 = fiat_shamir(&[&a, &b]);
        let h1_again = fiat_shamir(&[&a, &b]);
        assert_eq!(h1, h1_again);

        let h2 = fiat_shamir(&[&b, &a]);
        assert_ne!(h1, h2);

        let c = BigInt::from(457);
        let h3 = fiat_shamir(&[&a, &c]);
        assert_ne!(h1, h3);
    }
}