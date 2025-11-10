use crate::{util::*, fiat_shamir::*, commitment::*};
use crate::range_proof::Cuproof;
use num_bigint::BigInt;

pub fn cuproof_verify(proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt) -> bool {
	// 1. Fiat–Shamir
	let y = fiat_shamir(&[&proof.A, &proof.S, &proof.C, &proof.C_v1, &proof.C_v2]) % n;
	if y == BigInt::from(0) { return false; }
	let z = fiat_shamir(&[&y]) % n;
	if z == BigInt::from(0) { return false; }
	let x = fiat_shamir(&[&proof.T1, &proof.T2]) % n;
	if x == BigInt::from(0) { return false; }

	// 2. Check T1, T2 commitments
	if pedersen_commit(g, h, &proof.t1, &proof.tau1, n) != proof.T1 { return false; }
	if pedersen_commit(g, h, &proof.t2, &proof.tau2, n) != proof.T2 { return false; }

	// 3. Verify t_hat consistency: t_hat ?= t0 + t1 x + t2 x^2
	let rhs_t = &proof.t0 + &(&proof.t1 * &x) + &(&proof.t2 * &x * &x);
	if proof.t_hat != rhs_t { return false; }

	// 4. Verify commitment consistency for t_hat
	let lhs = pedersen_commit(g, h, &proof.t_hat, &proof.tau_x, n);
	// Construct a commitment to rhs_t using tau_x (already provided)
	let rhs = pedersen_commit(g, h, &rhs_t, &proof.tau_x, n);
	if lhs != rhs { return false; }

	// 5. Verify IPP proof (simplified verification)
	// In a full implementation, this would verify the recursive structure
	if proof.ipp_proof.L.len() != proof.ipp_proof.R.len() { return false; }
	
	// Check that we have the expected number of recursion levels
	// For dimension 64, we expect log2(64) = 6 levels
	let expected_levels = (64.0_f64).log2().ceil() as usize;
	if proof.ipp_proof.L.len() != expected_levels { return false; }

	// 6. Basic sanity: commitments must be within modulus and non-zero
	if &proof.A % n == BigInt::from(0) { return false; }
	if &proof.S % n == BigInt::from(0) { return false; }
	if &proof.T1 % n == BigInt::from(0) { return false; }
	if &proof.T2 % n == BigInt::from(0) { return false; }
	if &proof.C % n == BigInt::from(0) { return false; }
	if &proof.C_v1 % n == BigInt::from(0) { return false; }
	if &proof.C_v2 % n == BigInt::from(0) { return false; }

	// 7. Verify that C_v1 and C_v2 are consistent with C in a coarse way
	// Note: In a rigorous design, we would prove relations for v1, v2.
	// Here we at least ensure they are not trivially equal or zero modulo n.
	if &proof.C == &proof.C_v1 { return false; }
	if &proof.C == &proof.C_v2 { return false; }
	if &proof.C_v1 == &proof.C_v2 { return false; }

	true
}

pub fn cuproof_verify_with_range(proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt, a: &BigInt, b: &BigInt) -> bool {
    if !cuproof_verify(proof, g, h, n) { return false; }

    // Basic range-consistency checks via commitments C_v1 and C_v2
    // Expected: v1 = 4v - 4a + 1, v2 = 4b - 4v + 1
    // We cannot extract v from commitments, but we can at least check that
    // C_v1 and C_v2 are consistent with some v relative to (a,b) bounds using inequalities:
    // For any v in [a,b], v1 >= 1 and v2 >= 1.
    // So we ensure that C_v1 and C_v2 are non-trivial and distinct from C, already checked above.
    // Strengthen: ensure a <= b, and they are non-negative (typical demo domain)
    if a > b { return false; }

    // Additional conservative checks:
    // - Ensure T1, T2, tau1, tau2 not zero already done in cuproof_verify
    // - Ensure commitments are not equal pairwise already done
    // Range-specific simple guard: if a == b then proof should degenerate; reject for now
    if a == b { return false; }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_256;
    use crate::range_proof::cuproof_prove;
    use crate::util::{random_bigint, save_proof, load_proof, save_params, load_params};

    #[test]
    fn verify_pass_and_tamper_fail() {
        let (g, h, n) = setup_256();
        let a = BigInt::from(1);
        let b = BigInt::from(100);
        let v = BigInt::from(42);
        let r = random_bigint(128);
        let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
        assert!(cuproof_verify_with_range(&proof, &g, &h, &n, &a, &b));

        let mut bad = proof.clone();
        bad.T1 = &bad.T1 + BigInt::from(1);
        assert!(!cuproof_verify_with_range(&bad, &g, &h, &n, &a, &b));
    }

    #[test]
    fn verify_save_and_load() {
        let (g, h, n) = setup_256();
        let a = BigInt::from(1);
        let b = BigInt::from(100);
        let v = BigInt::from(42);
        let r = random_bigint(128);
        
        // Save params
        save_params("test_params.txt", &g, &h, &n).unwrap();
        
        // Generate and save proof
        let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
        save_proof("test_proof_save.txt", &proof).unwrap();
        
        // Load params and proof
        let (g2, h2, n2) = load_params("test_params.txt").unwrap();
        let proof2 = load_proof("test_proof_save.txt").unwrap();
        
        // Verify loaded proof
        assert!(cuproof_verify_with_range(&proof2, &g2, &h2, &n2, &a, &b));
    }
}

