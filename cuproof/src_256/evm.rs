use crate::range_proof::Cuproof;
use crate::util::bigint_to_hex;
use num_bigint::BigInt;
use std::io::{self, Write};

/// Convert BigInt to uint256 (ensure it fits in 256 bits)
/// Applies modulo n first to ensure values are in the correct range
/// Returns the lower 256 bits as a hex string
fn bigint_to_uint256(x: &BigInt, n: &BigInt) -> String {
    let x_mod = x % n;
    let (_sign, bytes) = x_mod.to_bytes_be();
    if bytes.len() > 32 {
        // Take only the last 32 bytes (lower 256 bits)
        let start = bytes.len() - 32;
        hex::encode(&bytes[start..])
    } else {
        // Pad with zeros if needed
        let mut padded = vec![0u8; 32];
        let offset = 32 - bytes.len();
        padded[offset..].copy_from_slice(&bytes);
        hex::encode(&padded)
    }
}

/// Serialize proof to EVM-compatible format
/// Returns a JSON-like structure that can be used in Solidity
/// T1 and T2 are recalculated from modulo'd t1, tau1, t2, tau2 to ensure consistency
pub fn serialize_proof_for_evm(proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt) -> String {
    use crate::commitment::pedersen_commit;
    
    // Apply modulo to t1, tau1, t2, tau2
    let t1_mod = &proof.t1 % n;
    let tau1_mod = &proof.tau1 % n;
    let t2_mod = &proof.t2 % n;
    let tau2_mod = &proof.tau2 % n;
    
    // Recalculate T1 and T2 from modulo'd values to ensure consistency with contract
    let T1_recalc = pedersen_commit(g, h, &t1_mod, &tau1_mod, n);
    let T2_recalc = pedersen_commit(g, h, &t2_mod, &tau2_mod, n);
    
    // Recalculate x from recalculated T1, T2
    use crate::fiat_shamir::fiat_shamir;
    let x_recalc = fiat_shamir(&[&T1_recalc, &T2_recalc]) % n;
    
    // Recalculate t_hat from modulo'd t0, t1, t2 and recalculated x
    let t0_mod = &proof.t0 % n;
    let t_hat_recalc = (&t0_mod + &(&t1_mod * &x_recalc) + &(&t2_mod * &x_recalc * &x_recalc)) % n;
    
    let mut output = String::new();
    
    output.push_str("// Cuproof Proof for EVM (256-bit modulus)\n");
    output.push_str("// Use this data with CuproofVerifier256.sol\n\n");
    
    output.push_str("// Scalars (15 values):\n");
    output.push_str("// [A, S, T1, T2, tau_x, mu, t_hat, C, C_v1, C_v2, t0, t1, t2, tau1, tau2]\n");
    output.push_str("uint256[15] memory scalars = [\n");
    
    // Recalculate tau_x from modulo'd tau1, tau2 and recalculated x
    let tau_x_recalc = (&tau2_mod * &x_recalc * &x_recalc + &tau1_mod * &x_recalc) % n;
    
    // Recalculate mu (mu = alpha + rho * x, but we don't have alpha, rho here)
    // Actually, mu is already in proof, but we need to ensure it's consistent
    // For now, keep original mu and tau_x, but use recalculated t_hat
    
    // Export with recalculated T1, T2, t_hat and modulo'd t0, t1, t2, tau1, tau2
    let scalars = vec![
        &proof.A, &proof.S, &T1_recalc, &T2_recalc, &tau_x_recalc,
        &proof.mu, &t_hat_recalc, &proof.C, &proof.C_v1, &proof.C_v2,
        &t0_mod, &t1_mod, &t2_mod, &tau1_mod, &tau2_mod,
    ];
    
    for (i, scalar) in scalars.iter().enumerate() {
        let hex_val = bigint_to_uint256(scalar, n);
        output.push_str(&format!("    uint256(0x{}),", hex_val));
        if i < scalars.len() - 1 {
            output.push_str(" // ");
            output.push_str(match i {
                0 => "A",
                1 => "S",
                2 => "T1",
                3 => "T2",
                4 => "tau_x",
                5 => "mu",
                6 => "t_hat",
                7 => "C",
                8 => "C_v1",
                9 => "C_v2",
                10 => "t0",
                11 => "t1",
                12 => "t2",
                13 => "tau1",
                14 => "tau2",
                _ => "",
            });
        }
        output.push('\n');
    }
    output.push_str("];\n\n");
    
    output.push_str("// IPP Proof L vector:\n");
    output.push_str(&format!("uint256[] memory ipp_L = new uint256[]({});\n", proof.ipp_proof.L.len()));
    for (i, l_val) in proof.ipp_proof.L.iter().enumerate() {
        let hex_val = bigint_to_uint256(l_val, n);
        output.push_str(&format!("ipp_L[{}] = uint256(0x{});\n", i, hex_val));
    }
    output.push('\n');
    
    output.push_str("// IPP Proof R vector:\n");
    output.push_str(&format!("uint256[] memory ipp_R = new uint256[]({});\n", proof.ipp_proof.R.len()));
    for (i, r_val) in proof.ipp_proof.R.iter().enumerate() {
        let hex_val = bigint_to_uint256(r_val, n);
        output.push_str(&format!("ipp_R[{}] = uint256(0x{});\n", i, hex_val));
    }
    output.push('\n');
    
    output.push_str("// IPP Proof scalars:\n");
    let a_hex = bigint_to_uint256(&proof.ipp_proof.a, n);
    let b_hex = bigint_to_uint256(&proof.ipp_proof.b, n);
    output.push_str(&format!("uint256 ipp_a = uint256(0x{});\n", a_hex));
    output.push_str(&format!("uint256 ipp_b = uint256(0x{});\n", b_hex));
    
    output
}

/// Export proof to JSON format for JavaScript/TypeScript integration
/// T1 and T2 are recalculated from modulo'd t1, tau1, t2, tau2 to ensure consistency
pub fn export_proof_json(proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt) -> String {
    use crate::commitment::pedersen_commit;
    
    // Apply modulo to t1, tau1, t2, tau2
    let t1_mod = &proof.t1 % n;
    let tau1_mod = &proof.tau1 % n;
    let t2_mod = &proof.t2 % n;
    let tau2_mod = &proof.tau2 % n;
    
    // Recalculate T1 and T2 from modulo'd values
    let T1_recalc = pedersen_commit(g, h, &t1_mod, &tau1_mod, n);
    let T2_recalc = pedersen_commit(g, h, &t2_mod, &tau2_mod, n);
    
    // Recalculate x from recalculated T1, T2
    use crate::fiat_shamir::fiat_shamir;
    let x_recalc = fiat_shamir(&[&T1_recalc, &T2_recalc]) % n;
    
    // Recalculate t_hat and tau_x from modulo'd values
    let t0_mod = &proof.t0 % n;
    let t_hat_recalc = (&t0_mod + &(&t1_mod * &x_recalc) + &(&t2_mod * &x_recalc * &x_recalc)) % n;
    let tau_x_recalc = (&tau2_mod * &x_recalc * &x_recalc + &tau1_mod * &x_recalc) % n;
    
    let mut json = String::new();
    json.push_str("{\n");
    
    json.push_str("  \"scalars\": [\n");
    let scalars = vec![
        &proof.A, &proof.S, &T1_recalc, &T2_recalc, &tau_x_recalc,
        &proof.mu, &t_hat_recalc, &proof.C, &proof.C_v1, &proof.C_v2,
        &t0_mod, &t1_mod, &t2_mod, &tau1_mod, &tau2_mod,
    ];
    for (i, scalar) in scalars.iter().enumerate() {
        let hex_val = bigint_to_uint256(scalar, n);
        json.push_str(&format!("    \"0x{}\"", hex_val));
        if i < scalars.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ],\n");
    
    json.push_str("  \"ipp_L\": [\n");
    for (i, l_val) in proof.ipp_proof.L.iter().enumerate() {
        let hex_val = bigint_to_uint256(l_val, n);
        json.push_str(&format!("    \"0x{}\"", hex_val));
        if i < proof.ipp_proof.L.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ],\n");
    
    json.push_str("  \"ipp_R\": [\n");
    for (i, r_val) in proof.ipp_proof.R.iter().enumerate() {
        let hex_val = bigint_to_uint256(r_val, n);
        json.push_str(&format!("    \"0x{}\"", hex_val));
        if i < proof.ipp_proof.R.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  ],\n");
    
    let a_hex = bigint_to_uint256(&proof.ipp_proof.a, n);
    let b_hex = bigint_to_uint256(&proof.ipp_proof.b, n);
    json.push_str(&format!("  \"ipp_a\": \"0x{}\",\n", a_hex));
    json.push_str(&format!("  \"ipp_b\": \"0x{}\"\n", b_hex));
    
    json.push_str("}\n");
    json
}

/// Save proof in EVM-compatible format to file
pub fn save_proof_for_evm(path: &str, proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt) -> io::Result<()> {
    let content = serialize_proof_for_evm(proof, g, h, n);
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Save proof in JSON format for JavaScript integration
pub fn save_proof_json(path: &str, proof: &Cuproof, g: &BigInt, h: &BigInt, n: &BigInt) -> io::Result<()> {
    let content = export_proof_json(proof, g, h, n);
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_256;
    use crate::range_proof::cuproof_prove;
    use crate::util::random_bigint;
    use num_bigint::BigInt;

    #[test]
    fn test_serialize_proof() {
        let (g, h, n) = setup_256();
        let a = BigInt::from(1);
        let b = BigInt::from(100);
        let v = BigInt::from(42);
        let r = random_bigint(128);
        let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
        
        let evm_format = serialize_proof_for_evm(&proof, &g, &h, &n);
        assert!(evm_format.contains("scalars"));
        assert!(evm_format.contains("ipp_L"));
        assert!(evm_format.contains("ipp_R"));
        
        let json_format = export_proof_json(&proof, &g, &h, &n);
        assert!(json_format.contains("\"scalars\""));
        assert!(json_format.contains("\"ipp_L\""));
    }
}

