const fs = require('fs');
const path = require('path');
const { ethers } = require('hardhat');

/**
 * Load proof from JSON file (exported from Rust)
 */
function loadProofJson(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return JSON.parse(content);
}

/**
 * Load parameters from params.txt
 */
function loadParams(filePath) {
  const lines = fs.readFileSync(filePath, 'utf8')
    .split(/\r?\n/)
    .map(l => l.trim())
    .filter(l => l.length > 0)
    .map(l => l.replace(/^0x/i, '').toLowerCase());
  
  if (lines.length < 3) {
    throw new Error('params.txt must contain 3 lines: g, h, n');
  }
  
  return {
    g: ethers.BigNumber.from('0x' + lines[0]),
    h: ethers.BigNumber.from('0x' + lines[1]),
    n: ethers.BigNumber.from('0x' + lines[2])
  };
}

/**
 * Convert hex string to uint256
 */
function hexToUint256(hexStr) {
  const cleaned = hexStr.replace(/^0x/i, '');
  return ethers.BigNumber.from('0x' + cleaned);
}

async function main() {
  const projectRoot = path.resolve(__dirname, '../..');
  const paramsPath = path.resolve(projectRoot, 'params.txt');
  const proofJsonPath = path.resolve(projectRoot, 'proof_evm.json');
  
  if (!fs.existsSync(proofJsonPath)) {
    throw new Error(`Missing proof JSON file: ${proofJsonPath}`);
  }
  if (!fs.existsSync(paramsPath)) {
    throw new Error(`Missing params file: ${paramsPath}`);
  }
  
  // Load proof and params
  const proofData = loadProofJson(proofJsonPath);
  const params = loadParams(paramsPath);
  
  // Convert proof data to BigNumber arrays
  const scalars = proofData.scalars.map(hexToUint256);
  const ipp_L = proofData.ipp_L.map(hexToUint256);
  const ipp_R = proofData.ipp_R.map(hexToUint256);
  const ipp_a = hexToUint256(proofData.ipp_a);
  const ipp_b = hexToUint256(proofData.ipp_b);
  
  // Get signers
  const [owner, subject] = await ethers.getSigners();
  
  // Deploy or attach to CuproofVerifier256
  const VerifierFactory = await ethers.getContractFactory('CuproofVerifier256');
  
  let verifier;
  const deploymentPath = path.resolve(__dirname, '../deployment-info.json');
  
  if (fs.existsSync(deploymentPath)) {
    const dep = JSON.parse(fs.readFileSync(deploymentPath, 'utf8'));
    const verifierAddress = dep.contracts && dep.contracts.CuproofVerifier256;
    
    if (verifierAddress) {
      const code = await ethers.provider.getCode(verifierAddress);
      if (code && code !== '0x') {
        verifier = VerifierFactory.attach(verifierAddress);
        console.log('Attached to existing CuproofVerifier256:', verifierAddress);
      }
    }
  }
  
  if (!verifier) {
    // Deploy new contract
    verifier = await VerifierFactory.deploy(params.g, params.h, params.n);
    await verifier.deployed();
    console.log('Deployed new CuproofVerifier256:', verifier.address);
    
    // Save deployment info
    const deploymentInfo = {
      contracts: {
        CuproofVerifier256: verifier.address
      },
      params: {
        g: params.g.toString(),
        h: params.h.toString(),
        n: params.n.toString()
      }
    };
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
  }
  
  // Verify proof
  const rangeMin = 0;
  const rangeMax = 100;
  
  console.log('\nVerifying proof on-chain...');
  console.log('Subject:', subject.address);
  console.log('Range:', `[${rangeMin}, ${rangeMax}]`);
  console.log('Proof scalars:', scalars.length);
  console.log('IPP L length:', ipp_L.length);
  console.log('IPP R length:', ipp_R.length);
  
  try {
    const tx = await verifier.verifyProofSimple(
      scalars,
      ipp_L,
      ipp_R,
      ipp_a,
      ipp_b,
      rangeMin,
      rangeMax,
      subject.address
    );
    
    console.log('Transaction sent:', tx.hash);
    const receipt = await tx.wait();
    console.log('Transaction confirmed in block:', receipt.blockNumber);
    
    // Check proof status
    // Contract uses: keccak256(abi.encode(proof))
    // We need to encode the Proof struct exactly as the contract does
    // Proof struct: A, S, T1, T2, tau_x, mu, t_hat, C, C_v1, C_v2, t0, t1, t2, tau1, tau2, ipp_L[], ipp_R[], ipp_a, ipp_b
    const proofHash = ethers.utils.keccak256(
      ethers.utils.defaultAbiCoder.encode(
        [
          'tuple(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256[],uint256[],uint256,uint256)'
        ],
        [[
          scalars[0],  // A
          scalars[1],  // S
          scalars[2],  // T1
          scalars[3],  // T2
          scalars[4], // tau_x
          scalars[5],  // mu
          scalars[6],  // t_hat
          scalars[7],  // C
          scalars[8],  // C_v1
          scalars[9],  // C_v2
          scalars[10], // t0
          scalars[11], // t1
          scalars[12], // t2
          scalars[13], // tau1
          scalars[14], // tau2
          ipp_L,
          ipp_R,
          ipp_a,
          ipp_b
        ]]
      )
    );
    
    const latestHash = await verifier.getLatestProofHash(subject.address);
    console.log('Calculated proof hash:', proofHash);
    console.log('Latest proof hash from contract:', latestHash);
    console.log('Hash match:', proofHash === latestHash);
    
    if (latestHash !== '0x0000000000000000000000000000000000000000000000000000000000000000') {
      const isVerified = await verifier.isProofVerified(latestHash);
      console.log('Proof verified status (using latest hash):', isVerified);
      if (isVerified) {
        console.log('SUCCESS: Proof has been verified and stored on-chain!');
      }
    } else {
      console.log('WARNING: Latest hash is zero - verification may have failed or contract was redeployed');
    }
    
  } catch (error) {
    console.error('Verification failed:', error.message);
    if (error.reason) {
      console.error('Reason:', error.reason);
    }
    process.exit(1);
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });

