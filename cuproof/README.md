# CuProof (Off-chain & On-chain)

Thư mục này chứa triển khai CuProof để sinh proof off-chain và xác minh on-chain (blockchain).

## Cấu trúc gợi ý
- `src/`: code sinh/kiểm proof (Rust hoặc ngôn ngữ bạn chọn)
- `scripts/`: tiện ích chạy benchmark, deploy, verify on-chain
- `contracts/`: smart contract verifier (Solidity/Move/khác)
- `results/`: log, CSV, JSON kết quả (tùy bạn lưu)

## Chuẩn bị
- Cài Rust (hoặc ngôn ngữ dùng cho off-chain)
- Cài Node.js/Docker nếu pipeline on-chain cần
- Khởi động blockchain cục bộ (ví dụ Anvil/Ganache):
  ```bash
  docker compose up -d   # nếu bạn có docker-compose.yml
  ```

## Chạy benchmark off-chain
```bash
./scripts/run_offchain.sh --range 32 --output results/cuproof_offchain_32.csv
```
Hoặc PowerShell:
```powershell
.\scripts\run_offchain.ps1 -Range 32 -Output results\cuproof_offchain_32.csv
```
Các tham số phổ biến:
- `--range/-Range`: bit-range
- `--repeat`: số lần lặp để thống kê
- `--output`: file lưu kết quả (thời gian prove/verify, size, trạng thái)

## Chạy verify on-chain
```bash
./scripts/run_onchain.sh --range 32 --rpc http://127.0.0.1:8545 --output results/cuproof_onchain_32.csv
```
Đảm bảo:
- Contract verifier đã deploy (script deploy tùy repo)
- RPC endpoint trỏ tới node đang chạy

## Tổng hợp
Sau khi có CSV cho nhiều range-bit, đưa vào pipeline chung (ví dụ `benchmark/`) để tạo `combined.csv` và biểu đồ so sánh với Bulletproof, Circom/zkSNARK.

