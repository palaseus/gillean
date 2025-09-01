# Gillean Blockchain TypeScript SDK v2.0.0

A comprehensive TypeScript SDK for interacting with the Gillean blockchain platform. This SDK provides full access to all blockchain functionality including wallet management, transactions, smart contracts, zero-knowledge proofs, state channels, sharding, cross-chain operations, decentralized identity, governance, and simulation.

## Features

- **Wallet Management**: Create, import, export, and manage encrypted wallets
- **Transaction Handling**: Send regular and private transactions with ZKP support
- **Smart Contracts**: Deploy and interact with WASM smart contracts
- **State Channels**: Open, update, and close off-chain state channels
- **Ethereum Integration**: Cross-chain transfers to Ethereum testnets
- **Decentralized Identity (DID)**: Create and manage DIDs for authentication
- **Governance**: Participate in on-chain governance proposals and voting
- **Simulation**: Run blockchain simulations for testing and analysis
- **Analytics**: Real-time blockchain metrics and monitoring
- **WebSocket Support**: Real-time event subscriptions
- **Type Safety**: Full TypeScript support with comprehensive type definitions

## Installation

```bash
npm install gillean-sdk-typescript
```

## Quick Start

```typescript
import { GilleanSDK } from 'gillean-sdk-typescript';

// Initialize the SDK
const sdk = new GilleanSDK({
  apiUrl: 'http://localhost:3000',
  wsUrl: 'ws://localhost:3000/ws',
  timeout: 30000,
  retryAttempts: 3
});

// Initialize and connect
await sdk.initialize();

// Create a wallet
const wallet = await sdk.wallet.createWallet({
  name: 'My Wallet',
  password: 'secure_password',
  encryptionEnabled: true
});

// Send a transaction
const transaction = await sdk.transactions.sendTransaction(
  wallet.address,
  'recipient_address',
  100.0,
  'Payment for services'
);

// Create a private transaction with ZKP
const privateTx = await sdk.transactions.createPrivateTransaction(
  wallet.address,
  'recipient_address',
  50.0,
  'secure_password',
  'Private payment'
);

// Transfer to Ethereum
const ethTransfer = await sdk.ethereum.transferToEthereum(
  wallet.address,
  '0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6',
  25.0,
  'secure_password'
);

// Create a DID
const did = await sdk.did.createDID();

// Link DID to wallet
await sdk.did.linkDIDToWallet(did.id, wallet.address);

// Create a governance proposal
const proposal = await sdk.governance.createProposal(
  wallet.address,
  'Increase Block Size',
  'Proposal to increase maximum block size to 2MB',
  'parameter_change',
  100,
  50.0
);

// Vote on proposal
await sdk.governance.voteOnProposal(
  proposal.id,
  wallet.address,
  'yes',
  1000.0
);

// Run a simulation
const simulation = await sdk.simulation.runSimulation({
  durationBlocks: 100,
  numNodes: 5,
  numWallets: 10,
  transactionRate: 5.0,
  zkpEnabled: true,
  stateChannelsEnabled: true,
  ethereumIntegrationEnabled: true,
  governanceEnabled: true,
  networkConditions: {
    latencyMs: 50,
    bandwidthMbps: 1000.0,
    packetLossRate: 0.01,
    nodeFailureRate: 0.001
  },
  shardConfig: {
    numShards: 4,
    crossShardTxRate: 0.2,
    shardLoadBalancing: true
  },
  failureScenarios: []
});

// Get blockchain metrics
const metrics = await sdk.analytics.getMetrics();
console.log('Blockchain metrics:', metrics);
```

## API Reference

### SDK Configuration

```typescript
interface SDKConfig {
  apiUrl: string;           // Gillean API server URL
  wsUrl?: string;           // WebSocket URL for real-time updates
  timeout?: number;         // Request timeout in milliseconds
  retryAttempts?: number;   // Number of retry attempts
  apiKey?: string;          // API key for authentication
}
```

### Wallet Management

```typescript
// Create wallet
const wallet = await sdk.wallet.createWallet({
  name: 'My Wallet',
  password: 'secure_password',
  encryptionEnabled: true
});

// Get wallet info
const walletInfo = await sdk.wallet.getWallet(wallet.address);

// Get balance
const balance = await sdk.wallet.getBalance(wallet.address);

// Import wallet
const importedWallet = await sdk.wallet.importWallet(privateKey, 'Imported Wallet');

// Export private key
const privateKey = await sdk.wallet.exportPrivateKey(wallet.address, 'password');
```

### Transaction Management

```typescript
// Send regular transaction
const tx = await sdk.transactions.sendTransaction(
  fromAddress,
  toAddress,
  amount,
  message,
  password
);

// Create private transaction with ZKP
const privateTx = await sdk.transactions.createPrivateTransaction(
  fromAddress,
  toAddress,
  amount,
  password,
  memo
);

// Get transaction status
const status = await sdk.transactions.getTransactionStatus(txId);

// Get pending transactions
const pending = await sdk.transactions.getPendingTransactions();
```

### Smart Contracts

```typescript
// Deploy contract
const contract = await sdk.contracts.deployContract({
  name: 'MyContract',
  code: 'contract code here',
  constructorParams: [],
  gasLimit: 1000000,
  gasPrice: 0.000001
});

// Call contract method
const result = await sdk.contracts.callContract({
  contractAddress: contract.address,
  method: 'transfer',
  params: ['recipient', 100],
  gasLimit: 100000
});

// Get contract info
const contractInfo = await sdk.contracts.getContract(contract.address);
```

### Ethereum Integration

```typescript
// Transfer to Ethereum
const transfer = await sdk.ethereum.transferToEthereum(
  fromAddress,
  toEthereumAddress,
  amount,
  password
);

// Get Ethereum balance
const ethBalance = await sdk.ethereum.getEthereumBalance(ethereumAddress);

// Get transfer status
const status = await sdk.ethereum.getTransferStatus(transferId);

// Get bridge statistics
const stats = await sdk.ethereum.getBridgeStats();
```

### Decentralized Identity (DID)

```typescript
// Create DID
const did = await sdk.did.createDID(controller, serviceEndpoints);

// Get DID document
const document = await sdk.did.getDIDDocument(did.id);

// Link DID to wallet
await sdk.did.linkDIDToWallet(did.id, walletAddress);

// Get DID for wallet
const linkedDid = await sdk.did.getDIDForWallet(walletAddress);

// Verify DID signature
const result = await sdk.did.verifyDIDSignature(did.id, message, signature);
```

### Governance

```typescript
// Create proposal
const proposal = await sdk.governance.createProposal(
  proposer,
  title,
  description,
  proposalType,
  votingPeriod,
  quorum,
  contractCode,
  parameters
);

// Vote on proposal
await sdk.governance.voteOnProposal(
  proposalId,
  voter,
  voteChoice,
  stakeAmount
);

// Execute passed proposal
await sdk.governance.executeProposal(proposalId);

// Get proposal info
const proposalInfo = await sdk.governance.getProposal(proposalId);

// Get all proposals
const proposals = await sdk.governance.getAllProposals();
```

### Simulation

```typescript
// Run simulation
const simulation = await sdk.simulation.runSimulation({
  durationBlocks: 100,
  numNodes: 5,
  numWallets: 10,
  transactionRate: 5.0,
  zkpEnabled: true,
  stateChannelsEnabled: true,
  ethereumIntegrationEnabled: true,
  governanceEnabled: true,
  networkConditions: {
    latencyMs: 50,
    bandwidthMbps: 1000.0,
    packetLossRate: 0.01,
    nodeFailureRate: 0.001
  },
  shardConfig: {
    numShards: 4,
    crossShardTxRate: 0.2,
    shardLoadBalancing: true
  },
  failureScenarios: [
    {
      type: 'node_failure',
      nodeId: 1,
      blockNumber: 50
    }
  ]
});

// Get simulation progress
const progress = await sdk.simulation.getSimulationProgress(simulationId);

// Get simulation results
const results = await sdk.simulation.getSimulationResults(simulationId);
```

### WebSocket Support

```typescript
// Connect to WebSocket for real-time updates
sdk.apiClient.connectWebSocket({
  url: 'ws://localhost:3000/ws',
  onMessage: (message) => {
    console.log('Received:', message);
  },
  onError: (error) => {
    console.error('WebSocket error:', error);
  },
  onClose: () => {
    console.log('WebSocket closed');
  },
  reconnectInterval: 5000,
  maxReconnectAttempts: 5
});

// Send WebSocket message
sdk.apiClient.sendWebSocketMessage({
  type: 'subscribe',
  data: { channel: 'transactions' },
  timestamp: new Date()
});
```

## Error Handling

The SDK uses a custom `GilleanError` class for error handling:

```typescript
import { GilleanError } from 'gillean-sdk-typescript';

try {
  const wallet = await sdk.wallet.createWallet();
} catch (error) {
  if (error instanceof GilleanError) {
    console.error('Gillean Error:', error.message);
    console.error('Error Code:', error.code);
    console.error('Status Code:', error.statusCode);
  } else {
    console.error('Unknown Error:', error);
  }
}
```

## Utility Functions

The SDK provides various utility functions:

```typescript
import {
  generateId,
  hashData,
  encryptData,
  decryptData,
  isValidEthereumAddress,
  isValidGilleanAddress,
  formatAmount,
  parseAmount,
  retry,
  sleep
} from 'gillean-sdk-typescript';

// Generate unique ID
const id = generateId();

// Hash data
const hash = hashData('hello world');

// Encrypt/decrypt data
const encrypted = encryptData('secret', 'key');
const decrypted = decryptData(encrypted, 'key');

// Validate addresses
const isValidEth = isValidEthereumAddress('0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6');
const isValidGil = isValidGilleanAddress('GilleanAddress123');

// Format amounts
const formatted = formatAmount(100000000, 8); // "1.00000000"
const parsed = parseAmount('1.5', 8); // 150000000

// Retry with exponential backoff
const result = await retry(
  () => sdk.wallet.getBalance(address),
  3,
  1000
);
```

## Development

### Building the SDK

```bash
# Install dependencies
npm install

# Build the SDK
npm run build

# Development mode with watch
npm run dev

# Run tests
npm test

# Lint code
npm run lint

# Format code
npm run format
```

### Testing

```bash
# Run all tests
npm test

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm test -- wallet.test.ts
```

## Examples

See the `examples/` directory for complete usage examples:

- `basic-usage.ts` - Basic SDK usage
- `wallet-management.ts` - Wallet operations
- `transactions.ts` - Transaction handling
- `smart-contracts.ts` - Smart contract operations
- `ethereum-integration.ts` - Cross-chain transfers
- `did-operations.ts` - Decentralized identity
- `governance.ts` - Governance participation
- `simulation.ts` - Blockchain simulation
- `websocket.ts` - Real-time updates

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:

- Create an issue on GitHub
- Check the documentation
- Join our community Discord

## Changelog

### v2.0.0
- Initial release with full blockchain functionality
- Ethereum testnet integration
- Decentralized Identity (DID) system
- On-chain governance framework
- TypeScript SDK support
- Simulation mode for testing
- Enhanced analytics and monitoring
- WebSocket support for real-time updates
