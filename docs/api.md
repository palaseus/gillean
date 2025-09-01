# API Reference

Complete API documentation for the Gillean blockchain platform.

## Base URL

```
http://localhost:3000/api/v1
```

## Authentication

Most endpoints require authentication. Include your API key in the request headers:

```bash
Authorization: Bearer YOUR_API_KEY
```

## Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2025-08-31T21:50:00Z"
}
```

## Blockchain API

### Get Blockchain Status

```http
GET /status
```

**Response:**
```json
{
  "success": true,
  "data": {
    "version": "2.0.0",
    "network": "testnet",
    "block_height": 1234,
    "total_transactions": 5678,
    "difficulty": 4,
    "mining_reward": 50.0,
    "consensus_type": "ProofOfStake",
    "uptime": "2h 15m 30s",
    "peers": 12,
    "sync_status": "synced"
  }
}
```

### Get Latest Block

```http
GET /blocks/latest
```

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0x1234567890abcdef...",
    "height": 1234,
    "timestamp": "2025-08-31T21:50:00Z",
    "transactions": 15,
    "difficulty": 4,
    "nonce": 12345,
    "merkle_root": "0xabcdef123456...",
    "previous_hash": "0x9876543210fedcba...",
    "total_amount": 150.0
  }
}
```

### Get Block by Height

```http
GET /blocks/{height}
```

**Parameters:**
- `height` (integer): Block height

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0x1234567890abcdef...",
    "height": 1234,
    "timestamp": "2025-08-31T21:50:00Z",
    "transactions": [
      {
        "id": "0xabc123...",
        "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
        "to": "0x1234567890123456789012345678901234567890",
        "amount": 10.0,
        "fee": 0.001,
        "timestamp": "2025-08-31T21:49:30Z"
      }
    ],
    "difficulty": 4,
    "nonce": 12345,
    "merkle_root": "0xabcdef123456...",
    "previous_hash": "0x9876543210fedcba...",
    "total_amount": 150.0
  }
}
```

### Get Block by Hash

```http
GET /blocks/hash/{hash}
```

**Parameters:**
- `hash` (string): Block hash

## Transaction API

### Send Transaction

```http
POST /transactions
```

**Request Body:**
```json
{
  "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "to": "0x1234567890123456789012345678901234567890",
  "amount": 10.0,
  "fee": 0.001,
  "message": "Payment for services",
  "signature": "0xabc123..."
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "transaction_id": "0xdef456...",
    "status": "pending",
    "timestamp": "2025-08-31T21:50:00Z"
  }
}
```

### Get Transaction

```http
GET /transactions/{tx_hash}
```

**Parameters:**
- `tx_hash` (string): Transaction hash

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "0xdef456...",
    "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "to": "0x1234567890123456789012345678901234567890",
    "amount": 10.0,
    "fee": 0.001,
    "message": "Payment for services",
    "timestamp": "2025-08-31T21:50:00Z",
    "block_height": 1234,
    "status": "confirmed",
    "confirmations": 6
  }
}
```

### Get Pending Transactions

```http
GET /transactions/pending
```

**Response:**
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "id": "0xdef456...",
        "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
        "to": "0x1234567890123456789012345678901234567890",
        "amount": 10.0,
        "fee": 0.001,
        "timestamp": "2025-08-31T21:50:00Z"
      }
    ],
    "count": 1
  }
}
```

## Wallet API

### Create Wallet

```http
POST /wallets
```

**Request Body:**
```json
{
  "name": "My Wallet",
  "password": "secure-password"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "name": "My Wallet",
    "balance": 0.0,
    "created_at": "2025-08-31T21:50:00Z"
  }
}
```

### Get Wallet Balance

```http
GET /wallets/{address}/balance
```

**Parameters:**
- `address` (string): Wallet address

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "balance": 100.0,
    "last_updated": "2025-08-31T21:50:00Z"
  }
}
```

### Get Wallet Transactions

```http
GET /wallets/{address}/transactions
```

**Parameters:**
- `address` (string): Wallet address
- `limit` (integer, optional): Number of transactions to return (default: 50)
- `offset` (integer, optional): Number of transactions to skip (default: 0)

**Response:**
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "id": "0xdef456...",
        "type": "sent",
        "amount": 10.0,
        "fee": 0.001,
        "timestamp": "2025-08-31T21:50:00Z",
        "status": "confirmed"
      }
    ],
    "total": 25,
    "limit": 50,
    "offset": 0
  }
}
```

## Smart Contract API

### Deploy Contract

```http
POST /contracts
```

**Request Body:**
```json
{
  "name": "Simple Counter",
  "bytecode": "0x606060...",
  "abi": "[{\"name\":\"increment\",\"inputs\":[],\"outputs\":[{\"type\":\"i32\"}]}]",
  "gas_limit": 1000000,
  "wallet_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "contract_address": "0x1234567890123456789012345678901234567890",
    "name": "Simple Counter",
    "gas_used": 500000,
    "deployment_tx": "0xabc123...",
    "timestamp": "2025-08-31T21:50:00Z"
  }
}
```

### Call Contract

```http
POST /contracts/{address}/call
```

**Parameters:**
- `address` (string): Contract address

**Request Body:**
```json
{
  "function": "increment",
  "args": [],
  "wallet_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "gas_limit": 100000
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "result": 42,
    "gas_used": 50000,
    "execution_time": "0.001s"
  }
}
```

### Get Contract Info

```http
GET /contracts/{address}
```

**Parameters:**
- `address` (string): Contract address

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0x1234567890123456789012345678901234567890",
    "name": "Simple Counter",
    "bytecode": "0x606060...",
    "abi": "[{\"name\":\"increment\",\"inputs\":[],\"outputs\":[{\"type\":\"i32\"}]}]",
    "owner": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "deployed_at": "2025-08-31T21:50:00Z",
    "total_calls": 15,
    "gas_used": 750000
  }
}
```

## Zero-Knowledge Proofs API

### Create Private Transaction

```http
POST /zkp/private-transactions
```

**Request Body:**
```json
{
  "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "to": "0x1234567890123456789012345678901234567890",
  "amount": 5.0,
  "memo": "Private payment"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "transaction_id": "0xzkp123...",
    "proof": "0xproof456...",
    "commitment": "0xcommit789...",
    "status": "pending"
  }
}
```

### Verify ZKP

```http
POST /zkp/verify
```

**Request Body:**
```json
{
  "proof": "0xproof456...",
  "public_inputs": ["0xcommit789..."]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "valid": true,
    "verification_time": "0.005s"
  }
}
```

## State Channels API

### Open Channel

```http
POST /channels
```

**Request Body:**
```json
{
  "participants": [
    "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "0x1234567890123456789012345678901234567890"
  ],
  "amount": 100.0,
  "timeout": 3600
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "channel_id": "0xchannel123...",
    "participants": [
      "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
      "0x1234567890123456789012345678901234567890"
    ],
    "amount": 100.0,
    "status": "open",
    "created_at": "2025-08-31T21:50:00Z"
  }
}
```

### Update Channel

```http
PUT /channels/{channel_id}
```

**Parameters:**
- `channel_id` (string): Channel ID

**Request Body:**
```json
{
  "state": {
    "balances": {
      "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6": 60.0,
      "0x1234567890123456789012345678901234567890": 40.0
    }
  },
  "signatures": ["0xsig1...", "0xsig2..."]
}
```

### Close Channel

```http
DELETE /channels/{channel_id}
```

**Parameters:**
- `channel_id` (string): Channel ID

**Request Body:**
```json
{
  "final_state": {
    "balances": {
      "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6": 60.0,
      "0x1234567890123456789012345678901234567890": 40.0
    }
  },
  "signatures": ["0xsig1...", "0xsig2..."]
}
```

## Cross-Chain Bridge API

### Initiate Transfer

```http
POST /bridge/transfers
```

**Request Body:**
```json
{
  "from_chain": "gillean",
  "to_chain": "ethereum",
  "from_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "to_address": "0x1234567890123456789012345678901234567890",
  "amount": 10.0,
  "asset": "GIL"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "transfer_id": "0xbridge123...",
    "status": "pending",
    "estimated_time": "5 minutes",
    "fee": 0.001
  }
}
```

### Get Transfer Status

```http
GET /bridge/transfers/{transfer_id}
```

**Parameters:**
- `transfer_id` (string): Transfer ID

**Response:**
```json
{
  "success": true,
  "data": {
    "transfer_id": "0xbridge123...",
    "from_chain": "gillean",
    "to_chain": "ethereum",
    "amount": 10.0,
    "status": "completed",
    "proof": "0xproof456...",
    "completed_at": "2025-08-31T21:55:00Z"
  }
}
```

## Governance API

### Create Proposal

```http
POST /governance/proposals
```

**Request Body:**
```json
{
  "title": "Increase Block Reward",
  "description": "Proposal to increase mining reward from 50 to 60 GIL",
  "proposal_type": "ParameterChange",
  "quorum": 0.6,
  "voting_period": 604800
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "proposal_id": "0xprop123...",
    "title": "Increase Block Reward",
    "status": "active",
    "voting_start": "2025-08-31T21:50:00Z",
    "voting_end": "2025-09-07T21:50:00Z"
  }
}
```

### Vote on Proposal

```http
POST /governance/proposals/{proposal_id}/vote
```

**Parameters:**
- `proposal_id` (string): Proposal ID

**Request Body:**
```json
{
  "vote": "yes",
  "voting_power": 1000.0
}
```

### Get Proposal

```http
GET /governance/proposals/{proposal_id}
```

**Parameters:**
- `proposal_id` (string): Proposal ID

**Response:**
```json
{
  "success": true,
  "data": {
    "proposal_id": "0xprop123...",
    "title": "Increase Block Reward",
    "description": "Proposal to increase mining reward from 50 to 60 GIL",
    "status": "active",
    "yes_votes": 6000.0,
    "no_votes": 2000.0,
    "abstain_votes": 1000.0,
    "total_votes": 9000.0,
    "quorum_met": true
  }
}
```

## AI Integration API

### Analyze Transaction

```http
POST /ai/analyze-transaction
```

**Request Body:**
```json
{
  "transaction": {
    "from": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "to": "0x1234567890123456789012345678901234567890",
    "amount": 10000.0,
    "timestamp": "2025-08-31T21:50:00Z"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "anomaly_score": 0.85,
    "risk_level": "high",
    "fraud_probability": 0.75,
    "recommendations": [
      "Review transaction amount",
      "Verify recipient address",
      "Consider additional verification"
    ]
  }
}
```

### Get AI Insights

```http
GET /ai/insights
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_transactions": 10000,
    "anomalies_detected": 150,
    "fraud_prevented": 25,
    "accuracy": 0.95,
    "model_version": "2.1.0",
    "last_updated": "2025-08-31T21:50:00Z"
  }
}
```

## Performance API

### Get Performance Stats

```http
GET /performance/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "cache_hit_rate": 0.89,
    "memory_usage": 0.63,
    "parallel_tasks": 10,
    "avg_response_time": "0.015s",
    "throughput_tps": 8500,
    "active_connections": 45
  }
}
```

### Get Cache Stats

```http
GET /performance/cache
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_requests": 100000,
    "hits": 89000,
    "misses": 11000,
    "hit_rate": 0.89,
    "evictions": 500,
    "memory_usage_mb": 256
  }
}
```

## Security API

### Get Security Audit

```http
GET /security/audit
```

**Response:**
```json
{
  "success": true,
  "data": {
    "audit_logs": 174,
    "threats_detected": 3,
    "encryption_strength": 256,
    "formal_verification": "passed",
    "last_audit": "2025-08-31T21:50:00Z",
    "recommendations": [
      "Update encryption keys",
      "Review access logs"
    ]
  }
}
```

### Get Threat Detection

```http
GET /security/threats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "active_threats": [
      {
        "id": "threat_001",
        "type": "suspicious_activity",
        "severity": "medium",
        "description": "Unusual transaction pattern detected",
        "detected_at": "2025-08-31T21:45:00Z",
        "status": "investigating"
      }
    ],
    "total_threats": 3,
    "mitigated": 2
  }
}
```

## Developer Tools API

### Get Debug Info

```http
GET /developer/debug
```

**Response:**
```json
{
  "success": true,
  "data": {
    "breakpoints": 5,
    "debug_logs": 60,
    "call_stack": [
      {
        "function": "process_transaction",
        "file": "blockchain.rs",
        "line": 123
      }
    ],
    "variables": {
      "tx_count": 15,
      "block_height": 1234
    }
  }
}
```

### Generate SDK

```http
POST /developer/sdk
```

**Request Body:**
```json
{
  "language": "typescript",
  "include_examples": true,
  "api_version": "v1"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "sdk_url": "https://api.gillean.org/sdk/typescript-v1.0.0.zip",
    "documentation_url": "https://docs.gillean.org/sdk/typescript",
    "examples": [
      "wallet_management.ts",
      "transaction_sending.ts",
      "contract_deployment.ts"
    ]
  }
}
```

## WebSocket API

### Connect to WebSocket

```bash
ws://localhost:3000/ws
```

### Subscribe to Events

```json
{
  "type": "subscribe",
  "events": ["new_block", "new_transaction", "wallet_update"]
}
```

### Event Types

- `new_block`: New block mined
- `new_transaction`: New transaction received
- `wallet_update`: Wallet balance updated
- `contract_event`: Smart contract event
- `channel_update`: State channel update
- `bridge_transfer`: Cross-chain transfer update

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized - Missing or invalid API key |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 422 | Unprocessable Entity - Validation error |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - Service temporarily unavailable |

## Rate Limits

- **Public endpoints**: 100 requests per minute
- **Authenticated endpoints**: 1000 requests per minute
- **WebSocket connections**: 10 concurrent connections per IP

## Pagination

Endpoints that return lists support pagination:

```http
GET /transactions?limit=20&offset=40
```

**Response:**
```json
{
  "success": true,
  "data": {
    "items": [...],
    "total": 100,
    "limit": 20,
    "offset": 40,
    "has_more": true
  }
}
```

## SDKs and Libraries

### Official SDKs

- **[Rust SDK](https://github.com/your-org/gillean-rust-sdk)**
- **[TypeScript SDK](https://github.com/your-org/gillean-ts-sdk)**
- **[Python SDK](https://github.com/your-org/gillean-python-sdk)**
- **[Go SDK](https://github.com/your-org/gillean-go-sdk)**

### Community Libraries

- **[JavaScript Client](https://github.com/community/gillean-js)**
- **[Java Client](https://github.com/community/gillean-java)**
- **[C# Client](https://github.com/community/gillean-csharp)**

## Support

For API support:

- **Documentation**: [docs.gillean.org/api](https://docs.gillean.org/api)
- **Postman Collection**: [Download API Collection](https://docs.gillean.org/api/postman)
- **OpenAPI Spec**: [Download OpenAPI Spec](https://docs.gillean.org/api/openapi.json)
- **Discord**: [discord.gg/gillean](https://discord.gg/gillean)
- **Email**: api-support@gillean.org
