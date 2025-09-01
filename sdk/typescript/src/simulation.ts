/**
 * Simulation Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { SimulationConfig, SimulationResult, SimulationState } from './types';

/**
 * Manages simulation operations for the Gillean blockchain
 */
export class SimulationManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Run a blockchain simulation
   */
  async runSimulation(config: SimulationConfig): Promise<SimulationResult> {
    return this.apiClient.post<SimulationResult>('/simulation/run', config);
  }

  /**
   * Get simulation progress
   */
  async getSimulationProgress(simulationId: string): Promise<number> {
    const response = await this.apiClient.get<{ progress: number }>(`/simulation/${simulationId}/progress`);
    return response.progress;
  }

  /**
   * Get current simulation state
   */
  async getSimulationState(simulationId: string): Promise<SimulationState> {
    return this.apiClient.get<SimulationState>(`/simulation/${simulationId}/state`);
  }

  /**
   * Stop a running simulation
   */
  async stopSimulation(simulationId: string): Promise<void> {
    await this.apiClient.post(`/simulation/${simulationId}/stop`);
  }

  /**
   * Get simulation results
   */
  async getSimulationResults(simulationId: string): Promise<SimulationResult> {
    return this.apiClient.get<SimulationResult>(`/simulation/${simulationId}/results`);
  }

  /**
   * Get all simulations
   */
  async getAllSimulations(): Promise<any[]> {
    return this.apiClient.get('/simulation/all');
  }
}
