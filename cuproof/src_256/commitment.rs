use num_bigint::BigInt;
use num_traits::Zero;

pub fn mod_exp(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    let base_pos = if base < &BigInt::zero() { -base } else { base.clone() };
    let exp_pos = if exp < &BigInt::zero() { -exp } else { exp.clone() };
    base_pos.modpow(&exp_pos, modulus)
}

/// Pedersen Commitment over RSA group
/// 
/// This function implements the Pedersen hash function:
/// H(m, r) = g^m * h^r mod n
/// 
/// Where:
/// - g, h are generators of the RSA group Z_n^*
/// - n = p * q is the RSA modulus (256-bit for this version)
/// - p, q are 128-bit primes
/// - m is the message/value to commit
/// - r is the random blinding factor
/// 
/// Security properties:
/// - Hiding: commitment reveals no information about m
/// - Binding: computationally infeasible to find (m', r') ≠ (m, r) with H(m', r') = H(m, r)
/// - Homomorphic: H(m1 + m2, r1 + r2) = H(m1, r1) * H(m2, r2)
pub fn pedersen_commit(g: &BigInt, h: &BigInt, m: &BigInt, r: &BigInt, n: &BigInt) -> BigInt {
    mod_exp(g, m, n) * mod_exp(h, r, n) % n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_256;
    use num_bigint::BigInt;

    #[test]
    fn pedersen_basic_properties() {
        let (g, h, n) = setup_256();
        let m1 = BigInt::from(5);
        let r1 = BigInt::from(7);
        let c1 = pedersen_commit(&g, &h, &m1, &r1, &n);
        let c1_again = pedersen_commit(&g, &h, &m1, &r1, &n);
        assert_eq!(c1, c1_again);

        let c1_diff_r = pedersen_commit(&g, &h, &m1, &BigInt::from(8), &n);
        assert_ne!(c1, c1_diff_r);

        let m2 = BigInt::from(11);
        let r2 = BigInt::from(3);
        let c2 = pedersen_commit(&g, &h, &m2, &r2, &n);
        let lhs = c1 * c2 % &n;
        let rhs = pedersen_commit(&g, &h, &(m1.clone()+m2.clone()), &(r1.clone()+r2.clone()), &n);
        assert_eq!(lhs, rhs);
    }
}

