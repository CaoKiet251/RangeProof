# Cuproof Blockchain Integration

Hệ thống tích hợp blockchain cho Cuproof Range Proof System.

## Tổng quan

Dự án này cung cấp smart contracts và tools để tích hợp hệ thống Cuproof với blockchain Ethereum, cho phép:

- **Native On-chain Verification**: Thực hiện verification hoàn toàn trên blockchain, không cần off-chain service
- **Lưu trữ và quản lý verification results**: Proof hashes và verification status được lưu trữ bất biến trên blockchain
- **Quản lý public parameters**: Public parameters (g, h, n) được quản lý và cập nhật bởi contract owner
- **Transparency và Audit Trail**: Tất cả verification operations được ghi lại qua events, có thể audit công khai

### Điểm nổi bật

✅ **Fully On-chain**: Toàn bộ quá trình verification diễn ra trên smart contract, không cần trust off-chain service  
✅ **Cryptographically Secure**: Sử dụng Fiat-Shamir heuristic và Pedersen commitments  
✅ **Gas Efficient**: Optimized cho EVM với Solidity optimizer  
✅ **Easy Integration**: Simple interface với `verifyProofSimple()` function  
✅ **Immutable Records**: Proof verification results được lưu trữ vĩnh viễn trên blockchain

## Kiến trúc

### Smart Contracts

1. **CuproofVerifier256**: Thực hiện verification on-chain cho Cuproof range proofs với 256-bit modulus
   - Verification hoàn toàn trên blockchain (không cần off-chain service)
   - Quản lý public parameters (g, h, n)
   - Lưu trữ kết quả verification và proof hashes

### Quy trình Chứng minh On-chain

Hệ thống thực hiện **native on-chain verification**, nghĩa là toàn bộ quá trình verify được thực hiện trực tiếp trên smart contract mà không cần off-chain service.

#### 1. Tạo Proof (Off-chain - Rust)

```bash
# Tạo proof và export sang JSON format cho blockchain
cargo run --release prove params.txt <a_hex> <b_hex> <v_hex> proof.txt --json
```

Quá trình này sẽ:
- Tạo Cuproof range proof với giá trị `v` trong khoảng `[a, b]`
- Export proof sang file `proof_evm.json` với format tương thích EVM
- File JSON chứa tất cả các thành phần của proof: scalars, IPP vectors (L, R), và IPP scalars (a, b)

#### 2. Deploy Smart Contract

```bash
# Deploy contract với public parameters
npx hardhat run scripts/deploy-256.js --network localhost
```

Contract `CuproofVerifier256` được deploy với:
- **g, h, n**: Public parameters (generators và modulus)
- Các parameters này phải khớp với parameters dùng để tạo proof

#### 3. Verify Proof On-chain

```bash
# Verify proof trên blockchain
npx hardhat run scripts/verify-256.js --network localhost
```

Script này sẽ:
- Load proof từ file `proof_evm.json`
- Load parameters từ `params.txt`
- Gọi function `verifyProofSimple()` trên smart contract
- Contract thực hiện verification hoàn toàn on-chain

#### 4. Quá trình Verification On-chain

Smart contract thực hiện các bước sau để verify proof:

**Bước 1: Fiat-Shamir Challenge Computation**

Contract sử dụng Fiat-Shamir heuristic để tạo challenges một cách non-interactive. Implementation khớp với Rust code:

```solidity
// Challenge y: hash của initial commitments
y = keccak256(A, S, C, C_v1, C_v2) % n

// Challenge z: hash của y
z = keccak256(y) % n

// Challenge x: hash của T1 và T2
x = keccak256(T1, T2) % n
```

**Lưu ý quan trọng:**
- Hash function sử dụng `keccak256` (EVM native)
- Mỗi uint256 được encode thành 32 bytes (big-endian) trước khi hash
- Format này khớp với Rust implementation sử dụng `sha3::Keccak256`
- Kết quả hash được modulo n để đảm bảo trong phạm vi [0, n-1]

**Bước 2: Commitment Verification**

Contract verify các Pedersen commitments sử dụng công thức:
```
PedersenCommit(m, r) = g^m * h^r (mod n)
```

Cụ thể:
- **T1 verification**: `pedersenCommit(t1, tau1) == T1`
  - Tính `g^t1 * h^tau1 (mod n)` và so sánh với T1
- **T2 verification**: `pedersenCommit(t2, tau2) == T2`
  - Tính `g^t2 * h^tau2 (mod n)` và so sánh với T2
- **t_hat verification**: 
  - Tính `rhs_t = t0 + t1*x + t2*x² (mod n)`
  - Verify `pedersenCommit(t_hat, tau_x) == pedersenCommit(rhs_t, tau_x)`
  - Đảm bảo t_hat khớp với polynomial evaluation

**Implementation:**
```solidity
function pedersenCommit(uint256 m, uint256 r) internal view returns (uint256) {
    uint256 g_m = modExp(params.g, m, params.n);
    uint256 h_r = modExp(params.h, r, params.n);
    return mulmod(g_m, h_r, params.n);
}
```

**Bước 3: Polynomial Verification**
- Verify polynomial relation: `t_hat == t0 + t1*x + t2*x² (mod n)`

**Bước 4: Sanity Checks**
- Kiểm tra các giá trị không bằng 0 mod n
- Kiểm tra C, C_v1, C_v2 là các giá trị khác nhau
- Kiểm tra IPP vectors có đúng độ dài (6 levels)

**Bước 5: Proof Hash Calculation**

Proof hash được tính từ toàn bộ proof structure để đảm bảo uniqueness:
```solidity
bytes32 proofHash = keccak256(abi.encode(proof));
```

Hash này được sử dụng để:
- Ngăn chặn verify lại cùng một proof (duplicate prevention)
- Tracking latest proof cho mỗi subject
- Index trong `verifiedProofs` mapping

**Lưu ý:** Proof hash được tính từ Proof struct, bao gồm tất cả các fields: scalars, IPP vectors, và IPP scalars.

**Bước 6: Lưu trữ Kết quả**
- Lưu proof hash vào mapping `verifiedProofs[proofHash] = true`
- Cập nhật `latestProofHash[subject] = proofHash`
- Emit event `ProofVerified` với thông tin chi tiết:
  - Subject address
  - Proof hash
  - Range [min, max]
  - Verification status (luôn là `true` nếu đến bước này)
  - Block timestamp

#### 5. Kiểm tra Trạng thái Proof

Sau khi verify, có thể kiểm tra:
```javascript
// Kiểm tra proof đã được verify chưa
const isVerified = await verifier.isProofVerified(proofHash);

// Lấy proof hash mới nhất của một subject
const latestHash = await verifier.getLatestProofHash(subjectAddress);
```

### Workflow Diagram

```
┌─────────────────┐
│  Rust Prover    │
│  (Off-chain)    │
└────────┬────────┘
         │
         │ 1. Tạo proof với cuproof_prove()
         │ 2. Export sang proof_evm.json
         ▼
┌─────────────────┐
│  proof_evm.json  │
│  params.txt     │
└────────┬────────┘
         │
         │ 3. Load proof & params
         ▼
┌─────────────────┐
│  verify-256.js   │
│  (Script)        │
└────────┬────────┘
         │
         │ 4. Gọi verifyProofSimple()
         ▼
┌─────────────────────────────────┐
│  CuproofVerifier256 Contract    │
│  (On-chain Verification)        │
│                                  │
│  • Fiat-Shamir challenges       │
│  • Commitment verification      │
│  • Polynomial verification      │
│  • IPP verification             │
│  • Sanity checks                │
│  • Store proof hash             │
└────────┬────────────────────────┘
         │
         │ 5. Emit ProofVerified event
         │ 6. Update verifiedProofs mapping
         ▼
┌─────────────────┐
│  Blockchain      │
│  (Immutable)     │
└─────────────────┘
```

## Cài đặt và Chạy

### Yêu cầu
- Node.js 16+
- npm hoặc yarn
- Hardhat

### Cài đặt dependencies
```bash
npm install
```

### Compile contracts
```bash
npm run compile
```

### Chạy tests
```bash
npm test
```

### Deploy contracts

#### Local development
```bash
# Terminal 1: Start local blockchain
npm run node

# Terminal 2: Deploy contracts
npm run deploy:local
```

#### Hardhat network
```bash
npm run deploy:hardhat
```

## Smart Contracts

### CuproofVerifier256

Contract chính thực hiện **native on-chain verification** cho Cuproof range proofs.

**Cấu trúc Proof:**
```solidity
struct Proof {
    uint256 A, S, T1, T2;           // Initial commitments
    uint256 tau_x, mu, t_hat;        // Challenge responses
    uint256 C, C_v1, C_v2;           // Value commitments
    uint256 t0, t1, t2;              // Polynomial coefficients
    uint256 tau1, tau2;              // Blinding factors
    uint256[] ipp_L, ipp_R;          // Inner Product Proof vectors
    uint256 ipp_a, ipp_b;            // Inner Product Proof scalars
}
```

**Chức năng chính:**

1. **On-chain Verification**
   - Thực hiện toàn bộ quá trình verify trực tiếp trên EVM
   - Không cần off-chain service hay trusted verifier
   - Sử dụng Fiat-Shamir heuristic để tạo challenges
   - Verify Pedersen commitments và polynomial relations

2. **Public Parameters Management**
   - Lưu trữ và quản lý g, h, n (generators và modulus)
   - Chỉ owner có thể cập nhật parameters

3. **Proof Storage**
   - Lưu trữ proof hashes đã verify
   - Tracking latest proof cho mỗi subject address
   - Prevent duplicate verification

**Key Functions:**

- `verifyProof(Proof memory, uint256 rangeMin, uint256 rangeMax, address subject)`
  - Function chính để verify proof
  - Thực hiện đầy đủ các bước verification
  - Trả về `true` nếu proof hợp lệ
  - Emit event `ProofVerified` khi thành công

- `verifyProofSimple(uint256[15] scalars, uint256[] ipp_L, uint256[] ipp_R, uint256 ipp_a, uint256 ipp_b, uint256 rangeMin, uint256 rangeMax, address subject)`
  - Interface đơn giản hóa để gọi từ JavaScript
  - Chuyển đổi parameters thành Proof struct và gọi `verifyProof()`

- `isProofVerified(bytes32 proofHash)`
  - Kiểm tra xem một proof đã được verify chưa
  - Sử dụng proof hash làm identifier

- `getLatestProofHash(address subject)`
  - Lấy proof hash mới nhất của một subject
  - Trả về `0x0` nếu chưa có proof nào

- `updateParams(uint256 _g, uint256 _h, uint256 _n)`
  - Cập nhật public parameters (chỉ owner)
  - Phải đảm bảo parameters hợp lệ (khác 0, nhỏ hơn n)

**Verification Algorithm:**

Contract thực hiện verification theo các bước sau:

1. **Input Validation**
   - Kiểm tra range hợp lệ (rangeMin ≤ rangeMax)
   - Kiểm tra subject address không phải zero address
   - Kiểm tra proof chưa được verify trước đó

2. **Fiat-Shamir Challenges**
   ```solidity
   y = keccak256(A, S, C, C_v1, C_v2) % n
   z = keccak256(y) % n
   x = keccak256(T1, T2) % n
   ```

3. **Commitment Verification**
   - Verify T1 = PedersenCommit(t1, tau1)
   - Verify T2 = PedersenCommit(t2, tau2)
   - Verify t_hat commitment consistency

4. **Polynomial Verification**
   - Verify t_hat = t0 + t1*x + t2*x² (mod n)

5. **Sanity Checks**
   - Tất cả commitments khác 0 mod n
   - C, C_v1, C_v2 là các giá trị khác nhau
   - IPP vectors có đúng độ dài (6 levels)

6. **Storage & Events**
   - Lưu proof hash vào `verifiedProofs` mapping
   - Cập nhật `latestProofHash[subject]`
   - Emit `ProofVerified` event

**Events:**

```solidity
event ProofVerified(
    address indexed subject,
    bytes32 indexed proofHash,
    uint256 rangeMin,
    uint256 rangeMax,
    bool isValid,
    uint256 timestamp
);
```

**Security Features:**

- **Duplicate Prevention**: Mỗi proof chỉ có thể verify một lần
- **Access Control**: Chỉ owner có thể update parameters
- **Input Validation**: Tất cả inputs được validate kỹ lưỡng
- **Modular Arithmetic**: Sử dụng safe math operations với modulus n

## Testing

### Chạy tất cả tests
```bash
npm test
```

### Chạy tests với verbose output
```bash
npm run test:verbose
```

### Test coverage
```bash
npm run coverage
```

## Scripts

### Available Scripts

- `npm run compile`: Compile smart contracts
- `npm run test`: Chạy tests
- `npm run deploy:local`: Deploy lên localhost network
- `npm run deploy:hardhat`: Deploy lên hardhat network
- `npm run node`: Start local blockchain node
- `npm run clean`: Clean build artifacts
- `npm run console`: Hardhat console

### Scripts Chi tiết

#### `scripts/deploy-256.js`

Script để deploy contract `CuproofVerifier256` lên blockchain.

**Input:**
- File `params.txt` ở root directory chứa 3 dòng: g, h, n (hex format)

**Quá trình:**
1. Load parameters từ `params.txt`
2. Deploy contract với parameters đó
3. Lưu deployment info vào `deployment-info.json`

**Output:**
- Contract address
- Deployment info JSON file

**Ví dụ:**
```bash
# Deploy lên localhost
npx hardhat run scripts/deploy-256.js --network localhost

# Deploy lên hardhat network
npx hardhat run scripts/deploy-256.js --network hardhat
```

#### `scripts/verify-256.js`

Script để verify proof on-chain.

**Input:**
- File `proof_evm.json` ở root directory (export từ Rust)
- File `params.txt` ở root directory
- Contract address từ `deployment-info.json` (hoặc deploy mới nếu chưa có)

**Quá trình:**
1. Load proof từ `proof_evm.json`
2. Load parameters từ `params.txt`
3. Attach hoặc deploy contract `CuproofVerifier256`
4. Gọi `verifyProofSimple()` với proof data
5. Kiểm tra kết quả verification
6. Hiển thị proof hash và verification status

**Proof Format (proof_evm.json):**
```json
{
  "scalars": [
    "0x...",  // A
    "0x...",  // S
    "0x...",  // T1
    "0x...",  // T2
    "0x...",  // tau_x
    "0x...",  // mu
    "0x...",  // t_hat
    "0x...",  // C
    "0x...",  // C_v1
    "0x...",  // C_v2
    "0x...",  // t0
    "0x...",  // t1
    "0x...",  // t2
    "0x...",  // tau1
    "0x..."   // tau2
  ],
  "ipp_L": ["0x...", "0x...", ...],  // 6 elements
  "ipp_R": ["0x...", "0x...", ...],  // 6 elements
  "ipp_a": "0x...",
  "ipp_b": "0x..."
}
```

**Ví dụ:**
```bash
# Verify proof trên localhost
npx hardhat run scripts/verify-256.js --network localhost
```

**Output:**
- Transaction hash
- Block number
- Proof hash (calculated và từ contract)
- Verification status

### Workflow Example Hoàn chỉnh

Dưới đây là ví dụ đầy đủ từ tạo proof đến verify on-chain:

```bash
# Bước 1: Setup public parameters (nếu chưa có)
cd ..
cargo run --release setup 256 params.txt

# Bước 2: Tạo proof và export sang JSON
cargo run --release prove params.txt 0x0 0x64 0x32 proof.txt --json
# Giá trị 0x32 (50) nằm trong khoảng [0, 100]
# File proof_evm.json sẽ được tạo ra

# Bước 3: Start local blockchain (terminal riêng)
cd cuproof-blockchain
npm run node

# Bước 4: Deploy contract (terminal mới)
npx hardhat run scripts/deploy-256.js --network localhost

# Bước 5: Verify proof on-chain
npx hardhat run scripts/verify-256.js --network localhost
```

**Kết quả mong đợi:**
```
Verifying proof on-chain...
Subject: 0x...
Range: [0, 100]
Proof scalars: 15
IPP L length: 6
IPP R length: 6
Transaction sent: 0x...
Transaction confirmed in block: 12345
Calculated proof hash: 0x...
Latest proof hash from contract: 0x...
Hash match: true
Proof verified status: true
SUCCESS: Proof has been verified and stored on-chain!
```

## Configuration

### Networks

Mặc định hỗ trợ:
- `hardhat`: Local hardhat network
- `localhost`: Local blockchain node

Để thêm networks khác, cập nhật `hardhat.config.js`:

```javascript
networks: {
  mainnet: {
    url: process.env.MAINNET_RPC_URL,
    accounts: [process.env.PRIVATE_KEY],
    gasPrice: 20000000000, // 20 gwei
  },
  goerli: {
    url: process.env.GOERLI_RPC_URL,
    accounts: [process.env.PRIVATE_KEY],
  }
}
```

## Deployment Info

Sau khi deploy, thông tin sẽ được lưu trong `deployment-info.json`:

```json
{
  "network": "localhost",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "contracts": {
    "CuproofRegistry": "0x...",
    "CuproofVerifier": "0x..."
  },
  "deployer": "0x...",
  "paramsHash": "0x..."
}
```

## Security

### Access Control
- Chỉ owner mới có thể cập nhật public parameters (g, h, n)
- Bất kỳ ai cũng có thể verify proof (public verification)
- Proof verification không yêu cầu authorization

### Duplicate Prevention
- Mỗi proof chỉ có thể verify một lần
- Proof hash được tính từ toàn bộ proof structure
- Mapping `verifiedProofs` ngăn chặn verify lại cùng một proof

### Input Validation
- Tất cả inputs được validate kỹ lưỡng:
  - Range phải hợp lệ (rangeMin ≤ rangeMax)
  - Subject address không được là zero address
  - Public parameters phải hợp lệ (khác 0, nhỏ hơn n)
  - Proof components phải thỏa mãn các điều kiện sanity checks

### Cryptographic Security
- Sử dụng Fiat-Shamir heuristic để tạo challenges (non-interactive)
- Pedersen commitments đảm bảo hiding và binding properties
- Modular arithmetic operations an toàn với Solidity
- Proof hash sử dụng keccak256 (EVM native)

### Gas Considerations
- Verification on-chain tiêu tốn gas đáng kể do:
  - Modular exponentiation operations
  - Multiple commitment verifications
  - Array operations cho IPP vectors
- Nên test trên testnet trước khi deploy lên mainnet
- Có thể optimize bằng cách sử dụng precompiled contracts (nếu có)

## Gas Optimization

- Solidity optimizer enabled với 200 runs
- Efficient storage patterns
- Event-based logging thay vì storage

## Lưu ý Quan trọng

### Compatibility với Rust Implementation

Contract được thiết kế để tương thích hoàn toàn với Rust implementation:

1. **Fiat-Shamir Hash**: Cả hai đều sử dụng keccak256, với cùng cách encode uint256 thành bytes
2. **Modular Arithmetic**: Cả hai đều sử dụng modular arithmetic với cùng modulus n
3. **Proof Format**: Proof structure khớp giữa Rust và Solidity
4. **Pedersen Commitments**: Công thức `g^m * h^r (mod n)` giống nhau

### IPP (Inner Product Proof) Verification

Contract hiện tại thực hiện:
- ✅ Sanity checks cho IPP vectors (độ dài, không rỗng)
- ✅ Lưu trữ IPP components (L, R, a, b)
- ⚠️ Full IPP verification logic có thể cần được implement thêm tùy theo yêu cầu

### Proof Export từ Rust

Khi export proof từ Rust với flag `--json`:
- File `proof_evm.json` được tạo ra với format tương thích
- Các giá trị T1, T2 được recalculate từ modulo'd values để đảm bảo consistency
- Tất cả values được convert sang hex format với prefix `0x`

### Network Requirements

- **Local Development**: Sử dụng `localhost` hoặc `hardhat` network
- **Testnet**: Có thể deploy lên Goerli, Sepolia, etc. (cần cấu hình trong `hardhat.config.js`)
- **Mainnet**: Nên test kỹ trên testnet trước, gas costs có thể cao

### File Dependencies

Scripts yêu cầu các files sau ở root directory:
- `params.txt`: Chứa g, h, n (3 dòng hex format)
- `proof_evm.json`: Proof được export từ Rust (với flag `--json`)
- `deployment-info.json`: Tự động tạo sau khi deploy

### Error Handling

Contract sử dụng `require()` statements để validate:
- Nếu validation fail, transaction sẽ revert với error message
- Script `verify-256.js` sẽ catch và hiển thị error message
- Common errors:
  - `"Invalid range"`: rangeMin > rangeMax
  - `"Invalid subject"`: subject address là zero
  - `"Proof already verified"`: Proof đã được verify trước đó
  - `"Invalid challenge y/z/x"`: Challenge tính được bằng 0 (rất hiếm)

## Troubleshooting

### Common Issues

1. **Compilation errors**
   - Kiểm tra Solidity version (contract yêu cầu ^0.8.19)
   - Đảm bảo đã cài đặt dependencies: `npm install`

2. **Deployment failures**
   - Đảm bảo có đủ ETH/ETH cho gas fees
   - Kiểm tra network connection (localhost node đang chạy?)
   - Verify file `params.txt` tồn tại và có đúng format (3 dòng hex)

3. **Verification failures**
   - **"Proof already verified"**: Proof này đã được verify trước đó
   - **"Invalid range"**: rangeMin phải ≤ rangeMax
   - **"T1 commitment mismatch"**: Proof không hợp lệ hoặc parameters không khớp
   - **"Invalid challenge y/z/x"**: Challenge tính được = 0 (rất hiếm, có thể do hash collision)
   - Đảm bảo `proof_evm.json` được tạo từ Rust với cùng parameters
   - Đảm bảo contract được deploy với cùng parameters như khi tạo proof

4. **File not found errors**
   - `proof_evm.json`: Chạy Rust với flag `--json` để tạo file này
   - `params.txt`: Chạy `cargo run --release setup 256 params.txt` để tạo
   - `deployment-info.json`: Tự động tạo sau khi deploy, nếu thiếu thì deploy lại

5. **Network connection errors**
   - Localhost: Đảm bảo đã chạy `npm run node` hoặc `npx hardhat node`
   - Testnet/Mainnet: Kiểm tra RPC URL trong `hardhat.config.js`
   - Kiểm tra private key/account có đủ balance

### Debug Commands

```bash
# Hardhat console để debug và tương tác với contract
npm run console
# Trong console:
# const Verifier = await ethers.getContractFactory('CuproofVerifier256');
# const verifier = await Verifier.attach('0x...');
# await verifier.params();

# Clean và rebuild
npm run clean
npm run compile

# Xem deployment info
cat deployment-info.json

# Test verification với verbose output
npx hardhat run scripts/verify-256.js --network localhost
```

### Debug Tips

1. **Kiểm tra parameters khớp nhau:**
   ```bash
   # Xem params trong contract
   npx hardhat console --network localhost
   # > const v = await ethers.getContractAt("CuproofVerifier256", "0x...");
   # > await v.params();
   
   # So sánh với params.txt
   cat params.txt
   ```

2. **Kiểm tra proof hash:**
   - Proof hash được tính trong script `verify-256.js`
   - So sánh với hash từ contract để đảm bảo consistency

3. **Gas estimation:**
   ```bash
   # Estimate gas trước khi verify
   npx hardhat console --network localhost
   # > const v = await ethers.getContractAt("CuproofVerifier256", "0x...");
   # > await v.estimateGas.verifyProofSimple(...);
   ```

## Contributing

1. Fork repository
2. Tạo feature branch
3. Commit changes
4. Push và tạo Pull Request

## License

MIT License - xem file LICENSE để biết thêm chi tiết.

## 🔗 Links

- [Cuproof Core Documentation](../README.md)
- [Blockchain Integration Guide](../blockchain_integration.md)
- [System Report](../BAO_CAO_CUPROOF.md)