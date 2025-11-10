use std::env;
use num_bigint::BigInt;
use cuproof::setup::{trusted_setup, fast_test_setup};
use cuproof::range_proof::{cuproof_prove};
use cuproof::verify::{cuproof_verify, cuproof_verify_with_range};
use cuproof::util::{save_params, load_params, save_proof, load_proof, hex_to_bigint};
use cuproof::benchmark::{benchmark_multiple_ranges, print_benchmark_summary};

/// CLI entry: supports commands
/// - setup [fast|trusted] <params_path>
/// - prove <params_path> <a> <b> <v> <proof_path>
/// - verify <params_path> <proof_path>
/// - benchmark [fast|trusted] [range_lengths...]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:\n  setup [fast|trusted] <params_path>\n  prove <params_path> <a_hex> <b_hex> <v_hex> <proof_path>\n  verify <params_path> <a_hex> <b_hex> <proof_path>\n  benchmark [fast|trusted] [range_lengths...]");
        return;
    }
    match args[1].as_str() {
        "setup" => {
            if args.len() < 4 { eprintln!("Usage: setup [fast|trusted] <params_path>"); return; }
            let mode = args[2].as_str();
            let path = &args[3];
            let (g, h, n) = match mode {
                "fast" => fast_test_setup(),
                "trusted" => trusted_setup(2048),
                _ => { eprintln!("mode must be fast or trusted"); return; }
            };
            if let Err(e) = save_params(path, &g, &h, &n) {
                eprintln!("Failed to save params: {}", e);
                return;
            }
            println!("Saved public parameters to {}", path);
        }
        "prove" => {
            if args.len() < 7 { eprintln!("Usage: prove <params_path> <a_hex> <b_hex> <v_hex> <proof_path>"); return; }
            let params_path = &args[2];
            let a = hex_to_bigint(&args[3]);
            let b = hex_to_bigint(&args[4]);
            let v = hex_to_bigint(&args[5]);
            let proof_path = &args[6];
            let (g, h, n) = match load_params(params_path) {
                Ok(t) => t,
                Err(e) => { eprintln!("Failed to load params: {}", e); return; }
            };
            // NOTE: In practice, r must be random and kept secret by prover
            let r = cuproof::util::random_bigint(256);
            let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
            if let Err(e) = save_proof(proof_path, &proof) {
                eprintln!("Failed to save proof: {}", e);
                return;
            }
            println!("Saved proof to {}", proof_path);
        }
        "verify" => {
            if args.len() < 6 { eprintln!("Usage: verify <params_path> <a_hex> <b_hex> <proof_path>"); return; }
            let params_path = &args[2];
            let a = hex_to_bigint(&args[3]);
            let b = hex_to_bigint(&args[4]);
            let proof_path = &args[5];
            let (g, h, n) = match load_params(params_path) {
                Ok(t) => t,
                Err(e) => { eprintln!("Failed to load params: {}", e); return; }
            };
            let proof = match load_proof(proof_path) {
                Ok(p) => p,
                Err(e) => { eprintln!("Failed to load proof: {}", e); return; }
            };
            let ok = cuproof_verify_with_range(&proof, &g, &h, &n, &a, &b);
            println!("{}", if ok { "VALID" } else { "INVALID" });
        }
        "benchmark" => {
            if args.len() < 3 { 
                eprintln!("Usage: benchmark [fast|trusted] [range_lengths...]");
                eprintln!("Example: benchmark fast 8 16 32 64");
                eprintln!("Example: benchmark trusted 8 16 32 64 128 256 512 1024");
                return; 
            }
            
            let mode = args[2].as_str();
            let use_fast_setup = match mode {
                "fast" => true,
                "trusted" => false,
                _ => { 
                    eprintln!("Mode must be 'fast' or 'trusted'"); 
                    return; 
                }
            };
            
            // Parse range lengths from command line arguments
            let mut range_lengths = Vec::new();
            if args.len() > 3 {
                for i in 3..args.len() {
                    match args[i].parse::<usize>() {
                        Ok(length) => range_lengths.push(length),
                        Err(_) => {
                            eprintln!("Invalid range length: {}", args[i]);
                            return;
                        }
                    }
                }
            } else {
                // Default range lengths if none specified
                range_lengths = vec![8, 16, 32, 64, 128, 256, 512, 1024];
            }
            
            println!("Bắt đầu benchmark Cuproof với {} độ dài khoảng", range_lengths.len());
            println!("Chế độ setup: {}", if use_fast_setup { "fast" } else { "trusted" });
            println!("Các độ dài khoảng: {:?}", range_lengths);
            println!();
            
            let results = benchmark_multiple_ranges(range_lengths, use_fast_setup);
            print_benchmark_summary(&results);
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
}
