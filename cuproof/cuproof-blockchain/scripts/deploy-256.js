const fs = require('fs');
const path = require('path');
const { ethers } = require('hardhat');

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

async function main() {
  const projectRoot = path.resolve(__dirname, '../..');
  const paramsPath = path.resolve(projectRoot, 'params.txt');
  
  if (!fs.existsSync(paramsPath)) {
    throw new Error(`Missing params file: ${paramsPath}`);
  }
  
  const params = loadParams(paramsPath);
  
  // Get network info
  const network = await ethers.provider.getNetwork();
  const networkName = network.name || 'unknown';
  
  console.log('Deploying CuproofVerifier256...');
  console.log(`Network: ${networkName} (Chain ID: ${network.chainId})`);
  console.log('Parameters:');
  console.log('  g:', params.g.toString());
  console.log('  h:', params.h.toString());
  console.log('  n:', params.n.toString());
  
  // Check if localhost is available (try to get block number)
  try {
    const blockNumber = await ethers.provider.getBlockNumber();
    console.log(`Connected to network (Current block: ${blockNumber})`);
  } catch (error) {
    if (error.message.includes('ECONNREFUSED') || error.message.includes('Cannot connect')) {
      console.error('\nError: Cannot connect to blockchain network');
      console.error('Solutions:');
      console.error('   1. Use hardhat network: npx hardhat run scripts/deploy-256.js --network hardhat');
      console.error('   2. Or start local node: npx hardhat node (in another terminal)');
      console.error('   3. Then run: npx hardhat run scripts/deploy-256.js --network localhost');
      throw new Error('Network connection failed. See tips above.');
    }
    throw error;
  }
  
  const VerifierFactory = await ethers.getContractFactory('CuproofVerifier256');
  const verifier = await VerifierFactory.deploy(params.g, params.h, params.n);
  
  await verifier.deployed();
  
  console.log('\nCuproofVerifier256 deployed to:', verifier.address);
  
  // Save deployment info
  const deploymentPath = path.resolve(__dirname, '../deployment-info.json');
  const networkInfo = await ethers.provider.getNetwork();
  const deploymentInfo = {
    contracts: {
      CuproofVerifier256: verifier.address
    },
    params: {
      g: params.g.toString(),
      h: params.h.toString(),
      n: params.n.toString()
    },
    network: {
      name: networkName,
      chainId: networkInfo.chainId.toString()
    },
    timestamp: new Date().toISOString()
  };
  
  fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
  console.log('Deployment info saved to:', deploymentPath);
  
  // Verify parameters
  const storedParams = await verifier.params();
  console.log('\nStored parameters verified:');
  console.log('  g:', storedParams.g.toString());
  console.log('  h:', storedParams.h.toString());
  console.log('  n:', storedParams.n.toString());
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });

