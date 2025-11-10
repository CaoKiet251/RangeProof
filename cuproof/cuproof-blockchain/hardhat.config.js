require("@nomiclabs/hardhat-ethers");
require("@nomiclabs/hardhat-waffle");

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      },
      viaIR: true  // Enable IR-based code generation to avoid "Stack too deep" errors
    }
  },
  networks: {
    hardhat: {
      chainId: 31337,
      accounts: {
        mnemonic: "test test test test test test test test test test test junk",
        count: 20,
        accountsBalance: "10000000000000000000000" // 10000 ETH
      }
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      chainId: 31337,
      accounts: {
        mnemonic: "test test test test test test test test test test test junk"
      }
    },
    // Add other networks as needed
    // mainnet: {
    //   url: process.env.MAINNET_RPC_URL,
    //   accounts: [process.env.PRIVATE_KEY],
    //   gasPrice: 20000000000, // 20 gwei
    // },
    // goerli: {
    //   url: process.env.GOERLI_RPC_URL,
    //   accounts: [process.env.PRIVATE_KEY],
    // }
  },
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts"
  },
  mocha: {
    timeout: 40000
  }
};
