use num_bigint::BigInt;
use sha3::{Keccak256, Digest};

pub fn fiat_shamir(inputs: &[&BigInt]) -> BigInt {
    let mut hasher = Keccak256::new();
    for i in inputs {
        // Convert BigInt to bytes (big-endian, 32 bytes for uint256)
        let (sign, bytes) = i.to_bytes_be();
        let mut padded = vec![0u8; 32];
        let start = 32usize.saturating_sub(bytes.len());
        padded[start..].copy_from_slice(&bytes);
        // Handle negative by inverting (though in practice all values should be positive)
        if sign == num_bigint::Sign::Minus {
            for b in &mut padded {
                *b = !*b;
            }
        }
        hasher.update(&padded);
    }
    let hash = hasher.finalize();
    BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

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

