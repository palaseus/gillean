/**
 * Contract Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { ContractInfo, ContractCall, ContractDeployment } from './types';

/**
 * Manages smart contract operations for the Gillean blockchain
 */
export class ContractManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Deploy a smart contract
   */
  async deployContract(deployment: ContractDeployment): Promise<ContractInfo> {
    return this.apiClient.post<ContractInfo>('/contract/deploy', deployment);
  }

  /**
   * Call a smart contract method
   */
  async callContract(call: ContractCall): Promise<any> {
    return this.apiClient.post('/contract/call', call);
  }

  /**
   * Get contract information
   */
  async getContract(address: string): Promise<ContractInfo> {
    return this.apiClient.get<ContractInfo>(`/contract/${address}`);
  }

  /**
   * Get all contracts
   */
  async getAllContracts(): Promise<ContractInfo[]> {
    return this.apiClient.get<ContractInfo[]>('/contract/all');
  }

  /**
   * Compile contract code
   */
  async compileContract(code: string, name: string): Promise<any> {
    const data = { code, name };
    return this.apiClient.post('/contract/compile', data);
  }

  /**
   * Get contract templates
   */
  async getContractTemplates(): Promise<any[]> {
    return this.apiClient.get('/contract/templates');
  }
}
