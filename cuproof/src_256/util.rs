use num_bigint::{BigInt, RandBigInt};
use num_traits::Signed;
use rand::rngs::OsRng;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use crate::range_proof::Cuproof;

pub fn random_bigint(bits: usize) -> BigInt {
    let mut rng = OsRng;
    rng.gen_bigint(bits as u64).abs()
}

pub fn inner_product(a: &[BigInt], b: &[BigInt]) -> BigInt {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn bigint_to_hex(x: &BigInt) -> String {
    let (_sign, bytes) = x.to_bytes_be();
    hex::encode(bytes)
}

pub fn hex_to_bigint(s: &str) -> BigInt {
    let trimmed = s.trim();
    let cleaned = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        &trimmed[2..]
    } else {
        trimmed
    };
    let bytes = hex::decode(cleaned).unwrap_or_default();
    BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes)
}

fn hex_to_bigint_strict(s: &str) -> io::Result<BigInt> {
    let t = s.trim();
    if t.is_empty() { return Err(io::Error::new(io::ErrorKind::InvalidData, "empty hex")); }
    let bytes = hex::decode(t).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid hex"))?;
    if bytes.is_empty() { return Err(io::Error::new(io::ErrorKind::InvalidData, "zero-length bytes")); }
    Ok(BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes))
}

fn write_lines(path: &str, lines: &[String]) -> io::Result<()> {
    if let Some(parent) = Path::new(path).parent() { fs::create_dir_all(parent)?; }
    let mut f = fs::File::create(path)?;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 { f.write_all(b"\n")?; }
        f.write_all(line.as_bytes())?;
    }
    Ok(())
}

fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

pub fn save_params(path: &str, g: &BigInt, h: &BigInt, n: &BigInt) -> io::Result<()> {
    let lines = vec![
        bigint_to_hex(g),
        bigint_to_hex(h),
        bigint_to_hex(n),
    ];
    write_lines(path, &lines)
}

pub fn load_params(path: &str) -> io::Result<(BigInt, BigInt, BigInt)> {
    let lines = read_lines(path)?;
    if lines.len() < 3 { return Err(io::Error::new(io::ErrorKind::InvalidData, "params file too short")); }
    let g = hex_to_bigint_strict(&lines[0])?;
    let h = hex_to_bigint_strict(&lines[1])?;
    let n = hex_to_bigint_strict(&lines[2])?;
    Ok((g, h, n))
}

pub fn save_proof(path: &str, proof: &Cuproof) -> io::Result<()> {
    let mut lines = Vec::new();
    lines.push(bigint_to_hex(&proof.A));
    lines.push(bigint_to_hex(&proof.S));
    lines.push(bigint_to_hex(&proof.T1));
    lines.push(bigint_to_hex(&proof.T2));
    lines.push(bigint_to_hex(&proof.tau_x));
    lines.push(bigint_to_hex(&proof.mu));
    lines.push(bigint_to_hex(&proof.t_hat));
    lines.push(bigint_to_hex(&proof.C));
    lines.push(bigint_to_hex(&proof.C_v1));
    lines.push(bigint_to_hex(&proof.C_v2));
    lines.push(bigint_to_hex(&proof.t0));
    lines.push(bigint_to_hex(&proof.t1));
    lines.push(bigint_to_hex(&proof.t2));
    lines.push(bigint_to_hex(&proof.tau1));
    lines.push(bigint_to_hex(&proof.tau2));
    lines.push(proof.ipp_proof.L.len().to_string());
    for x in &proof.ipp_proof.L { lines.push(bigint_to_hex(x)); }
    lines.push(proof.ipp_proof.R.len().to_string());
    for x in &proof.ipp_proof.R { lines.push(bigint_to_hex(x)); }
    lines.push(bigint_to_hex(&proof.ipp_proof.a));
    lines.push(bigint_to_hex(&proof.ipp_proof.b));
    write_lines(path, &lines)
}

pub fn load_proof(path: &str) -> io::Result<Cuproof> {
    let lines = read_lines(path)?;
    let mut i = 0usize;
    let take = |i: &mut usize| -> io::Result<String> {
        let s = lines.get(*i).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unexpected end of file"))?.clone();
        *i += 1;
        Ok(s)
    };

    let A = hex_to_bigint_strict(&take(&mut i)?)?;
    let S = hex_to_bigint_strict(&take(&mut i)?)?;
    let T1 = hex_to_bigint_strict(&take(&mut i)?)?;
    let T2 = hex_to_bigint_strict(&take(&mut i)?)?;
    let tau_x = hex_to_bigint_strict(&take(&mut i)?)?;
    let mu = hex_to_bigint_strict(&take(&mut i)?)?;
    let t_hat = hex_to_bigint_strict(&take(&mut i)?)?;
    let C = hex_to_bigint_strict(&take(&mut i)?)?;
    let C_v1 = hex_to_bigint_strict(&take(&mut i)?)?;
    let C_v2 = hex_to_bigint_strict(&take(&mut i)?)?;
    let t0 = hex_to_bigint_strict(&take(&mut i)?)?;
    let t1 = hex_to_bigint_strict(&take(&mut i)?)?;
    let t2 = hex_to_bigint_strict(&take(&mut i)?)?;
    let tau1 = hex_to_bigint_strict(&take(&mut i)?)?;
    let tau2 = hex_to_bigint_strict(&take(&mut i)?)?;

    let l_len: usize = take(&mut i)?.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid L length"))?;
    if l_len == 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "L length must be > 0")); }
    let mut L_vec = Vec::with_capacity(l_len);
    for _ in 0..l_len { L_vec.push(hex_to_bigint_strict(&take(&mut i)?)?); }
    let r_len: usize = take(&mut i)?.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid R length"))?;
    if r_len == 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "R length must be > 0")); }
    if r_len != l_len { return Err(io::Error::new(io::ErrorKind::InvalidData, "L and R length mismatch")); }
    let mut R_vec = Vec::with_capacity(r_len);
    for _ in 0..r_len { R_vec.push(hex_to_bigint_strict(&take(&mut i)?)?); }

    let a = hex_to_bigint_strict(&take(&mut i)?)?;
    let b = hex_to_bigint_strict(&take(&mut i)?)?;
    let zero = BigInt::from(0);
    if A == zero || S == zero || T1 == zero || T2 == zero { return Err(io::Error::new(io::ErrorKind::InvalidData, "zero scalar in header")); }

    let ipp_proof = crate::range_proof::IPPProof { L: L_vec, R: R_vec, a, b };
    Ok(Cuproof { A, S, T1, T2, tau_x, mu, t_hat, C, C_v1, C_v2, t0, t1, t2, tau1, tau2, ipp_proof })
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn hex_roundtrip_and_inner_product() {
        let x = BigInt::from(123456789u64);
        let hx = bigint_to_hex(&x);
        let x2 = hex_to_bigint(&hx);
        assert_eq!(x, x2);

        let a = vec![BigInt::from(1), BigInt::from(2), BigInt::from(3)];
        let b = vec![BigInt::from(4), BigInt::from(5), BigInt::from(6)];
        let ip = inner_product(&a, &b);
        assert_eq!(ip, BigInt::from(32));
    }
}

