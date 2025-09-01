/**
 * Wallet Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { WalletInfo, WalletCreationOptions, GilleanError } from './types';

/**
 * Manages wallet operations for the Gillean blockchain
 */
export class WalletManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Create a new wallet
   */
  async createWallet(options: WalletCreationOptions = {}): Promise<WalletInfo> {
    const data = {
      name: options.name || `Wallet_${Date.now()}`,
      password: options.password || 'default_password',
      encryption_enabled: options.encryptionEnabled !== false
    };

    return this.apiClient.post<WalletInfo>('/wallet/create', data);
  }

  /**
   * Get wallet information
   */
  async getWallet(address: string): Promise<WalletInfo> {
    return this.apiClient.get<WalletInfo>(`/wallet/${address}`);
  }

  /**
   * Get all wallets
   */
  async getAllWallets(): Promise<WalletInfo[]> {
    return this.apiClient.get<WalletInfo[]>('/wallet/all');
  }

  /**
   * Get wallet balance
   */
  async getBalance(address: string): Promise<number> {
    const response = await this.apiClient.get<{ balance: number }>(`/wallet/${address}/balance`);
    return response.balance;
  }

  /**
   * Import wallet from private key
   */
  async importWallet(privateKey: string, name?: string): Promise<WalletInfo> {
    const data = {
      private_key: privateKey,
      name: name || `Imported_Wallet_${Date.now()}`
    };

    return this.apiClient.post<WalletInfo>('/wallet/import', data);
  }

  /**
   * Export wallet private key
   */
  async exportPrivateKey(address: string, password: string): Promise<string> {
    const data = { password };
    const response = await this.apiClient.post<{ private_key: string }>(`/wallet/${address}/export`, data);
    return response.private_key;
  }

  /**
   * Delete wallet
   */
  async deleteWallet(address: string, password: string): Promise<void> {
    const data = { password };
    await this.apiClient.delete(`/wallet/${address}`, { data });
  }

  /**
   * Update wallet name
   */
  async updateWalletName(address: string, newName: string): Promise<WalletInfo> {
    const data = { name: newName };
    return this.apiClient.put<WalletInfo>(`/wallet/${address}/name`, data);
  }

  /**
   * Change wallet password
   */
  async changePassword(address: string, oldPassword: string, newPassword: string): Promise<void> {
    const data = {
      old_password: oldPassword,
      new_password: newPassword
    };
    await this.apiClient.put(`/wallet/${address}/password`, data);
  }

  /**
   * Backup wallet
   */
  async backupWallet(address: string, password: string): Promise<string> {
    const data = { password };
    const response = await this.apiClient.post<{ backup_data: string }>(`/wallet/${address}/backup`, data);
    return response.backup_data;
  }

  /**
   * Restore wallet from backup
   */
  async restoreWallet(backupData: string, password: string): Promise<WalletInfo> {
    const data = {
      backup_data: backupData,
      password
    };
    return this.apiClient.post<WalletInfo>('/wallet/restore', data);
  }

  /**
   * Get wallet transaction history
   */
  async getTransactionHistory(address: string, limit: number = 50, offset: number = 0): Promise<any[]> {
    return this.apiClient.get<any[]>(`/wallet/${address}/transactions`, {
      params: { limit, offset }
    });
  }

  /**
   * Validate wallet address
   */
  async validateAddress(address: string): Promise<boolean> {
    try {
      await this.apiClient.get(`/wallet/validate/${address}`);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Get wallet statistics
   */
  async getWalletStats(address: string): Promise<any> {
    return this.apiClient.get(`/wallet/${address}/stats`);
  }
}
