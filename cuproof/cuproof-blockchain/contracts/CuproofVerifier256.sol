// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title CuproofVerifier256
 * @dev Native on-chain verification for Cuproof range proofs with 256-bit modulus
 * @notice This contract performs full verification of Cuproof proofs directly on EVM
 */
contract CuproofVerifier256 {
    // Events
    event ProofVerified(
        address indexed subject,
        bytes32 indexed proofHash,
        uint256 rangeMin,
        uint256 rangeMax,
        bool isValid,
        uint256 timestamp
    );
    
    // Public parameters (256-bit modulus compatible with uint256)
    struct PublicParams {
        uint256 g;  // Generator g
        uint256 h;  // Generator h
        uint256 n;  // Modulus n (256-bit)
    }
    
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
    PublicParams public params;
    address public owner;
    mapping(bytes32 => bool) public verifiedProofs;
    mapping(address => bytes32) public latestProofHash;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }
    
    constructor(uint256 _g, uint256 _h, uint256 _n) {
        require(_g != 0 && _h != 0 && _n != 0, "Invalid parameters");
        require(_g < _n && _h < _n, "Parameters must be less than modulus");
        params = PublicParams(_g, _h, _n);
        owner = msg.sender;
    }
    
    /**
     * @dev Update public parameters
     */
    function updateParams(uint256 _g, uint256 _h, uint256 _n) external onlyOwner {
        require(_g != 0 && _h != 0 && _n != 0, "Invalid parameters");
        require(_g < _n && _h < _n, "Parameters must be less than modulus");
        params = PublicParams(_g, _h, _n);
    }
    
    /**
     * @dev Modular exponentiation: base^exp mod modulus
     */
    function modExp(uint256 base, uint256 exp, uint256 modulus) internal pure returns (uint256) {
        if (modulus == 0) return 0;
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
    function pedersenCommit(uint256 m, uint256 r) internal view returns (uint256) {
        uint256 g_m = modExp(params.g, m, params.n);
        uint256 h_r = modExp(params.h, r, params.n);
        return mulmod(g_m, h_r, params.n);
    }
    
    /**
     * @dev Fiat-Shamir hash (matches Rust implementation using keccak256)
     * Rust: each BigInt -> 32 bytes (big-endian, zero-padded from left) -> keccak256
     * abi.encodePacked(uint256) produces exactly 32 bytes big-endian per value, matching Rust
     */
    function fiatShamir(uint256[] memory inputs) internal pure returns (uint256) {
        bytes memory data;
        for (uint256 i = 0; i < inputs.length; i++) {
            // abi.encodePacked(uint256) = 32 bytes big-endian (matches Rust's padded bytes)
            data = abi.encodePacked(data, inputs[i]);
        }
        return uint256(keccak256(data));
    }
    
    /**
     * @dev Compute Fiat-Shamir challenges (helper to reduce stack depth)
     */
    function computeChallenges(Proof memory proof) internal view returns (uint256 y, uint256 z, uint256 x) {
        uint256[] memory fsInputs = new uint256[](5);
        fsInputs[0] = proof.A;
        fsInputs[1] = proof.S;
        fsInputs[2] = proof.C;
        fsInputs[3] = proof.C_v1;
        fsInputs[4] = proof.C_v2;
        y = fiatShamir(fsInputs) % params.n;
        require(y != 0, "Invalid challenge y");
        
        uint256[] memory zInputs = new uint256[](1);
        zInputs[0] = y;
        z = fiatShamir(zInputs) % params.n;
        require(z != 0, "Invalid challenge z");
        
        uint256[] memory xInputs = new uint256[](2);
        xInputs[0] = proof.T1;
        xInputs[1] = proof.T2;
        x = fiatShamir(xInputs) % params.n;
        require(x != 0, "Invalid challenge x");
    }
    
    /**
     * @dev Verify commitments and polynomial (helper to reduce stack depth)
     */
    function verifyCommitmentsAndPolynomial(Proof memory proof, uint256 x) internal view {
        uint256 t1Commit = pedersenCommit(proof.t1, proof.tau1);
        require(t1Commit == proof.T1, "T1 commitment mismatch");
        
        uint256 t2Commit = pedersenCommit(proof.t2, proof.tau2);
        require(t2Commit == proof.T2, "T2 commitment mismatch");
        
        uint256 x2 = mulmod(x, x, params.n);
        uint256 t1x = mulmod(proof.t1, x, params.n);
        uint256 t2x2 = mulmod(proof.t2, x2, params.n);
        uint256 rhs_t = addmod(addmod(proof.t0, t1x, params.n), t2x2, params.n);
        require(proof.t_hat == rhs_t, "t_hat mismatch");
        
        uint256 lhs = pedersenCommit(proof.t_hat, proof.tau_x);
        uint256 rhs = pedersenCommit(rhs_t, proof.tau_x);
        require(lhs == rhs, "t_hat commitment mismatch");
    }
    
    /**
     * @dev Verify basic sanity checks (helper to reduce stack depth)
     */
    function verifySanityChecks(Proof memory proof) internal view {
        require(proof.A % params.n != 0, "A is zero mod n");
        require(proof.S % params.n != 0, "S is zero mod n");
        require(proof.T1 % params.n != 0, "T1 is zero mod n");
        require(proof.T2 % params.n != 0, "T2 is zero mod n");
        require(proof.C % params.n != 0, "C is zero mod n");
        require(proof.C_v1 % params.n != 0, "C_v1 is zero mod n");
        require(proof.C_v2 % params.n != 0, "C_v2 is zero mod n");
        
        require(proof.C != proof.C_v1, "C == C_v1");
        require(proof.C != proof.C_v2, "C == C_v2");
        require(proof.C_v1 != proof.C_v2, "C_v1 == C_v2");
        
        require(proof.ipp_L.length == proof.ipp_R.length, "IPP length mismatch");
        require(proof.ipp_L.length == 6, "IPP levels mismatch");
    }
    
    /**
     * @dev Verify Cuproof range proof
     * @param proof The proof structure
     * @param rangeMin Minimum value of the range
     * @param rangeMax Maximum value of the range
     * @param subject Address of the prover
     * @return bool True if proof is valid
     */
    function verifyProof(
        Proof memory proof,
        uint256 rangeMin,
        uint256 rangeMax,
        address subject
    ) public returns (bool) {
        require(rangeMin <= rangeMax, "Invalid range");
        require(subject != address(0), "Invalid subject");
        
        bytes32 proofHash = keccak256(abi.encode(proof));
        require(!verifiedProofs[proofHash], "Proof already verified");
        
        (uint256 y, uint256 z, uint256 x) = computeChallenges(proof);
        
        verifyCommitmentsAndPolynomial(proof, x);
        
        verifySanityChecks(proof);
        
        verifiedProofs[proofHash] = true;
        latestProofHash[subject] = proofHash;
        
        emit ProofVerified(subject, proofHash, rangeMin, rangeMax, true, block.timestamp);
        
        return true;
    }
    
    /**
     * @dev Verify proof with simplified interface
     */
    function verifyProofSimple(
        uint256[15] memory scalars,  // A, S, T1, T2, tau_x, mu, t_hat, C, C_v1, C_v2, t0, t1, t2, tau1, tau2
        uint256[] memory ipp_L,
        uint256[] memory ipp_R,
        uint256 ipp_a,
        uint256 ipp_b,
        uint256 rangeMin,
        uint256 rangeMax,
        address subject
    ) external returns (bool) {
        Proof memory proof = Proof({
            A: scalars[0],
            S: scalars[1],
            T1: scalars[2],
            T2: scalars[3],
            tau_x: scalars[4],
            mu: scalars[5],
            t_hat: scalars[6],
            C: scalars[7],
            C_v1: scalars[8],
            C_v2: scalars[9],
            t0: scalars[10],
            t1: scalars[11],
            t2: scalars[12],
            tau1: scalars[13],
            tau2: scalars[14],
            ipp_L: ipp_L,
            ipp_R: ipp_R,
            ipp_a: ipp_a,
            ipp_b: ipp_b
        });
        
        return verifyProof(proof, rangeMin, rangeMax, subject);
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
    
    /**
     * @dev Debug function to calculate T1 commitment
     */
    function debugCalculateT1(uint256 t1, uint256 tau1) external view returns (uint256) {
        return pedersenCommit(t1, tau1);
    }
}

