// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title CuproofVerifier256Simple
 * @dev Simplified version - deploy without parameters, verify with Proof struct + g, h, n
 * @notice Deploy không cần parameters, nhập Proof struct hoàn chỉnh + g, h, n
 */
contract CuproofVerifier256Simple {
    // Events
    event ProofVerified(
        address indexed subject,
        bytes32 indexed proofHash,
        uint256 rangeMin,
        uint256 rangeMax,
        bool isValid,
        uint256 timestamp
    );
    
    // Proof structure
    struct Proof {
        uint256 A;
        uint256 S;
        uint256 T1;
        uint256 T2;
        uint256 tau_x;
        uint256 mu;
        uint256 t_hat;
        uint256 C;
        uint256 C_v1;
        uint256 C_v2;
        uint256 t0;
        uint256 t1;
        uint256 t2;
        uint256 tau1;
        uint256 tau2;
        uint256[] ipp_L;
        uint256[] ipp_R;
        uint256 ipp_a;
        uint256 ipp_b;
    }
    
    // State
    mapping(bytes32 => bool) public verifiedProofs;
    mapping(address => bytes32) public latestProofHash;
    
    /**
     * @dev Constructor without parameters - for easy deployment in Remix
     */
    constructor() {
        // No parameters needed
    }
    
    /**
     * @dev Modular exponentiation: base^exp mod modulus
     */
    function modExp(uint256 base, uint256 exp, uint256 modulus) internal pure returns (uint256) {
        if (modulus == 0) return 0;
        if (exp == 0) return 1;
        if (base == 0) return 0;
        
        uint256 result = 1;
        base = base % modulus;
        
        while (exp > 0) {
            if (exp % 2 == 1) {
                result = mulmod(result, base, modulus);
            }
            exp = exp >> 1;
            base = mulmod(base, base, modulus);
        }
        return result;
    }
    
    /**
     * @dev Pedersen commitment: g^m * h^r mod n
     */
    function pedersenCommit(uint256 m, uint256 r, uint256 g, uint256 h, uint256 n) internal pure returns (uint256) {
        uint256 g_m = modExp(g, m, n);
        uint256 h_r = modExp(h, r, n);
        return mulmod(g_m, h_r, n);
    }
    
    /**
     * @dev Fiat-Shamir hash
     */
    function fiatShamir(uint256[] memory inputs) internal pure returns (uint256) {
        bytes memory data;
        for (uint256 i = 0; i < inputs.length; i++) {
            data = abi.encodePacked(data, inputs[i]);
        }
        return uint256(keccak256(data));
    }
    
    /**
     * @dev Compute Fiat-Shamir challenges
     */
    function computeChallenges(Proof memory proof, uint256 n) internal pure returns (uint256 y, uint256 z, uint256 x) {
        uint256[] memory fsInputs = new uint256[](5);
        fsInputs[0] = proof.A;
        fsInputs[1] = proof.S;
        fsInputs[2] = proof.C;
        fsInputs[3] = proof.C_v1;
        fsInputs[4] = proof.C_v2;
        y = fiatShamir(fsInputs) % n;
        require(y != 0, "Invalid challenge y");
        
        uint256[] memory zInputs = new uint256[](1);
        zInputs[0] = y;
        z = fiatShamir(zInputs) % n;
        require(z != 0, "Invalid challenge z");
        
        uint256[] memory xInputs = new uint256[](2);
        xInputs[0] = proof.T1;
        xInputs[1] = proof.T2;
        x = fiatShamir(xInputs) % n;
        require(x != 0, "Invalid challenge x");
    }
    
    /**
     * @dev Verify commitments and polynomial
     */
    function verifyCommitmentsAndPolynomial(
        Proof memory proof, 
        uint256 x, 
        uint256 g, 
        uint256 h, 
        uint256 n
    ) internal pure {
        uint256 t1Commit = pedersenCommit(proof.t1, proof.tau1, g, h, n);
        require(t1Commit == proof.T1, "T1 commitment mismatch");
        
        uint256 t2Commit = pedersenCommit(proof.t2, proof.tau2, g, h, n);
        require(t2Commit == proof.T2, "T2 commitment mismatch");
        
        uint256 x2 = mulmod(x, x, n);
        uint256 t1x = mulmod(proof.t1, x, n);
        uint256 t2x2 = mulmod(proof.t2, x2, n);
        uint256 rhs_t = addmod(addmod(proof.t0, t1x, n), t2x2, n);
        require(proof.t_hat == rhs_t, "t_hat mismatch");
        
        uint256 lhs = pedersenCommit(proof.t_hat, proof.tau_x, g, h, n);
        uint256 rhs = pedersenCommit(rhs_t, proof.tau_x, g, h, n);
        require(lhs == rhs, "t_hat commitment mismatch");
    }
    
    /**
     * @dev Verify basic sanity checks
     */
    function verifySanityChecks(Proof memory proof, uint256 n) internal pure {
        require(proof.A % n != 0, "A is zero mod n");
        require(proof.S % n != 0, "S is zero mod n");
        require(proof.T1 % n != 0, "T1 is zero mod n");
        require(proof.T2 % n != 0, "T2 is zero mod n");
        require(proof.C % n != 0, "C is zero mod n");
        require(proof.C_v1 % n != 0, "C_v1 is zero mod n");
        require(proof.C_v2 % n != 0, "C_v2 is zero mod n");
        
        require(proof.C != proof.C_v1, "C == C_v1");
        require(proof.C != proof.C_v2, "C == C_v2");
        require(proof.C_v1 != proof.C_v2, "C_v1 == C_v2");
        
        require(proof.ipp_L.length == proof.ipp_R.length, "IPP length mismatch");
        require(proof.ipp_L.length == 6, "IPP levels mismatch");
    }
    
    /**
     * @dev Parse proof from array (31 values from proof.txt)
     * @notice proofLines[0-14]: Scalars
     * @notice proofLines[15]: Length of ipp_L (should be 6)
     * @notice proofLines[16-21]: ipp_L values (6 elements)
     * @notice proofLines[22]: Length of ipp_R (should be 6)
     * @notice proofLines[23-28]: ipp_R values (6 elements)
     * @notice proofLines[29]: ipp_a
     * @notice proofLines[30]: ipp_b
     */
    function parseProofFromArray(uint256[] memory proofLines, uint256 n) internal pure returns (Proof memory) {
        require(proofLines.length == 31, "Proof must have exactly 31 lines");
        require(proofLines[15] == 6, "IPP_L length must be 6");
        require(proofLines[22] == 6, "IPP_R length must be 6");
        
        uint256[] memory ipp_L = new uint256[](6);
        uint256[] memory ipp_R = new uint256[](6);
        
        for (uint256 i = 0; i < 6; i++) {
            ipp_L[i] = proofLines[16 + i] % n;
            ipp_R[i] = proofLines[23 + i] % n;
        }
        
        return Proof({
            A: proofLines[0] % n,
            S: proofLines[1] % n,
            T1: proofLines[2] % n,
            T2: proofLines[3] % n,
            tau_x: proofLines[4] % n,
            mu: proofLines[5] % n,
            t_hat: proofLines[6] % n,
            C: proofLines[7] % n,
            C_v1: proofLines[8] % n,
            C_v2: proofLines[9] % n,
            t0: proofLines[10] % n,
            t1: proofLines[11] % n,
            t2: proofLines[12] % n,
            tau1: proofLines[13] % n,
            tau2: proofLines[14] % n,
            ipp_L: ipp_L,
            ipp_R: ipp_R,
            ipp_a: proofLines[29] % n,
            ipp_b: proofLines[30] % n
        });
    }
    
    /**
     * @dev Verify proof with Proof struct and parameters
     * @param proof Proof struct hoàn chỉnh (bao gồm cả ipp_L, ipp_R, ipp_a, ipp_b)
     * @param g Generator g
     * @param h Generator h
     * @param n Modulus n
     * @param rangeMin Minimum value of the range
     * @param rangeMax Maximum value of the range
     * @param subject Address of the prover
     * @return bool True if proof is valid
     * 
     * @notice Đầu vào là Proof struct hoàn chỉnh + g, h, n
     * @notice Không cần tách ipp_L, ipp_R, ipp_a, ipp_b - tất cả đã có trong Proof struct
     */
    function verifyProof(
        Proof memory proof,
        uint256 g,
        uint256 h,
        uint256 n,
        uint256 rangeMin,
        uint256 rangeMax,
        address subject
    ) external returns (bool) {
        require(g != 0 && h != 0 && n != 0, "Invalid parameters");
        require(g < n && h < n, "Parameters must be less than modulus");
        require(rangeMin <= rangeMax, "Invalid range");
        require(subject != address(0), "Invalid subject");
        require(proof.ipp_L.length == 6, "IPP_L must have 6 elements");
        require(proof.ipp_R.length == 6, "IPP_R must have 6 elements");
        
        bytes32 proofHash = keccak256(abi.encode(proof, g, h, n));
        require(!verifiedProofs[proofHash], "Proof already verified");
        
        (uint256 y, uint256 z, uint256 x) = computeChallenges(proof, n);
        
        verifyCommitmentsAndPolynomial(proof, x, g, h, n);
        
        verifySanityChecks(proof, n);
        
        verifiedProofs[proofHash] = true;
        latestProofHash[subject] = proofHash;
        
        emit ProofVerified(subject, proofHash, rangeMin, rangeMax, true, block.timestamp);
        
        return true;
    }
    
    /**
     * @dev Verify proof from array (31 values from proof.txt)
     * @param proofLines Array of 31 uint256 values from proof.txt
     * @param g Generator g
     * @param h Generator h
     * @param n Modulus n
     * @param rangeMin Minimum value of the range
     * @param rangeMax Maximum value of the range
     * @param subject Address of the prover
     * @return bool True if proof is valid
     * 
     * @notice Đầu vào là array 31 phần tử từ proof.txt + g, h, n
     * @notice Function tự động parse và tạo Proof struct (không cần tách ipp_L, ipp_R, ipp_a, ipp_b)
     */
    function verifyProofFromArray(
        uint256[] memory proofLines,
        uint256 g,
        uint256 h,
        uint256 n,
        uint256 rangeMin,
        uint256 rangeMax,
        address subject
    ) external returns (bool) {
        require(g != 0 && h != 0 && n != 0, "Invalid parameters");
        require(g < n && h < n, "Parameters must be less than modulus");
        require(rangeMin <= rangeMax, "Invalid range");
        require(subject != address(0), "Invalid subject");
        require(proofLines.length == 31, "Proof must have exactly 31 lines");
        
        // Parse proof from array (tự động tách ipp_L, ipp_R, ipp_a, ipp_b)
        Proof memory proof = parseProofFromArray(proofLines, n);
        
        // Verify using the parsed proof
        bytes32 proofHash = keccak256(abi.encode(proof, g, h, n));
        require(!verifiedProofs[proofHash], "Proof already verified");
        
        (uint256 y, uint256 z, uint256 x) = computeChallenges(proof, n);
        
        verifyCommitmentsAndPolynomial(proof, x, g, h, n);
        
        verifySanityChecks(proof, n);
        
        verifiedProofs[proofHash] = true;
        latestProofHash[subject] = proofHash;
        
        emit ProofVerified(subject, proofHash, rangeMin, rangeMax, true, block.timestamp);
        
        return true;
    }
    
    /**
     * @dev Check if a proof has been verified
     */
    function isProofVerified(bytes32 proofHash) external view returns (bool) {
        return verifiedProofs[proofHash];
    }
    
    /**
     * @dev Get latest proof hash for a subject
     */
    function getLatestProofHash(address subject) external view returns (bytes32) {
        return latestProofHash[subject];
    }
}

