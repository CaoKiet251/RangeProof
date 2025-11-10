pub mod setup;
pub mod commitment;
pub mod fiat_shamir;
pub mod lagrange;
pub mod range_proof;
pub mod verify;
pub mod util;
pub mod benchmark;
pub mod evm;

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;
    use std::time::Instant;

    #[test]
    fn test_basic_range_proof() {
        let (g, h, n) = setup::setup_256();
        let a = 10.to_bigint().unwrap();
        let b = 100.to_bigint().unwrap();
        let v = 30.to_bigint().unwrap();
        let r = 42.to_bigint().unwrap();

        let start_prove = Instant::now();
        let proof = range_proof::cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
        let prove_duration = start_prove.elapsed();

        let start_verify = Instant::now();
        let is_valid = verify::cuproof_verify(&proof, &g, &h, &n);
        let verify_duration = start_verify.elapsed();

        println!("Basic Range Proof Timing:");
        println!("  Proof generation time: {:?}", prove_duration);
        println!("  Proof verification time: {:?}", verify_duration);

        assert!(is_valid, "Basic range proof verification failed");
    }

    #[test]
    fn test_multiple_values() {
        let (g, h, n) = setup::setup_256();
        let a = 0.to_bigint().unwrap();
        let b = 1000.to_bigint().unwrap();
        let r = 123.to_bigint().unwrap();

        let test_values = vec![0, 100, 500, 999, 1000];
        let test_values_len = test_values.len();
        
        let mut total_prove_time = std::time::Duration::new(0, 0);
        let mut total_verify_time = std::time::Duration::new(0, 0);
        
        for test_v in test_values {
            let v = test_v.to_bigint().unwrap();
            
            let start_prove = Instant::now();
            let proof = range_proof::cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
            let prove_duration = start_prove.elapsed();
            total_prove_time += prove_duration;
            
            let start_verify = Instant::now();
            let is_valid = verify::cuproof_verify(&proof, &g, &h, &n);
            let verify_duration = start_verify.elapsed();
            total_verify_time += verify_duration;
            
            println!("Value {}: Prove={:?}, Verify={:?}", test_v, prove_duration, verify_duration);
            
            assert!(is_valid, "Proof verification failed for value {}", test_v);
        }
        
        println!("\nMultiple Values Test Summary:");
        println!("  Total proof generation time: {:?}", total_prove_time);
        println!("  Total proof verification time: {:?}", total_verify_time);
        println!("  Average proof generation time: {:?}", total_prove_time / test_values_len as u32);
        println!("  Average proof verification time: {:?}", total_verify_time / test_values_len as u32);
    }

    #[test]
    fn test_different_ranges() {
        let (g, h, n) = setup::setup_256();
        let r = 42.to_bigint().unwrap();

        let test_ranges = vec![
            (0, 100, 50),
            (100, 200, 150),
            (500, 1000, 750),
            (1000, 2000, 1500),
        ];
        let test_ranges_len = test_ranges.len();

        let mut total_prove_time = std::time::Duration::new(0, 0);
        let mut total_verify_time = std::time::Duration::new(0, 0);

        for (a_val, b_val, v_val) in test_ranges {
            let a = a_val.to_bigint().unwrap();
            let b = b_val.to_bigint().unwrap();
            let v = v_val.to_bigint().unwrap();

            let start_prove = Instant::now();
            let proof = range_proof::cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
            let prove_duration = start_prove.elapsed();
            total_prove_time += prove_duration;

            let start_verify = Instant::now();
            let is_valid = verify::cuproof_verify(&proof, &g, &h, &n);
            let verify_duration = start_verify.elapsed();
            total_verify_time += verify_duration;

            println!("Range [{}, {}] with value {}: Prove={:?}, Verify={:?}", 
                     a_val, b_val, v_val, prove_duration, verify_duration);

            assert!(is_valid, "Proof verification failed for range [{}, {}] with value {}", a_val, b_val, v_val);
        }
        
        println!("\nDifferent Ranges Test Summary:");
        println!("  Total proof generation time: {:?}", total_prove_time);
        println!("  Total proof verification time: {:?}", total_verify_time);
        println!("  Average proof generation time: {:?}", total_prove_time / test_ranges_len as u32);
        println!("  Average proof verification time: {:?}", total_verify_time / test_ranges_len as u32);
    }
}

