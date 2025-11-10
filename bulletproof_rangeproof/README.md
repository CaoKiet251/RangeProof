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
cargo run --release -- --range 32 --output bulletproofs_measurements.csv
```

Điều chỉnh tham số:
- `--range <bits>`: bit-range (ví dụ 8, 16, 32, 64)
- `--repeat <n>` (nếu binary hỗ trợ): số lần lặp để lấy thống kê
- `--output <file>`: nơi ghi CSV kết quả (thời gian prove/verify, kích thước proof)

