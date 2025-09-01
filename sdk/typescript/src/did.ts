/**
 * DID Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { DIDDocument, DIDVerificationResult } from './types';

/**
 * Manages Decentralized Identity operations for the Gillean blockchain
 */
export class DIDManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Create a new DID
   */
  async createDID(controller?: string, serviceEndpoints?: any[]): Promise<DIDDocument> {
    const data = {
      controller,
      service_endpoints: serviceEndpoints || []
    };

    return this.apiClient.post<DIDDocument>('/did/create', data);
  }

  /**
   * Get DID document
   */
  async getDIDDocument(did: string): Promise<DIDDocument> {
    return this.apiClient.get<DIDDocument>(`/did/${did}`);
  }

  /**
   * Link DID to wallet
   */
  async linkDIDToWallet(did: string, walletAddress: string): Promise<void> {
    const data = { wallet_address: walletAddress };
    await this.apiClient.post(`/did/${did}/link`, data);
  }

  /**
   * Get DID for wallet
   */
  async getDIDForWallet(walletAddress: string): Promise<string | null> {
    try {
      const response = await this.apiClient.get<{ did: string }>(`/did/wallet/${walletAddress}`);
      return response.did;
    } catch (error) {
      return null;
    }
  }

  /**
   * Verify DID signature
   */
  async verifyDIDSignature(did: string, message: string, signature: string): Promise<DIDVerificationResult> {
    const data = {
      message,
      signature
    };

    return this.apiClient.post<DIDVerificationResult>(`/did/${did}/verify`, data);
  }

  /**
   * Get all DIDs
   */
  async getAllDIDs(): Promise<string[]> {
    return this.apiClient.get<string[]>('/did/all');
  }

  /**
   * Get DID statistics
   */
  async getDIDStats(): Promise<any> {
    return this.apiClient.get('/did/stats');
  }
}
