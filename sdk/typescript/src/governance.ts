/**
 * Governance Manager for Gillean Blockchain TypeScript SDK
 */

import { APIClient } from './client';
import { GovernanceProposal, Vote, ProposalType, VoteChoice } from './types';

/**
 * Manages governance operations for the Gillean blockchain
 */
export class GovernanceManager {
  private apiClient: APIClient;

  constructor(apiClient: APIClient) {
    this.apiClient = apiClient;
  }

  /**
   * Create a governance proposal
   */
  async createProposal(
    proposer: string,
    title: string,
    description: string,
    proposalType: ProposalType,
    votingPeriod: number,
    quorum: number,
    contractCode?: string,
    parameters?: Record<string, string>
  ): Promise<GovernanceProposal> {
    const data = {
      proposer,
      title,
      description,
      proposal_type: proposalType,
      voting_period: votingPeriod,
      quorum,
      contract_code: contractCode,
      parameters: parameters || {}
    };

    return this.apiClient.post<GovernanceProposal>('/governance/propose', data);
  }

  /**
   * Vote on a proposal
   */
  async voteOnProposal(
    proposalId: string,
    voter: string,
    vote: VoteChoice,
    stakeAmount: number
  ): Promise<void> {
    const data = {
      proposal_id: proposalId,
      voter,
      vote,
      stake_amount: stakeAmount
    };

    await this.apiClient.post('/governance/vote', data);
  }

  /**
   * Execute a passed proposal
   */
  async executeProposal(proposalId: string): Promise<void> {
    await this.apiClient.post(`/governance/proposal/${proposalId}/execute`);
  }

  /**
   * Get proposal by ID
   */
  async getProposal(proposalId: string): Promise<GovernanceProposal> {
    return this.apiClient.get<GovernanceProposal>(`/governance/proposal/${proposalId}`);
  }

  /**
   * Get all proposals
   */
  async getAllProposals(): Promise<GovernanceProposal[]> {
    return this.apiClient.get<GovernanceProposal[]>('/governance/proposals');
  }

  /**
   * Get votes for a proposal
   */
  async getProposalVotes(proposalId: string): Promise<Vote[]> {
    return this.apiClient.get<Vote[]>(`/governance/proposal/${proposalId}/votes`);
  }

  /**
   * Get governance statistics
   */
  async getGovernanceStats(): Promise<any> {
    return this.apiClient.get('/governance/stats');
  }
}
