# Circom Range Proofs (zkSNARK)

Thư mục này chứa circuit, key và script phục vụ tạo/kiểm chứng range proof bằng Circom + SnarkJS (hoặc hệ tương đương).

## Cấu trúc chính
- `circuits/`: các file `.circom` theo từng bit-range
- `build/`: artifact build (r1cs, wasm, witness calculator) cho từng circuit
- `keys/`: tệp Powers of Tau (`.ptau`), proving/verification key (`.zkey`, `verification_key*.json`)
- `inputs/`: dữ liệu đầu vào mẫu (`input_*.json`)
- `proofs/`: proof (`proof_*.json`), public signals (`public_*.json`), witness (`witness_*.wtns`)
- `scripts/`: script benchmark, sinh proof, phân tích kết quả
- `results/`: báo cáo, CSV thống kê

## Chuẩn bị
```bash
npm install   # nếu project dùng package.json
```
Nếu chưa có artifact, chạy script build (ví dụ):
```bash
node scripts/generate_proofs.js --range 32
```
(Cập nhật lệnh đúng với workflow của bạn)

## Chạy benchmark
Ví dụ chạy đo thời gian và ghi kết quả:
```bash
node scripts/run_benchmark.js --ranges 8,16,32,64 --output results/range_proof_benchmark_results.csv
```

Trong đó:
- `--ranges`: danh sách bit-range (phân tách dấu phẩy)
- `--repeat` (nếu script hỗ trợ): số lần lặp
- `--output`: file CSV hoặc JSON để lưu kết quả prove/verify

## Tổng hợp
Dùng kết quả trong `results/` để so sánh với Bulletproof và CuProof. Có thể kết hợp với thư mục `benchmark/` hoặc script của bạn để tạo `combined.csv` và vẽ biểu đồ.

