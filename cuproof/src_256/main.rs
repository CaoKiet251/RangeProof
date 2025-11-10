use std::env;
use num_bigint::BigInt;

mod setup;
mod commitment;
mod fiat_shamir;
mod lagrange;
mod range_proof;
mod verify;
mod util;
mod benchmark;
mod evm;

use setup::{setup_256, fast_test_setup};
use range_proof::cuproof_prove;
use verify::cuproof_verify_with_range;
use util::{save_params, load_params, save_proof, load_proof, hex_to_bigint, random_bigint};
use benchmark::{benchmark_multiple_ranges, print_benchmark_summary};
use evm::{save_proof_for_evm, save_proof_json};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:\n  setup [256|fast] <params_path>\n  prove <params_path> <a_hex> <b_hex> <v_hex> <proof_path>\n  verify <params_path> <a_hex> <b_hex> <proof_path>\n  benchmark [256|fast] [range_lengths...]");
        return;
    }
    match args[1].as_str() {
        "setup" => {
            if args.len() < 4 { eprintln!("Usage: setup [256|fast] <params_path>"); return; }
            let mode = args[2].as_str();
            let path = &args[3];
            let (g, h, n) = match mode {
                "256" => setup_256(),
                "fast" => fast_test_setup(),
                _ => { eprintln!("mode must be 256 or fast"); return; }
            };
            if let Err(e) = save_params(path, &g, &h, &n) {
                eprintln!("Failed to save params: {}", e);
                return;
            }
            println!("Saved public parameters to {}", path);
        }
        "prove" => {
            if args.len() < 7 { eprintln!("Usage: prove <params_path> <a_hex> <b_hex> <v_hex> <proof_path> [--evm] [--json]"); return; }
            let params_path = &args[2];
            let a = hex_to_bigint(&args[3]);
            let b = hex_to_bigint(&args[4]);
            let v = hex_to_bigint(&args[5]);
            let proof_path = &args[6];
            let export_evm = args.contains(&"--evm".to_string());
            let export_json = args.contains(&"--json".to_string());
            
            let (g, h, n) = match load_params(params_path) {
                Ok(t) => t,
                Err(e) => { eprintln!("Failed to load params: {}", e); return; }
            };
            let r = random_bigint(256);
            let proof = cuproof_prove(&v, &r, &a, &b, &g, &h, &n);
            
            if let Err(e) = save_proof(proof_path, &proof) {
                eprintln!("Failed to save proof: {}", e);
                return;
            }
            println!("Saved proof to {}", proof_path);
            
            if export_evm {
                let evm_path = format!("{}_evm.sol", proof_path.trim_end_matches(".txt"));
                if let Err(e) = save_proof_for_evm(&evm_path, &proof, &g, &h, &n) {
                    eprintln!("Failed to save EVM format: {}", e);
                } else {
                    println!("Saved EVM-compatible proof to {}", evm_path);
                }
            }
            
            if export_json {
                let json_path = format!("{}_evm.json", proof_path.trim_end_matches(".txt"));
                if let Err(e) = save_proof_json(&json_path, &proof, &g, &h, &n) {
                    eprintln!("Failed to save JSON format: {}", e);
                } else {
                    println!("Saved JSON proof to {}", json_path);
                }
            }
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
                eprintln!("Usage: benchmark [256|fast] [range_lengths...]");
                eprintln!("Example: benchmark 256 8 16 32 64");
                eprintln!("Example: benchmark fast 8 16 32 64");
                return; 
            }
            
            let mode = args[2].as_str();
            let use_256_setup = match mode {
                "256" => true,
                "fast" => false,
                _ => { 
                    eprintln!("Mode must be '256' or 'fast'"); 
                    return; 
                }
            };
            
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
                range_lengths = vec![8, 16, 32, 64, 128, 256, 512, 1024];
            }
            
            println!("Bắt đầu benchmark Cuproof với {} độ dài khoảng", range_lengths.len());
            println!("Chế độ setup: {}", if use_256_setup { "256-bit" } else { "fast" });
            println!("Các độ dài khoảng: {:?}", range_lengths);
            println!();
            
            let results = benchmark_multiple_ranges(range_lengths, use_256_setup);
            print_benchmark_summary(&results);
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
}

