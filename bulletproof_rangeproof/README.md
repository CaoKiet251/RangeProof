# Bulletproof Range Proofs

Thư mục này chứa triển khai Bulletproof bằng Rust. Mục tiêu: sinh proof phạm vi cho các bit-range khác nhau, đo thời gian và kích thước proof.

## Chuẩn bị
- Cài Rust toolchain: `https://rustup.rs`
- Build release:
  ```bash
  cargo build --release
  ```

## Chạy benchmark
Ví dụ chạy với range 32-bit và ghi log:
```bash
cargo run --release -- --range 32 --output bulletproofs_measurements_$(date +%Y%m%d_%H%M%S).csv
```

Điều chỉnh tham số:
- `--range <bits>`: bit-range (ví dụ 8, 16, 32, 64, 128…)
- `--repeat <n>` (nếu binary hỗ trợ): số lần lặp để lấy thống kê
- `--output <file>`: nơi ghi CSV kết quả (thời gian prove/verify, kích thước proof)

## Tổng kết
Sau khi chạy cho các range-bits khác nhau, tổng hợp CSV lại để so sánh với các hệ khác (zkSNARK, CuProof). Dùng `benchmark/` (nếu có) hoặc script cá nhân tùy nhu cầu.

