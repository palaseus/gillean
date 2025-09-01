/**
 * Main Gillean SDK class for TypeScript
 */

import { SDKConfig, GilleanError } from './types';
import { APIClient } from './client';
import { WalletManager } from './wallet';
import { TransactionManager } from './transactions';
import { ContractManager } from './contracts';
import { AnalyticsManager } from './analytics';
import { EthereumManager } from './ethereum';
import { DIDManager } from './did';
import { GovernanceManager } from './governance';
import { SimulationManager } from './simulation';

/**
 * Main Gillean SDK class that provides access to all blockchain functionality
 */
export class GilleanSDK {
  private config: SDKConfig;
  private apiClient: APIClient;
  public wallet: WalletManager;
  public transactions: TransactionManager;
  public contracts: ContractManager;
  public analytics: AnalyticsManager;
  public ethereum: EthereumManager;
  public did: DIDManager;
  public governance: GovernanceManager;
  public simulation: SimulationManager;

  constructor(config: SDKConfig) {
    this.config = {
      timeout: 30000,
      retryAttempts: 3,
      ...config
    };

    this.apiClient = new APIClient(this.config);
    
    // Initialize all managers
    this.wallet = new WalletManager(this.apiClient);
    this.transactions = new TransactionManager(this.apiClient);
    this.contracts = new ContractManager(this.apiClient);
    this.analytics = new AnalyticsManager(this.apiClient);
    this.ethereum = new EthereumManager(this.apiClient);
    this.did = new DIDManager(this.apiClient);
    this.governance = new GovernanceManager(this.apiClient);
    this.simulation = new SimulationManager(this.apiClient);
  }

  /**
   * Initialize the SDK and verify connectivity
   */
  async initialize(): Promise<void> {
    try {
      // Test API connectivity
      await this.apiClient.get('/health');
      console.log('Gillean SDK initialized successfully');
    } catch (error) {
      throw new GilleanError(
        'Failed to initialize SDK: Unable to connect to Gillean blockchain',
        'INITIALIZATION_ERROR',
        500
      );
    }
  }

  /**
   * Get SDK configuration
   */
  getConfig(): SDKConfig {
    return { ...this.config };
  }

  /**
   * Update SDK configuration
   */
  updateConfig(newConfig: Partial<SDKConfig>): void {
    this.config = { ...this.config, ...newConfig };
    this.apiClient.updateConfig(this.config);
  }

  /**
   * Get blockchain status
   */
  async getStatus(): Promise<any> {
    return this.apiClient.get('/blockchain/status');
  }

  /**
   * Get SDK version
   */
  getVersion(): string {
    return '2.0.0';
  }

  /**
   * Close SDK connections
   */
  async close(): Promise<void> {
    await this.apiClient.close();
  }
}

// Default export
export default GilleanSDK;
