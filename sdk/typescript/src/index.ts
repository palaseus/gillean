/**
 * Gillean Blockchain TypeScript SDK v2.0.0
 * 
 * A comprehensive TypeScript SDK for interacting with the Gillean blockchain platform.
 * Provides functionality for wallet management, transactions, smart contracts,
 * zero-knowledge proofs, state channels, sharding, cross-chain operations,
 * decentralized identity, governance, and simulation.
 */

export * from './client';
export * from './wallet';
export * from './transactions';
export * from './contracts';
export * from './analytics';
export * from './ethereum';
export * from './did';
export * from './governance';
export * from './simulation';
export * from './types';
export * from './utils';

// Main SDK class
export { GilleanSDK } from './sdk';
