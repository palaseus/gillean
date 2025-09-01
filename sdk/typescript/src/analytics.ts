/**
 * Analytics Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { BlockchainMetrics } from './types';

/**
 * Manages analytics and monitoring for the Gillean blockchain
 */
export class AnalyticsManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Get blockchain metrics
   */
  async getMetrics(): Promise<BlockchainMetrics> {
    return this.apiClient.get<BlockchainMetrics>('/metrics');
  }

  /**
   * Get blockchain status
   */
  async getStatus(): Promise<any> {
    return this.apiClient.get('/status');
  }

  /**
   * Get health check
   */
  async getHealth(): Promise<any> {
    return this.apiClient.get('/health');
  }
}
