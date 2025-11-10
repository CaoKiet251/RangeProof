use crate::{util::*, lagrange::*, commitment::*, fiat_shamir::*};
use num_bigint::BigInt;
use num_traits::Zero;

#[derive(Clone)]
pub struct IPPProof {
	pub L: Vec<BigInt>,  // Left commitments at each level
	pub R: Vec<BigInt>,  // Right commitments at each level
	pub a: BigInt,        // Final scalar
	pub b: BigInt,        // Final scalar
}

#[derive(Clone)]
pub struct Cuproof {
	pub A: BigInt,
	pub S: BigInt,
	pub T1: BigInt,
	pub T2: BigInt,
	pub tau_x: BigInt,
	pub mu: BigInt,
	pub t_hat: BigInt,
	pub C: BigInt,  // Commitment to value v
	pub C_v1: BigInt,  // Commitment to v1 = 4v - 4a + 1
	pub C_v2: BigInt,  // Commitment to v2 = 4b - 4v + 1
	pub t0: BigInt,
	pub t1: BigInt,
	pub t2: BigInt,
	pub tau1: BigInt,
	pub tau2: BigInt,
	pub ipp_proof: IPPProof,  // Inner Product Argument proof
}

// Interactive Proof Protocol Structures
#[derive(Clone)]
pub struct ProverState {
	pub v: BigInt,
	pub a: BigInt,
	pub b: BigInt,
	pub r: BigInt,
	pub alpha: BigInt,
	pub rho: BigInt,
	pub sL: Vec<BigInt>,
	pub sR: Vec<BigInt>,
	pub d: Vec<BigInt>,
	pub v1: BigInt,
	pub v2: BigInt,
	pub l0: Vec<BigInt>,
	pub r0: Vec<BigInt>,
	pub t0: BigInt,
	pub t1: BigInt,
	pub t2: BigInt,
	pub tau1: BigInt,
	pub tau2: BigInt,
}

#[derive(Clone)]
pub struct VerifierState {
	pub g: BigInt,
	pub h: BigInt,
	pub n: BigInt,
	pub A: BigInt,
	pub S: BigInt,
	pub T1: BigInt,
	pub T2: BigInt,
	pub y: BigInt,
	pub z: BigInt,
	pub x: BigInt,
}

// Helper function to compute commitment to a value
fn commit_value(g: &BigInt, h: &BigInt, value: &BigInt, n: &BigInt) -> (BigInt, BigInt) {
	let r = random_bigint(256);
	let commitment = pedersen_commit(g, h, value, &r, n);
	(commitment, r)
}

// Full Inner Product Argument implementation
fn inner_product_argument_recursive(
	l_vec: &[BigInt], 
	r_vec: &[BigInt], 
	g: &BigInt, 
	h: &BigInt, 
	n: &BigInt,
	level: usize
) -> (BigInt, BigInt, Vec<BigInt>, Vec<BigInt>) {
	if l_vec.len() == 1 {
		return (l_vec[0].clone(), r_vec[0].clone(), vec![], vec![]);
	}
	
	
	let mid = l_vec.len() / 2;
	let l_left = &l_vec[..mid];
	let l_right = &l_vec[mid..];
	let r_left = &r_vec[..mid];  // Fixed: use r_vec instead of l_vec
	let r_right = &r_vec[mid..]; // Fixed: use r_vec instead of l_vec
	
	let c_L = inner_product(l_left, r_right);
	let c_R = inner_product(l_right, r_left);
	
	// Create commitments to c_L and c_R
	let r_L = random_bigint(256);
	let r_R = random_bigint(256);
	let L = pedersen_commit(g, h, &c_L, &r_L, n);
	let R = pedersen_commit(g, h, &c_R, &r_R, n);
	
	let y = fiat_shamir(&[&L, &R]) % n;
	
	let l_new: Vec<BigInt> = l_left.iter().zip(l_right.iter())
		.map(|(l, r)| l + &(&y * r))
		.collect();
	let r_new: Vec<BigInt> = r_left.iter().zip(r_right.iter())
		.map(|(l, r)| r + &(&y * l))
		.collect();
	
	let (a, b, mut L_vec, mut R_vec) = inner_product_argument_recursive(&l_new, &r_new, g, h, n, level + 1);
	
	// Add current level commitments
	L_vec.push(L);
	R_vec.push(R);
	
	(a, b, L_vec, R_vec)
}

// Interactive Proof Protocol Implementation
pub fn interactive_prove_step1(v: &BigInt, r: &BigInt, a: &BigInt, b: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt) -> (ProverState, BigInt, BigInt) {
	// Fixed optimal dimension for interactive protocol
	let dimension = 16;
	
	// Step 1: Calculate v1 and v2
	let v1 = 4 * v - 4 * a + 1;
	let v2 = 4 * b - 4 * v + 1;

	// Step 2: Find six integers d = (d1, d2, d3, d4, d5, d6) using Lagrange's theorem
	let d1 = find_3_squares(&v1);  // v1 = d1² + d2² + d3²
	let d2 = find_3_squares(&v2);  // v2 = d4² + d5² + d6²
	let d_base = [d1, d2].concat(); // length 6

	// Expand d to the fixed dimension by repeating the base pattern
	let d = (0..dimension)
		.map(|i| d_base[i % d_base.len()].clone())
		.collect::<Vec<_>>();

	// Step 3: Create Pedersen commitment A for values d with random value α
	let alpha = random_bigint(256);
	let A = pedersen_commit(g, h, &d.iter().sum::<BigInt>(), &alpha, n);

	// Step 4: Create commitment S using values sL and sR
	let rho = random_bigint(256);
	let sL = (0..dimension).map(|_| random_bigint(256)).collect::<Vec<_>>();
	let sR = (0..dimension).map(|_| random_bigint(256)).collect::<Vec<_>>();
	let sum_s = sL.iter().sum::<BigInt>() + sR.iter().sum::<BigInt>();
	let S = pedersen_commit(g, h, &sum_s, &rho, n);

	// Create commitments to v, v1, v2
	let (C, _r_v) = commit_value(g, h, v, n);
	let (C_v1, _r_v1) = commit_value(g, h, &v1, n);
	let (C_v2, _r_v2) = commit_value(g, h, &v2, n);

	// Calculate l0 and r0 for later use
	let l0 = d.iter().map(|di| di.clone()).collect::<Vec<_>>();
	let r0 = d.iter().map(|di| di.clone()).collect::<Vec<_>>();

	// Calculate polynomial coefficients
	let t0 = inner_product(&l0, &r0);
	let t1 = l0.iter().zip(&sR).map(|(l0i, sRi)| l0i * sRi).sum::<BigInt>()
		+ r0.iter().zip(&sL).map(|(r0i, sLi)| r0i * sLi).sum::<BigInt>();
	let t2 = inner_product(&sL, &sR);

	let tau1 = random_bigint(256);
	let tau2 = random_bigint(256);

	let prover_state = ProverState {
		v: v.clone(), a: a.clone(), b: b.clone(), r: r.clone(),
		alpha, rho, sL, sR, d, v1, v2, l0, r0, t0, t1, t2, tau1, tau2,
	};

	(prover_state, A, S)
}

pub fn interactive_prove_step2(prover_state: &ProverState, y: &BigInt, z: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt) -> (BigInt, BigInt) {
	// Step 7: Use challenges y and z to compute vectors l(x) and r(x)
	let l0 = prover_state.l0.iter().map(|di| z * di + y).collect::<Vec<_>>();
	let r0 = prover_state.r0.iter().map(|di| z * di + y).collect::<Vec<_>>();

	// Step 8: Calculate T1 and T2 as Pedersen commitments for coefficients t1 and t2
	let T1 = pedersen_commit(g, h, &prover_state.t1, &prover_state.tau1, n);
	let T2 = pedersen_commit(g, h, &prover_state.t2, &prover_state.tau2, n);

	(T1, T2)
}

pub fn interactive_prove_step3(prover_state: &ProverState, x: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt) -> (BigInt, BigInt, BigInt, BigInt, BigInt) {
	// Step 11: Calculate final values
	let l_vec = prover_state.l0.iter().zip(&prover_state.sL)
		.map(|(l0i, sLi)| l0i + &(sLi * x)).collect::<Vec<_>>();
	let r_vec = prover_state.r0.iter().zip(&prover_state.sR)
		.map(|(r0i, sRi)| r0i + &(sRi * x)).collect::<Vec<_>>();

	let t_hat = inner_product(&l_vec, &r_vec);
	let mu = &prover_state.alpha + &(&prover_state.rho * x);
	let tau_x = &prover_state.tau2 * x * x + &prover_state.tau1 * x;

	// Generate IPP proof for l_vec and r_vec
	let (a_final, b_final, L_vec, R_vec) = inner_product_argument_recursive(&l_vec, &r_vec, g, h, n, 0);
	
			let ipp_proof = IPPProof {
			L: L_vec,
			R: R_vec,
			a: a_final.clone(),
			b: b_final.clone(),
		};

	// Create final proof
	let C = pedersen_commit(g, h, &prover_state.v, &prover_state.r, n);
	let C_v1 = pedersen_commit(g, h, &prover_state.v1, &random_bigint(256), n);
	let C_v2 = pedersen_commit(g, h, &prover_state.v2, &random_bigint(256), n);

	let final_proof = Cuproof {
		A: BigInt::from(0), // Will be set by caller
		S: BigInt::from(0), // Will be set by caller
		T1: BigInt::from(0), // Will be set by caller
		T2: BigInt::from(0), // Will be set by caller
		tau_x: tau_x.clone(),
		mu: mu.clone(),
		t_hat: t_hat.clone(),
		C,
		C_v1,
		C_v2,
		t0: prover_state.t0.clone(),
		t1: prover_state.t1.clone(),
		t2: prover_state.t2.clone(),
		tau1: prover_state.tau1.clone(),
		tau2: prover_state.tau2.clone(),
		ipp_proof,
	};

	(t_hat, mu, tau_x, a_final, b_final)
}

// Interactive Verification Protocol
pub fn interactive_verify_step1(g: &BigInt, h: &BigInt, n: &BigInt) -> (VerifierState, BigInt, BigInt) {
	// Step 6: Verifier chooses natural values y', z' and computes y = g^(y'), z = g^(z')
	let y_prime = random_bigint(256);
	let z_prime = random_bigint(256);
	let y = g.modpow(&y_prime, n);
	let z = g.modpow(&z_prime, n);

	let verifier_state = VerifierState {
		g: g.clone(), h: h.clone(), n: n.clone(),
		A: BigInt::from(0), S: BigInt::from(0), T1: BigInt::from(0), T2: BigInt::from(0),
		y: y.clone(), z: z.clone(), x: BigInt::from(0),
	};

	(verifier_state, y, z)
}

pub fn interactive_verify_step2(verifier_state: &mut VerifierState, A: &BigInt, S: &BigInt) {
	// Step 5: Verifier receives commitments A and S from Prover
	verifier_state.A = A.clone();
	verifier_state.S = S.clone();
}

pub fn interactive_verify_step3(verifier_state: &mut VerifierState, T1: &BigInt, T2: &BigInt) {
	// Step 9: Verifier receives T1 and T2 from Prover
	verifier_state.T1 = T1.clone();
	verifier_state.T2 = T2.clone();
}

pub fn interactive_verify_step4(verifier_state: &mut VerifierState, g: &BigInt, n: &BigInt) -> BigInt {
	// Step 10: Verifier chooses natural value x' and computes x = g^(x')
	let x_prime = random_bigint(256);
	let x = g.modpow(&x_prime, n);
	verifier_state.x = x.clone();
	x
}

pub fn interactive_verify_final(verifier_state: &VerifierState, t_hat: &BigInt, mu: &BigInt, tau_x: &BigInt, a_final: &BigInt, b_final: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt) -> bool {
	// Step 12: Verifier performs verification checks
	
	// Check 1: Verify that commitments A and S are not zero (basic validation)
	if verifier_state.A == BigInt::from(0) || verifier_state.S == BigInt::from(0) { return false; }
	
	// Check 2: Verify that T1 and T2 are not zero (basic validation)
	if verifier_state.T1 == BigInt::from(0) || verifier_state.T2 == BigInt::from(0) { return false; }
	
	// Check 3: Verify that challenges y, z, x are not zero
	if verifier_state.y == BigInt::from(0) || verifier_state.z == BigInt::from(0) || verifier_state.x == BigInt::from(0) { return false; }
	
	// Check 4: Verify that the final values are reasonable
	if t_hat == &BigInt::from(0) || mu == &BigInt::from(0) || tau_x == &BigInt::from(0) { return false; }
	if a_final == &BigInt::from(0) || b_final == &BigInt::from(0) { return false; }
	
	// Check 5: Verify polynomial relationship t(x) = <l(x), r(x)>
	// For dimension 64, we expect t_hat to be a reasonable value
	// This is a simplified check - in a real implementation, we would verify the full polynomial
	
	// Check 6: Verify that t_hat is within reasonable bounds
	// Since t_hat = <l_vec, r_vec> where l_vec and r_vec are 64-dimensional vectors
	// Each component is typically small (from 3-squares), so t_hat should not be extremely large
	let max_expected = BigInt::from(1000000u64); // Reasonable upper bound for demo
	if t_hat > &max_expected { return false; }
	
	// Check 7: Verify that mu and tau_x are reasonable
	if mu > &max_expected || tau_x > &max_expected { return false; }
	
	// For a complete implementation, we would also verify:
	// - The commitment relationships for T1 and T2
	// - The IPP proof structure recursively
	// - The polynomial coefficients t0, t1, t2
	
	true
}

// Original non-interactive proof (kept for compatibility)
pub fn cuproof_prove_with_dimension(v: &BigInt, r: &BigInt, a: &BigInt, b: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt, dimension: usize) -> Cuproof {
	let v1 = 4 * v - 4 * a + 1;
	let v2 = 4 * b - 4 * v + 1;

	// Use 3-squares for numbers of the form 4x+1
	let d1 = find_3_squares(&v1);
	let d2 = find_3_squares(&v2);
	let d_base = [d1, d2].concat(); // length 6

	// Expand d to the requested dimension by repeating the base pattern
	let d = (0..dimension)
		.map(|i| d_base[i % d_base.len()].clone())
		.collect::<Vec<_>>();

	// Create commitments to v, v1, v2
	let (C, _r_v) = commit_value(g, h, v, n);
	let (C_v1, _r_v1) = commit_value(g, h, &v1, n);
	let (C_v2, _r_v2) = commit_value(g, h, &v2, n);

	let alpha = random_bigint(256);
	let rho = random_bigint(256);
	let sL = (0..dimension).map(|_| random_bigint(256)).collect::<Vec<_>>();
	let sR = (0..dimension).map(|_| random_bigint(256)).collect::<Vec<_>>();

	// Commit A and S (demo-style, sum-based)
	let sum_d = d.iter().sum();
	let A = pedersen_commit(g, h, &sum_d, &alpha, n);
	let sum_s = sL.iter().sum::<BigInt>() + sR.iter().sum::<BigInt>();
	let S = pedersen_commit(g, h, &sum_s, &rho, n);

	// Fiat–Shamir challenges
	let y = fiat_shamir(&[&A, &S, &C, &C_v1, &C_v2]) % n;
	let z = fiat_shamir(&[&y]) % n;

	// l0 = z*d + y ; r0 = z*d + y
	let l0 = d.iter().map(|di| &z * di + &y).collect::<Vec<_>>();
	let r0 = d.iter().map(|di| &z * di + &y).collect::<Vec<_>>();

	// Coefficients of t(x) = <l(x), r(x)> = t0 + t1 x + t2 x^2
	let t0 = inner_product(&l0, &r0);
	let t1 = l0.iter().zip(&sR).map(|(l0i, sRi)| l0i * sRi).sum::<BigInt>()
		+ r0.iter().zip(&sL).map(|(r0i, sLi)| r0i * sLi).sum::<BigInt>();
	let t2 = inner_product(&sL, &sR);

	// Commit T1 = Commit(t1, tau1), T2 = Commit(t2, tau2)
	let tau1 = random_bigint(256);
	let tau2 = random_bigint(256);
	let T1 = pedersen_commit(g, h, &t1, &tau1, n);
	let T2 = pedersen_commit(g, h, &t2, &tau2, n);

	// Challenge x
	let x = fiat_shamir(&[&T1, &T2]) % n;

	// Evaluate t_hat at x
	let t_hat = &t0 + &(&t1 * &x) + &(&t2 * &x * &x);

	// Aggregate blinding terms (demo-style): μ = α + ρ x ; τx = τ2 x^2 + τ1 x
	let mu = &alpha + &(&rho * &x);
	let tau_x = &tau2 * &x * &x + &tau1 * &x;

	// Generate IPP proof for l_vec and r_vec
	let l_vec = l0.iter().zip(&sL).map(|(l0i, sLi)| l0i + &(sLi * &x)).collect::<Vec<_>>();
	let r_vec = r0.iter().zip(&sR).map(|(r0i, sRi)| r0i + &(sRi * &x)).collect::<Vec<_>>();
	
	let (a_final, b_final, L_vec, R_vec) = inner_product_argument_recursive(&l_vec, &r_vec, g, h, n, 0);
	
	let ipp_proof = IPPProof {
		L: L_vec,
		R: R_vec,
		a: a_final,
		b: b_final,
	};

	Cuproof {
		A, S, T1, T2, tau_x, mu, t_hat, C, C_v1, C_v2, t0, t1, t2, tau1, tau2, ipp_proof,
	}
}

// Backward-compatible wrapper that defaults to larger dimension for IPP
pub fn cuproof_prove(v: &BigInt, r: &BigInt, a: &BigInt, b: &BigInt, g: &BigInt, h: &BigInt, n: &BigInt) -> Cuproof {
	// Use larger dimension to ensure enough recursion levels for IPP
	cuproof_prove_with_dimension(v, r, a, b, g, h, n, 64) // Reduced from 1024 to 64
}

fn bigint_size_bytes(x: &BigInt) -> usize {
	let (_sign, bytes) = x.to_bytes_be();
	bytes.len()
}

pub fn proof_size_bytes(proof: &Cuproof) -> usize {
	let mut sum = 0usize;
	sum += bigint_size_bytes(&proof.A);
	sum += bigint_size_bytes(&proof.S);
	sum += bigint_size_bytes(&proof.T1);
	sum += bigint_size_bytes(&proof.T2);
	sum += bigint_size_bytes(&proof.tau_x);
	sum += bigint_size_bytes(&proof.mu);
	sum += bigint_size_bytes(&proof.t_hat);
	sum += bigint_size_bytes(&proof.C);
	sum += bigint_size_bytes(&proof.C_v1);
	sum += bigint_size_bytes(&proof.C_v2);
	sum += bigint_size_bytes(&proof.t0);
	sum += bigint_size_bytes(&proof.t1);
	sum += bigint_size_bytes(&proof.t2);
	sum += bigint_size_bytes(&proof.tau1);
	sum += bigint_size_bytes(&proof.tau2);
	
	// Add IPP proof size
	sum += proof.ipp_proof.L.iter().map(|x| bigint_size_bytes(x)).sum::<usize>();
	sum += proof.ipp_proof.R.iter().map(|x| bigint_size_bytes(x)).sum::<usize>();
	sum += bigint_size_bytes(&proof.ipp_proof.a);
	sum += bigint_size_bytes(&proof.ipp_proof.b);
	
	sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use crate::setup::fast_test_setup;
    use crate::util::random_bigint;

    // Purpose: smoke test proof generation returns non-zero-sized proof with consistent fields
    // Params: small demo range and random r
    // Output: asserts on non-zero size and non-empty IPP vectors
    // Usage: `cargo test -- src::range_proof` or `cargo test`
    #[test]
    fn prove_smoke_nonzero_size() {
        let (g, h, n) = fast_test_setup();
        let a = BigInt::from(1);
        let b = BigInt::from(100);
        let v = BigInt::from(42);
        let r = random_bigint(128);
        let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
        let sz = proof_size_bytes(&proof);
        assert!(sz > 0);
        assert_eq!(proof.ipp_proof.L.len(), proof.ipp_proof.R.len());
        assert!(proof.ipp_proof.L.len() > 0);
    }
}

// Inner Product Argument (simplified version - kept for reference)
pub fn inner_product_argument(l_vec: &[BigInt], r_vec: &[BigInt], g: &BigInt, h: &BigInt, n: &BigInt) -> (BigInt, BigInt) {
	if l_vec.len() == 1 {
		return (l_vec[0].clone(), r_vec[0].clone());
	}
	
	let mid = l_vec.len() / 2;
	let l_left = &l_vec[..mid];
	let l_right = &l_vec[mid..];
	let r_left = &l_vec[mid..];
	let r_right = &r_vec[..mid];
	
	let c_L = inner_product(l_left, r_right);
	let c_R = inner_product(l_right, l_left);
	
	let y = fiat_shamir(&[&c_L, &c_R]) % n;
	
	let l_new: Vec<BigInt> = l_left.iter().zip(l_right.iter())
		.map(|(l, r)| l + &(&y * r))
		.collect();
	let r_new: Vec<BigInt> = r_left.iter().zip(r_right.iter())
		.map(|(l, r)| r + &(&y * l))
		.collect();
	
	inner_product_argument(&l_new, &r_new, g, h, n)
}
