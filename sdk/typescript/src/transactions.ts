/**
 * Transaction Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { Transaction, PrivateTransaction, TransactionType, TransactionStatus } from './types';

/**
 * Manages transaction operations for the Gillean blockchain
 */
export class TransactionManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Send a regular transaction
   */
  async sendTransaction(
    fromAddress: string,
    toAddress: string,
    amount: number,
    message?: string,
    password?: string
  ): Promise<Transaction> {
    const data = {
      sender: fromAddress,
      receiver: toAddress,
      amount,
      message,
      password
    };

    return this.apiClient.post<Transaction>('/transaction/send', data);
  }

  /**
   * Create a private transaction using ZKP
   */
  async createPrivateTransaction(
    fromAddress: string,
    toAddress: string,
    amount: number,
    password: string,
    memo?: string
  ): Promise<PrivateTransaction> {
    const data = {
      sender: fromAddress,
      receiver: toAddress,
      amount,
      password,
      memo
    };

    return this.apiClient.post<PrivateTransaction>('/transaction/private', data);
  }

  /**
   * Get transaction by ID
   */
  async getTransaction(transactionId: string): Promise<Transaction> {
    return this.apiClient.get<Transaction>(`/transaction/${transactionId}`);
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(transactionId: string): Promise<TransactionStatus> {
    const response = await this.apiClient.get<{ status: TransactionStatus }>(`/transaction/${transactionId}/status`);
    return response.status;
  }

  /**
   * Get pending transactions
   */
  async getPendingTransactions(): Promise<Transaction[]> {
    return this.apiClient.get<Transaction[]>('/transaction/pending');
  }

  /**
   * Get recent transactions
   */
  async getRecentTransactions(limit: number = 50): Promise<Transaction[]> {
    return this.apiClient.get<Transaction[]>('/transaction/recent', {
      params: { limit }
    });
  }

  /**
   * Verify ZKP proof
   */
  async verifyZKPProof(proofData: string): Promise<boolean> {
    const data = { proof_data: proofData };
    const response = await this.apiClient.post<{ valid: boolean }>('/zkp/verify', data);
    return response.valid;
  }

  /**
   * Get transaction statistics
   */
  async getTransactionStats(): Promise<any> {
    return this.apiClient.get('/transaction/stats');
  }
}
