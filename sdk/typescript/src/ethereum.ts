/**
 * Ethereum Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { EthereumTransfer, TransferStatus } from './types';

/**
 * Manages Ethereum integration for the Gillean blockchain
 */
export class EthereumManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Transfer tokens from Gillean to Ethereum
   */
  async transferToEthereum(
    fromAddress: string,
    toEthereumAddress: string,
    amount: number,
    password: string
  ): Promise<EthereumTransfer> {
    const data = {
      from_address: fromAddress,
      to_ethereum_address: toEthereumAddress,
      amount,
      password
    };

    return this.apiClient.post<EthereumTransfer>('/eth/transfer', data);
  }

  /**
   * Get Ethereum balance for an address
   */
  async getEthereumBalance(address: string): Promise<number> {
    const response = await this.apiClient.get<{ balance: number }>(`/eth/balance/${address}`);
    return response.balance;
  }

  /**
   * Get transfer status
   */
  async getTransferStatus(transferId: string): Promise<TransferStatus> {
    const response = await this.apiClient.get<{ status: TransferStatus }>(`/eth/transfer/${transferId}/status`);
    return response.status;
  }

  /**
   * Get all pending transfers
   */
  async getPendingTransfers(): Promise<EthereumTransfer[]> {
    return this.apiClient.get<EthereumTransfer[]>('/eth/transfers/pending');
  }

  /**
   * Get bridge statistics
   */
  async getBridgeStats(): Promise<any> {
    return this.apiClient.get('/eth/bridge/stats');
  }
}
