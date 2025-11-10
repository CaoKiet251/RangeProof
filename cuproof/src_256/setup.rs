use num_bigint::{BigInt, RandBigInt, Sign, BigUint};
use num_traits::{Signed, Zero, One};
use num_integer::Integer;
use rand::rngs::OsRng;

fn miller_rabin(n: &BigUint, k: u32) -> bool {
    if *n < BigUint::from(2u32) { return false; }
    for p in [2u32,3,5,7,11,13,17,19,23,29,31,37] {
        let p_b = BigUint::from(p);
        if &p_b == n { return true; }
        if n % &p_b == BigUint::zero() { return false; }
    }

    let one = BigUint::one();
    let n_minus_one = n - &one;
    let mut d = n_minus_one.clone();
    let mut r = 0u32;
    while &d % 2u32 == BigUint::zero() { d >>= 1; r += 1; }

    let mut rng = OsRng;
    'witness: for _ in 0..k {
        let two = BigUint::from(2u32);
        let n_minus_two = n - &two;
        if n_minus_two <= two { return true; }
        use rand::RngCore;
        let mut a;
        loop {
            let mut buf = vec![0u8; n.bits() as usize / 8 + 1];
            rng.fill_bytes(&mut buf);
            a = BigUint::from_bytes_be(&buf);
            a = two.clone() + (a % (&n_minus_two - &two + &one));
            if a >= two && a <= n_minus_two { break; }
        }

        let mut x = a.modpow(&d, n);
        if x == one || x == n_minus_one { continue 'witness; }
        for _ in 0..(r-1) {
            x = x.modpow(&two, n);
            if x == n_minus_one { continue 'witness; }
        }
        return false;
    }
    true
}

fn generate_probable_prime(bits: usize) -> BigUint {
    let mut rng = OsRng;
    loop {
        let high = BigUint::one() << (bits.saturating_sub(1) as u32);
        let lower = BigUint::from_bytes_be(&{
            let mut buf = vec![0u8; bits.saturating_sub(1) / 8 + 1];
            use rand::RngCore; rng.fill_bytes(&mut buf); buf
        });
        let mut cand = high.clone() + (lower % &high);
        if &cand % 2u32 == BigUint::zero() { cand += BigUint::one(); }
        if miller_rabin(&cand, 16) { return cand; }
    }
}

pub fn trusted_setup(bits: usize) -> (BigInt, BigInt, BigInt) {
    let mut rng = OsRng;

    let prime_bits = 1024;
    let p = generate_probable_prime(prime_bits);
    let mut q = generate_probable_prime(prime_bits);
    while q == p { q = generate_probable_prime(prime_bits); }
    let n_u = &p * &q;
    let n = BigInt::from_biguint(Sign::Plus, n_u.clone());

    let two = BigInt::from(2u32);
    let one = BigInt::one();
    let mut g;
    loop {
        g = rng.gen_bigint_range(&two, &n);
        if g.gcd(&n) == one { break; }
    }
    let mut h;
    loop {
        h = rng.gen_bigint_range(&two, &n);
        if h.gcd(&n) == one && h != g { break; }
    }

    (g, h, n)
}

pub fn fast_test_setup() -> (BigInt, BigInt, BigInt) {
    let mut rng = OsRng;

    let prime_bits = 256;
    let p = generate_probable_prime(prime_bits);
    let mut q = generate_probable_prime(prime_bits);
    while q == p { q = generate_probable_prime(prime_bits); }
    let n_u = &p * &q;
    let n = BigInt::from_biguint(Sign::Plus, n_u.clone());

    let two = BigInt::from(2u32);
    let one = BigInt::one();
    let mut g;
    loop {
        g = rng.gen_bigint_range(&two, &n);
        if g.gcd(&n) == one { break; }
    }
    let mut h;
    loop {
        h = rng.gen_bigint_range(&two, &n);
        if h.gcd(&n) == one && h != g { break; }
    }

    (g, h, n)
}

/// Setup for 256-bit modulus using 128-bit primes
/// 
/// This function generates RSA-style modulus n = p * q where p and q are 128-bit primes,
/// resulting in a 256-bit modulus suitable for EVM compatibility.
/// 
/// # Returns
/// A tuple (g, h, n) where:
/// - g, h are generators of the RSA group Z_n^*
/// - n is a 256-bit RSA modulus (p * q)
pub fn setup_256() -> (BigInt, BigInt, BigInt) {
    let mut rng = OsRng;

    let prime_bits = 128;
    let p = generate_probable_prime(prime_bits);
    let mut q = generate_probable_prime(prime_bits);
    while q == p { q = generate_probable_prime(prime_bits); }
    let n_u = &p * &q;
    let n = BigInt::from_biguint(Sign::Plus, n_u.clone());

    let two = BigInt::from(2u32);
    let one = BigInt::one();
    let mut g;
    loop {
        g = rng.gen_bigint_range(&two, &n);
        if g.gcd(&n) == one { break; }
    }
    let mut h;
    loop {
        h = rng.gen_bigint_range(&two, &n);
        if h.gcd(&n) == one && h != g { break; }
    }

    (g, h, n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    #[test]
    fn fast_setup_generates_valid_params() {
        let (g, h, n) = fast_test_setup();
        assert!(g.gcd(&n).is_one());
        assert!(h.gcd(&n).is_one());
        assert_ne!(g, h);
        assert!(!n.is_zero());
    }

    #[test]
    fn setup_256_generates_valid_params() {
        let (g, h, n) = setup_256();
        assert!(g.gcd(&n).is_one());
        assert!(h.gcd(&n).is_one());
        assert_ne!(g, h);
        assert!(!n.is_zero());
    }
}

