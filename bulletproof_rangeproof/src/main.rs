use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand::rngs::OsRng;
use rand::RngCore;
use std::time::Instant;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use chrono::{DateTime, Local};

/// Cấu trúc đơn giản để lưu trữ kết quả đo
#[derive(Debug)]
struct MeasurementResult {
    bitsize: usize,
    run_number: usize,
    generation_time_ms: f64,
    verification_time_ms: f64,
    total_time_ms: f64,
    proof_size_bytes: usize,
    success: bool,
}

impl MeasurementResult {
    fn new(
        bitsize: usize,
        run_number: usize,
        gen_time: Duration,
        ver_time: Duration,
        proof_size: usize,
        success: bool,
    ) -> Self {
        Self {
            bitsize,
            run_number,
            generation_time_ms: gen_time.as_micros() as f64 / 1000.0,
            verification_time_ms: ver_time.as_micros() as f64 / 1000.0,
            total_time_ms: (gen_time + ver_time).as_micros() as f64 / 1000.0,
            proof_size_bytes: proof_size,
            success,
        }
    }

    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{:.2},{:.2},{:.2},{},{}\n",
            self.bitsize,
            self.run_number,
            if self.success { "SUCCESS" } else { "FAILED" },
            self.generation_time_ms,
            self.verification_time_ms,
            self.total_time_ms,
            self.proof_size_bytes,
            if self.success { "OK" } else { "ERROR" }
        )
    }
}

/// Thực hiện đo cho một bitsize cụ thể
fn measure_bitsize(bitsize: usize, test_value: u64, runs: usize) -> Vec<MeasurementResult> {
    let mut results = Vec::new();
    
    println!("=== Đo {}-bit range proof ===", bitsize);
    println!("Giá trị test: {}", test_value);
    println!("Số lần chạy: {}", runs);
    
    // Validate test_value
    let max_value = if bitsize >= 64 {
        u64::MAX
    } else {
        (1u64 << bitsize) - 1
    };
    
    if test_value > max_value {
        panic!("Giá trị test {} vượt quá giới hạn cho {}-bit range (max: {})", test_value, bitsize, max_value);
    }
    
    // Tạo generators
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(bitsize, 1);
    
    for run in 1..=runs {
        println!("  Lần chạy {}...", run);
        
        // Tạo blinding factor
        let mut rng = OsRng;
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        let blinding = Scalar::from_bytes_mod_order(bytes[..32].try_into().unwrap());
        
        // Đo thời gian tạo proof
        let mut transcript = Transcript::new(match bitsize {
            8 => b"RangeProofBenchmark_8",
            16 => b"RangeProofBenchmark_16",
            32 => b"RangeProofBenchmark_32",
            64 => b"RangeProofBenchmark_64",
            _ => b"RangeProofBenchmark",
        });
        
        let gen_start = Instant::now();
        
        let (proof, committed_value) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            test_value,
            &blinding,
            bitsize,
        ).expect(&format!("Tạo proof thất bại cho {}-bit lần {}", bitsize, run));
        
        let gen_time = gen_start.elapsed();
        
        // Đo kích thước proof
        let proof_size = bincode::serialize(&proof).unwrap().len();
        
        // Đo thời gian xác minh
        let mut verifier_transcript = Transcript::new(match bitsize {
            8 => b"RangeProofBenchmark_8",
            16 => b"RangeProofBenchmark_16",
            32 => b"RangeProofBenchmark_32",
            64 => b"RangeProofBenchmark_64",
            _ => b"RangeProofBenchmark",
        });
        
        let ver_start = Instant::now();
        
        let verification_result = proof.verify_single(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &committed_value,
            bitsize,
        );
        
        let ver_time = ver_start.elapsed();
        
        // Kiểm tra kết quả
        let success = verification_result.is_ok();
        if !success {
            panic!("Xác minh thất bại cho {}-bit lần {}: {:?}", bitsize, run, verification_result.err());
        }
        
        // Tạo kết quả đo
        let result = MeasurementResult::new(
            bitsize,
            run,
            gen_time,
            ver_time,
            proof_size,
            success,
        );
        
        results.push(result);
        
        println!(" Gen: {:.2}ms, Ver: {:.2}ms, Size: {}B", 
                 gen_time.as_micros() as f64 / 1000.0,
                 ver_time.as_micros() as f64 / 1000.0,
                 proof_size);
    }
    
    println!("Hoàn thành đo {}-bit range proof\n", bitsize);
    results
}

/// Ghi dữ liệu vào file CSV đơn giản
fn save_measurements_to_csv(all_results: &[Vec<MeasurementResult>]) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("bulletproofs_measurements_{}.csv", timestamp);
    
    let mut file = File::create(&filename)?;
    
    // Ghi header đơn giản
    writeln!(file, "Bitsize,RunNumber,Status,GenerationTime_ms,VerificationTime_ms,TotalTime_ms,ProofSize_bytes,Result")?;
    
    // Ghi dữ liệu cho tất cả bitsizes
    for results in all_results {
        for result in results {
            file.write_all(result.to_csv_line().as_bytes())?;
        }
    }
    
    println!("Dữ liệu đã lưu vào: {}", filename);
    Ok(())
}

/// Ghi báo cáo tổng hợp đơn giản
fn save_summary_report(all_results: &[Vec<MeasurementResult>]) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("bulletproofs_summary_{}.txt", timestamp);
    
    let mut file = File::create(&filename)?;
    
    writeln!(file, "=== BÁO CÁO ĐO BULLETPROOFS RANGE PROOF ===")?;
    writeln!(file, "Thời gian tạo: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file)?;
    
    // Báo cáo cho từng bitsize
    for results in all_results {
        if let Some(first_result) = results.first() {
            let bitsize = first_result.bitsize;
            
            writeln!(file, "--- {}-bit Range Proof ---", bitsize)?;
            writeln!(file, "Số lần chạy: {}", results.len())?;
            
            // Tính thống kê
            let gen_times: Vec<f64> = results.iter().map(|r| r.generation_time_ms).collect();
            let ver_times: Vec<f64> = results.iter().map(|r| r.verification_time_ms).collect();
            let total_times: Vec<f64> = results.iter().map(|r| r.total_time_ms).collect();
            
            let avg_gen = gen_times.iter().sum::<f64>() / gen_times.len() as f64;
            let avg_ver = ver_times.iter().sum::<f64>() / ver_times.len() as f64;
            let avg_total = total_times.iter().sum::<f64>() / total_times.len() as f64;
            
            let min_gen = gen_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_gen = gen_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let proof_size = results[0].proof_size_bytes;
            
            writeln!(file, "Thời gian tạo trung bình: {:.2} ms", avg_gen)?;
            writeln!(file, "Thời gian xác minh trung bình: {:.2} ms", avg_ver)?;
            writeln!(file, "Tổng thời gian trung bình: {:.2} ms", avg_total)?;
            writeln!(file, "Kích thước proof: {} bytes", proof_size)?;
            writeln!(file, "Phạm vi thời gian tạo: {:.2} - {:.2} ms", min_gen, max_gen)?;
            
            // Chi tiết từng lần chạy
            writeln!(file, "Chi tiết từng lần chạy:")?;
            for result in results {
                writeln!(file, "  Lần {}: Gen={:.2}ms, Ver={:.2}ms, Total={:.2}ms", 
                         result.run_number, 
                         result.generation_time_ms,
                         result.verification_time_ms,
                         result.total_time_ms)?;
            }
            writeln!(file)?;
        }
    }
    
    println!(" Báo cáo tổng hợp đã lưu vào: {}", filename);
    Ok(())
}

fn main() {
    println!("=== ĐO VÀ GHI DỮ LIỆU BULLETPROOFS RANGE PROOF ===\n");
    
    // Cấu hình đo
    let bitsizes = [8, 16, 32, 64];
    let test_values = [255u64, 65535u64, 4294967295u64, 18446744073709551615u64];
    let runs_per_bitsize = 10;
    
    println!("Cấu hình đo:");
    println!("- Bit ranges: {:?}", bitsizes);
    println!("- Số lần chạy mỗi bitsize: {}", runs_per_bitsize);
    println!("- Tổng số lần đo: {}\n", bitsizes.len() * runs_per_bitsize);
    
    // Thực hiện đo cho từng bitsize
    let mut all_results = Vec::new();
    
    for (i, &bitsize) in bitsizes.iter().enumerate() {
        let test_value = test_values[i];
        let results = measure_bitsize(bitsize, test_value, runs_per_bitsize);
        all_results.push(results);
    }
    
    // Lưu dữ liệu vào file
    println!("=== LƯU DỮ LIỆU ===");
    
    if let Err(e) = save_measurements_to_csv(&all_results) {
        eprintln!("Lỗi khi lưu file CSV: {}", e);
    }
    
    if let Err(e) = save_summary_report(&all_results) {
        eprintln!(" Lỗi khi lưu báo cáo: {}", e);
    }
    
    // In tổng kết
    println!("\n=== TỔNG KẾT ===");
    let total_runs: usize = all_results.iter().map(|r| r.len()).sum();
    println!(" Tổng số lần đo: {}", total_runs);
    println!(" Tất cả các lần đo đều thành công");
    println!(" Dữ liệu đã được lưu vào file CSV và báo cáo");
    
    println!("\n=== THỐNG KÊ NHANH ===");
    for results in &all_results {
        if let Some(first_result) = results.first() {
            let bitsize = first_result.bitsize;
            let avg_gen: f64 = results.iter().map(|r| r.generation_time_ms).sum::<f64>() / results.len() as f64;
            let avg_ver: f64 = results.iter().map(|r| r.verification_time_ms).sum::<f64>() / results.len() as f64;
            let proof_size = first_result.proof_size_bytes;
            
            println!("{}-bit: Gen={:.2}ms, Ver={:.2}ms, Size={}B", 
                     bitsize, avg_gen, avg_ver, proof_size);
        }
    }
    
    println!("\n Hoàn thành đo và ghi dữ liệu!");
}