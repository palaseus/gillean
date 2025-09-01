/**
 * Type definitions for Gillean Blockchain TypeScript SDK
 */

// SDK Configuration
export interface SDKConfig {
  apiUrl: string;
  wsUrl?: string;
  timeout?: number;
  retryAttempts?: number;
  apiKey?: string;
}

// Wallet Types
export interface WalletInfo {
  address: string;
  name: string;
  balance: number;
  createdAt: Date;
  isEncrypted: boolean;
}

export interface WalletCreationOptions {
  name?: string;
  password?: string;
  encryptionEnabled?: boolean;
}

// Transaction Types
export interface Transaction {
  id: string;
  sender: string;
  receiver: string;
  amount: number;
  message?: string;
  timestamp: Date;
  status: TransactionStatus;
  blockHash?: string;
  confirmations?: number;
}

export interface PrivateTransaction extends Transaction {
  zkProof: string;
  commitment: string;
  memo?: string;
}

export interface StateChannelTransaction extends Transaction {
  channelId: string;
  channelState: string;
  isOffChain: boolean;
}

export enum TransactionStatus {
  Pending = 'pending',
  Confirmed = 'confirmed',
  Failed = 'failed',
  Rejected = 'rejected'
}

export enum TransactionType {
  Regular = 'regular',
  Private = 'private',
  StateChannel = 'state_channel',
  CrossChain = 'cross_chain',
  Contract = 'contract'
}

// Smart Contract Types
export interface ContractInfo {
  address: string;
  name: string;
  code: string;
  abi?: string;
  deployedAt: Date;
  creator: string;
  gasUsed: number;
}

export interface ContractCall {
  contractAddress: string;
  method: string;
  params: any[];
  gasLimit?: number;
  gasPrice?: number;
}

export interface ContractDeployment {
  name: string;
  code: string;
  constructorParams?: any[];
  gasLimit?: number;
  gasPrice?: number;
}

// State Channel Types
export interface StateChannel {
  id: string;
  participant1: string;
  participant2: string;
  balance1: number;
  balance2: number;
  status: ChannelStatus;
  createdAt: Date;
  updatedAt: Date;
  timeout: Date;
}

export enum ChannelStatus {
  Open = 'open',
  Updating = 'updating',
  Closing = 'closing',
  Closed = 'closed',
  Disputed = 'disputed'
}

// Ethereum Integration Types
export interface EthereumTransfer {
  id: string;
  fromGillean: string;
  toEthereum: string;
  amount: number;
  status: TransferStatus;
  createdAt: Date;
  ethereumTxHash?: string;
  gasUsed?: number;
  gasPrice?: number;
}

export enum TransferStatus {
  Pending = 'pending',
  Processing = 'processing',
  Completed = 'completed',
  Failed = 'failed'
}

// DID Types
export interface DIDDocument {
  id: string;
  controller?: string;
  verificationMethods: VerificationMethod[];
  authentication: string[];
  assertionMethod: string[];
  keyAgreement: string[];
  serviceEndpoints: ServiceEndpoint[];
  created: Date;
  updated: Date;
}

export interface VerificationMethod {
  id: string;
  controller: string;
  keyType: string;
  publicKeyMultibase: string;
  publicKeyJwk?: any;
}

export interface ServiceEndpoint {
  id: string;
  serviceType: string;
  serviceEndpoint: string;
}

export interface DIDVerificationResult {
  isValid: boolean;
  errorMessage?: string;
  verificationMethod?: string;
}

// Governance Types
export interface GovernanceProposal {
  id: string;
  title: string;
  description: string;
  proposer: string;
  proposalType: ProposalType;
  contractCode?: string;
  parameters: Record<string, string>;
  votingPeriod: number;
  quorum: number;
  createdAt: Date;
  votingStart: number;
  votingEnd: number;
  status: ProposalStatus;
  totalVotes: number;
  yesVotes: number;
  noVotes: number;
  executedAt?: Date;
}

export enum ProposalType {
  ProtocolUpgrade = 'protocol_upgrade',
  ParameterChange = 'parameter_change',
  ContractDeployment = 'contract_deployment',
  EmergencyAction = 'emergency_action',
  TreasuryAllocation = 'treasury_allocation'
}

export enum ProposalStatus {
  Active = 'active',
  Passed = 'passed',
  Failed = 'failed',
  Executed = 'executed',
  Cancelled = 'cancelled'
}

export interface Vote {
  proposalId: string;
  voter: string;
  vote: VoteChoice;
  stakeAmount: number;
  votedAt: Date;
  blockNumber: number;
}

export enum VoteChoice {
  Yes = 'yes',
  No = 'no',
  Abstain = 'abstain'
}

// Simulation Types
export interface SimulationConfig {
  durationBlocks: number;
  numNodes: number;
  numWallets: number;
  transactionRate: number;
  zkpEnabled: boolean;
  stateChannelsEnabled: boolean;
  ethereumIntegrationEnabled: boolean;
  governanceEnabled: boolean;
  networkConditions: NetworkConditions;
  shardConfig: ShardConfig;
  failureScenarios: FailureScenario[];
}

export interface NetworkConditions {
  latencyMs: number;
  bandwidthMbps: number;
  packetLossRate: number;
  nodeFailureRate: number;
}

export interface ShardConfig {
  numShards: number;
  crossShardTxRate: number;
  shardLoadBalancing: boolean;
}

export interface FailureScenario {
  type: 'node_failure' | 'network_partition' | 'high_latency' | 'invalid_transaction';
  nodeId?: number;
  blockNumber?: number;
  durationBlocks?: number;
  latencyMs?: number;
  transactionId?: string;
}

export interface SimulationResult {
  config: SimulationConfig;
  metrics: SimulationMetrics;
  events: SimulationEvent[];
  durationSeconds: number;
  success: boolean;
  errorMessage?: string;
}

export interface SimulationMetrics {
  totalBlocks: number;
  totalTransactions: number;
  totalZkpTransactions: number;
  totalStateChannelTransactions: number;
  totalEthereumTransfers: number;
  totalGovernanceProposals: number;
  averageBlockTime: number;
  averageTransactionThroughput: number;
  zkpGenerationTime: number;
  stateChannelSuccessRate: number;
  ethereumBridgeSuccessRate: number;
  governanceParticipationRate: number;
  shardUtilization: Record<number, number>;
  nodePerformance: Record<number, NodePerformance>;
}

export interface NodePerformance {
  blocksMined: number;
  transactionsProcessed: number;
  uptimePercentage: number;
  averageResponseTime: number;
}

export interface SimulationEvent {
  blockNumber: number;
  eventType: SimulationEventType;
  timestamp: Date;
  details: Record<string, string>;
}

export enum SimulationEventType {
  BlockMined = 'block_mined',
  TransactionProcessed = 'transaction_processed',
  ZKPGenerated = 'zkp_generated',
  StateChannelOpened = 'state_channel_opened',
  StateChannelClosed = 'state_channel_closed',
  EthereumTransferInitiated = 'ethereum_transfer_initiated',
  EthereumTransferCompleted = 'ethereum_transfer_completed',
  GovernanceProposalCreated = 'governance_proposal_created',
  GovernanceVoteCast = 'governance_vote_cast',
  NodeFailure = 'node_failure',
  NetworkPartition = 'network_partition',
  ShardCreated = 'shard_created',
  CrossShardTransaction = 'cross_shard_transaction'
}

// Analytics Types
export interface BlockchainMetrics {
  totalBlocks: number;
  totalTransactions: number;
  pendingTransactions: number;
  averageBlockTime: number;
  networkHashRate: number;
  difficulty: number;
  zkpStats: ZKPStats;
  stateChannelStats: StateChannelStats;
  shardStats: ShardStats;
  ethereumBridgeStats: BridgeStats;
  governanceStats: GovernanceStats;
}

export interface ZKPStats {
  totalProofs: number;
  averageGenerationTime: number;
  successRate: number;
  cacheHitRate: number;
}

export interface StateChannelStats {
  openChannels: number;
  totalUpdates: number;
  averageChannelLifetime: number;
  successRate: number;
}

export interface ShardStats {
  numShards: number;
  totalThroughput: number;
  averageLatency: number;
  crossShardTransactions: number;
  utilization: Record<number, number>;
}

export interface BridgeStats {
  totalTransfers: number;
  completedTransfers: number;
  failedTransfers: number;
  pendingTransfers: number;
  totalVolume: number;
}

export interface GovernanceStats {
  totalProposals: number;
  activeProposals: number;
  passedProposals: number;
  failedProposals: number;
  executedProposals: number;
  cancelledProposals: number;
  totalVotes: number;
}

// Error Types
export class GilleanError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode?: number
  ) {
    super(message);
    this.name = 'GilleanError';
  }
}

// WebSocket Types
export interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: Date;
}

export interface WebSocketConfig {
  url: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage?: (message: WebSocketMessage) => void;
  onError?: (error: Error) => void;
  onClose?: () => void;
  onOpen?: () => void;
}
