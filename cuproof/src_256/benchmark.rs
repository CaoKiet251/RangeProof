use std::time::{Instant, Duration};
use num_bigint::BigInt;
use crate::setup::{setup_256, fast_test_setup};
use crate::range_proof::{cuproof_prove, proof_size_bytes};
use crate::verify::cuproof_verify;
use crate::util::random_bigint;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub range_length: usize,
    pub setup_time_ms: u128,
    pub prove_time_ms: u128,
    pub verify_time_ms: u128,
    pub proof_size_bytes: usize,
    pub success: bool,
}

fn measure_time_accurate<F>(mut f: F, iterations: usize) -> Duration 
where F: FnMut(),
{
    for _ in 0..3 {
        f();
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let total_time = start.elapsed();
    
    Duration::from_nanos(total_time.as_nanos() as u64 / iterations as u64)
}

pub fn benchmark_range_length(range_length: usize, use_256_setup: bool) -> BenchmarkResult {
    println!("Đang benchmark với {} bit (khoảng [0, 2^{}-1]):", range_length, range_length);
    
    let setup_time = measure_time_accurate(|| {
        let _ = if use_256_setup {
            setup_256()
        } else {
            fast_test_setup()
        };
    }, 5);
    
    let (g, h, n) = if use_256_setup {
        setup_256()
    } else {
        fast_test_setup()
    };
    
    let a = BigInt::from(0);
    let b = BigInt::from(2).pow(range_length as u32) - 1;
    let v = BigInt::from(2).pow(range_length as u32 - 1);
    let r = random_bigint(256);
    
    let prove_time = measure_time_accurate(|| {
        let _proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
    }, 3);
    
    let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
    
    let proof_size = proof_size_bytes(&proof);
    
    let verify_time = measure_time_accurate(|| {
        let _result = cuproof_verify(&proof, &g, &h, &n);
    }, 10);
    
    let verify_result = cuproof_verify(&proof, &g, &h, &n);
    
    BenchmarkResult {
        range_length,
        setup_time_ms: setup_time.as_millis(),
        prove_time_ms: prove_time.as_millis(),
        verify_time_ms: verify_time.as_millis(),
        proof_size_bytes: proof_size,
        success: verify_result,
    }
}

pub fn benchmark_multiple_ranges(range_lengths: Vec<usize>, use_256_setup: bool) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    println!("Bắt đầu benchmark cho {} độ dài khoảng khác nhau", range_lengths.len());
    println!("Sử dụng {} setup", if use_256_setup { "256-bit" } else { "fast" });
    println!("{}", "=".repeat(80));
    
    for &range_length in &range_lengths {
        let result = benchmark_range_length(range_length, use_256_setup);
        results.push(result.clone());
        
        print_benchmark_result(&result);
        println!("{}", "=".repeat(80));
    }
    
    results
}

pub fn print_benchmark_result(result: &BenchmarkResult) {
    println!("Kết quả benchmark cho {} bit (khoảng [0, 2^{}-1]):", result.range_length, result.range_length);
    println!("  ✓ Thời gian setup: {:.2} ms", result.setup_time_ms as f64);
    println!("  ✓ Thời gian tạo proof: {:.2} ms", result.prove_time_ms as f64);
    println!("  ✓ Thời gian verify: {:.2} ms", result.verify_time_ms as f64);
    println!("  ✓ Kích thước proof: {} bytes ({:.2} KB)", 
             result.proof_size_bytes, 
             (result.proof_size_bytes as f64 / 1024.0 * 100.0).round() / 100.0);
    println!("  ✓ Trạng thái: {}", if result.success { "THÀNH CÔNG" } else { "THẤT BẠI" });
}

pub fn print_benchmark_summary(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(80));
    println!("TỔNG KẾT BENCHMARK");
    println!("{}", "=".repeat(80));
    
    println!("{:<12} {:<15} {:<15} {:<15} {:<15} {:<10}",
             "Số bit", "Setup(ms)", "Prove(ms)", "Verify(ms)", "Size(bytes)", "Trạng thái");
    println!("{}", "-".repeat(90));
    
    for result in results {
        println!("{:<12} {:<15.2} {:<15.2} {:<15.2} {:<15} {:<10}", 
                 result.range_length,
                 result.setup_time_ms as f64,
                 result.prove_time_ms as f64,
                 result.verify_time_ms as f64,
                 result.proof_size_bytes,
                 if result.success { "OK" } else { "FAIL" });
    }
    
    let total_setup_time: u128 = results.iter().map(|r| r.setup_time_ms).sum();
    let total_prove_time: u128 = results.iter().map(|r| r.prove_time_ms).sum();
    let total_verify_time: u128 = results.iter().map(|r| r.verify_time_ms).sum();
    let avg_proof_size: f64 = results.iter().map(|r| r.proof_size_bytes).sum::<usize>() as f64 / results.len() as f64;
    
    println!("{}", "-".repeat(90));
    println!("Tổng thời gian setup: {:.2} ms", total_setup_time as f64);
    println!("Tổng thời gian prove: {:.2} ms", total_prove_time as f64);
    println!("Tổng thời gian verify: {:.2} ms", total_verify_time as f64);
    println!("Kích thước proof trung bình: {:.2} bytes ({:.2} KB)", avg_proof_size, (avg_proof_size / 1024.0 * 100.0).round() / 100.0);
    
    println!("\nPHÂN TÍCH XU HƯỚNG:");
    if results.len() >= 2 {
        let first_prove = results[0].prove_time_ms as f64;
        let last_prove = results[results.len()-1].prove_time_ms as f64;
        let prove_growth = (last_prove / first_prove - 1.0) * 100.0;
        
        let first_size = results[0].proof_size_bytes as f64;
        let last_size = results[results.len()-1].proof_size_bytes as f64;
        let size_growth = (last_size / first_size - 1.0) * 100.0;
        
        println!("  • Thời gian prove tăng {:.2}% từ {} bit đến {} bit", 
                 prove_growth, results[0].range_length, results[results.len()-1].range_length);
        println!("  • Kích thước proof tăng {:.2}% từ {} bit đến {} bit", 
                 size_growth, results[0].range_length, results[results.len()-1].range_length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_small_range() {
        let result = benchmark_range_length(8, true);
        assert!(result.success);
        assert!(result.prove_time_ms > 0);
        assert!(result.verify_time_ms > 0);
        assert!(result.proof_size_bytes > 0);
    }

    #[test]
    fn test_benchmark_multiple_ranges() {
        let range_lengths = vec![8, 16, 32];
        let results = benchmark_multiple_ranges(range_lengths, true);
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.success);
        }
    }
}

