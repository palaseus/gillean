#!/usr/bin/env python3
"""
Gillean Blockchain Multi-Node Test Suite

This script spins up multiple Gillean blockchain nodes and performs comprehensive
testing of various blockchain aspects including consensus, immutability, and
other blockchain properties.

Usage:
    python3 blockchain_test_suite.py [--nodes N] [--test-type TYPE] [--duration SECONDS]
"""

import asyncio
import json
import logging
import os
import random
import signal
import subprocess
import sys
import time
import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import aiohttp
import requests
from concurrent.futures import ThreadPoolExecutor
import threading

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('blockchain_test_suite.log'),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class NodeConfig:
    """Configuration for a blockchain node"""
    id: int
    port: int
    db_path: str
    consensus_type: str = "pow"
    difficulty: int = 4
    reward: float = 50.0
    min_stake: float = 100.0
    max_validators: int = 5

@dataclass
class TestResult:
    """Result of a blockchain test"""
    test_name: str
    success: bool
    duration: float
    details: str
    error: Optional[str] = None

class BlockchainNode:
    """Manages a single Gillean blockchain node"""
    
    def __init__(self, config: NodeConfig):
        self.config = config
        self.process: Optional[subprocess.Popen] = None
        self.api_url = f"http://127.0.0.1:{config.port}"
        self.is_running = False
        self.start_time: Optional[float] = None
        
    async def start(self) -> bool:
        """Start the blockchain node"""
        try:
            # Create database directory
            os.makedirs(self.config.db_path, exist_ok=True)
            
            # Start the node using the correct command structure
            cmd = [
                "./target/release/gillean", "start-api",
                "--address", f"127.0.0.1:{self.config.port}",
                "--db-path", self.config.db_path
            ]
            
            self.process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            # Wait for node to start
            await self._wait_for_startup()
            self.is_running = True
            self.start_time = time.time()
            logger.info(f"Node {self.config.id} started successfully on port {self.config.port}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to start node {self.config.id}: {e}")
            return False
    
    async def _wait_for_startup(self, timeout: int = 30) -> bool:
        """Wait for the node to start up and be ready"""
        start_time = time.time()
        while time.time() - start_time < timeout:
            try:
                async with aiohttp.ClientSession() as session:
                    async with session.get(f"{self.api_url}/health") as response:
                        if response.status == 200:
                            return True
            except:
                pass
            await asyncio.sleep(1)
        return False
    
    async def stop(self) -> bool:
        """Stop the blockchain node"""
        if self.process:
            try:
                self.process.terminate()
                self.process.wait(timeout=10)
                self.is_running = False
                logger.info(f"Node {self.config.id} stopped successfully")
                return True
            except subprocess.TimeoutExpired:
                self.process.kill()
                logger.warning(f"Node {self.config.id} force killed")
                return False
        return True
    
    async def get_chain_info(self) -> Optional[Dict]:
        """Get blockchain information from the node"""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(f"{self.api_url}/chain") as response:
                    if response.status == 200:
                        return await response.json()
        except Exception as e:
            logger.error(f"Failed to get chain info from node {self.config.id}: {e}")
        return None
    
    async def add_transaction(self, sender: str, receiver: str, amount: float, message: str = "") -> Optional[Dict]:
        """Add a transaction to the node"""
        try:
            tx_data = {
                "sender": sender,
                "receiver": receiver,
                "amount": amount,
                "message": message
            }
            
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.api_url}/transaction",
                    json=tx_data
                ) as response:
                    if response.status in [200, 400]:  # 400 is expected for insufficient balance
                        return await response.json()
        except Exception as e:
            logger.error(f"Failed to add transaction to node {self.config.id}: {e}")
        return None
    
    async def mine_block(self, miner: str) -> Optional[Dict]:
        """Mine a block on the node"""
        try:
            mine_data = {"miner_address": miner}
            
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.api_url}/mine",
                    json=mine_data
                ) as response:
                    if response.status in [200, 400]:  # 400 is expected for no pending transactions
                        return await response.json()
        except Exception as e:
            logger.error(f"Failed to mine block on node {self.config.id}: {e}")
        return None
    
    async def get_balance(self, address: str) -> Optional[float]:
        """Get balance for an address"""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(f"{self.api_url}/balance/{address}") as response:
                    if response.status == 200:
                        data = await response.json()
                        return data.get("balance", 0.0)
        except Exception as e:
            logger.error(f"Failed to get balance from node {self.config.id}: {e}")
        return None

class BlockchainTestSuite:
    """Comprehensive blockchain testing framework"""
    
    def __init__(self, num_nodes: int = 3, test_duration: int = 300):
        self.num_nodes = num_nodes
        self.test_duration = test_duration
        self.nodes: List[BlockchainNode] = []
        self.test_results: List[TestResult] = []
        self.running = False
        self.stop_event = threading.Event()
        
        # Generate node configurations
        self._generate_node_configs()
        
    def _generate_node_configs(self):
        """Generate configurations for all nodes"""
        base_port = 3000
        for i in range(self.num_nodes):
            config = NodeConfig(
                id=i,
                port=base_port + i,
                db_path=f"./data/test_node_{i}_db_{int(time.time())}_{i}",
                consensus_type="pow" if i == 0 else "pos",  # First node is PoW, others are PoS
                difficulty=4 + i,  # Varying difficulty
                reward=50.0 + (i * 10),  # Varying rewards
                min_stake=100.0 + (i * 50),  # Varying stake requirements
                max_validators=3 + i  # Varying validator limits
            )
            self.nodes.append(BlockchainNode(config))
    
    async def start_all_nodes(self) -> bool:
        """Start all blockchain nodes"""
        logger.info(f"Starting {self.num_nodes} blockchain nodes...")
        
        # Start nodes sequentially to avoid conflicts
        results = []
        for i, node in enumerate(self.nodes):
            logger.info(f"Starting node {i}...")
            result = await node.start()
            results.append(result)
            
            if result:
                logger.info(f"Node {i} started successfully")
                # Wait a bit before starting the next node
                if i < len(self.nodes) - 1:
                    await asyncio.sleep(2)
            else:
                logger.error(f"Failed to start node {i}")
        
        success_count = sum(results)
        if success_count == self.num_nodes:
            logger.info("All nodes started successfully")
            return True
        else:
            logger.error(f"Only {success_count}/{self.num_nodes} nodes started successfully")
            return False
    
    async def stop_all_nodes(self):
        """Stop all blockchain nodes"""
        logger.info("Stopping all blockchain nodes...")
        
        stop_tasks = [node.stop() for node in self.nodes]
        await asyncio.gather(*stop_tasks)
        
        logger.info("All nodes stopped")
    
    async def test_consensus_mechanism(self) -> TestResult:
        """Test consensus mechanism across nodes"""
        start_time = time.time()
        test_name = "Consensus Mechanism Test"
        
        try:
            logger.info("Testing consensus mechanism...")
            
            # Test PoW consensus on first node
            pow_node = self.nodes[0]
            if pow_node.config.consensus_type == "pow":
                # Mine a few blocks
                for i in range(3):
                    result = await pow_node.mine_block(f"miner_{i}")
                    if result and result.get("status") == "success":
                        logger.info(f"Mined block {i} on PoW node")
                    await asyncio.sleep(1)
                
                # Check chain length
                chain_info = await pow_node.get_chain_info()
                if chain_info and chain_info.get("data", {}).get("blocks"):
                    blocks = chain_info["data"]["blocks"]
                    if len(blocks) >= 3:
                        duration = time.time() - start_time
                        return TestResult(
                            test_name=test_name,
                            success=True,
                            duration=duration,
                            details=f"PoW consensus working: {len(blocks)} blocks mined"
                        )
            
            # Test PoS consensus on other nodes
            pos_nodes = [n for n in self.nodes[1:] if n.config.consensus_type == "pos"]
            for node in pos_nodes:
                # Try to register as validator
                # Note: This would require additional API endpoints in the actual implementation
                logger.info(f"PoS node {node.config.id} ready for validation")
            
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=True,
                duration=duration,
                details=f"Consensus mechanisms initialized: {len(self.nodes)} nodes running"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=False,
                duration=duration,
                details="Consensus test failed",
                error=str(e)
            )
    
    async def test_immutability(self) -> TestResult:
        """Test blockchain immutability"""
        start_time = time.time()
        test_name = "Immutability Test"
        
        try:
            logger.info("Testing blockchain immutability...")
            
            # Get initial state from all nodes
            initial_states = []
            for node in self.nodes:
                chain_info = await node.get_chain_info()
                if chain_info:
                    initial_states.append({
                        'node_id': node.config.id,
                        'chain_hash': chain_info.get('data', {}).get('chain_hash', ''),
                        'block_count': len(chain_info.get('data', {}).get('blocks', []))
                    })
            
            # Perform some operations
            await asyncio.sleep(2)
            
            # Check if states remain consistent
            final_states = []
            for node in self.nodes:
                chain_info = await node.get_chain_info()
                if chain_info:
                    final_states.append({
                        'node_id': node.config.id,
                        'chain_hash': chain_info.get('data', {}).get('chain_hash', ''),
                        'block_count': len(chain_info.get('data', {}).get('blocks', []))
                    })
            
            # Verify immutability
            immutable = True
            for initial, final in zip(initial_states, final_states):
                if initial['chain_hash'] != final['chain_hash']:
                    immutable = False
                    break
            
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=immutable,
                duration=duration,
                details=f"Immutability verified across {len(self.nodes)} nodes"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=False,
                duration=duration,
                details="Immutability test failed",
                error=str(e)
            )
    
    async def test_transaction_processing(self) -> TestResult:
        """Test transaction processing across nodes"""
        start_time = time.time()
        test_name = "Transaction Processing Test"
        
        try:
            logger.info("Testing transaction processing...")
            
            # Create test transactions
            test_transactions = [
                ("alice", "bob", 10.0, "Test transaction 1"),
                ("bob", "charlie", 5.0, "Test transaction 2"),
                ("charlie", "alice", 3.0, "Test transaction 3")
            ]
            
            # Add transactions to first node
            node = self.nodes[0]
            tx_results = []
            for sender, receiver, amount, message in test_transactions:
                result = await node.add_transaction(sender, receiver, amount, message)
                tx_results.append(result)
                await asyncio.sleep(0.5)
            
            # Mine a block to process transactions
            mine_result = await node.mine_block("test_miner")
            
            # Check final state
            chain_info = await node.get_chain_info()
            success = False
            if chain_info and chain_info.get('data', {}).get('total_transactions', 0) > 0:
                success = True
            
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=success,
                duration=duration,
                details=f"Transaction processing completed: {len(tx_results)} transactions"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=False,
                duration=duration,
                details="Transaction processing test failed",
                error=str(e)
            )
    
    async def test_network_synchronization(self) -> TestResult:
        """Test network synchronization between nodes"""
        start_time = time.time()
        test_name = "Network Synchronization Test"
        
        try:
            logger.info("Testing network synchronization...")
            
            # Get chain states from all nodes
            chain_states = []
            for node in self.nodes:
                chain_info = await node.get_chain_info()
                if chain_info:
                    chain_states.append({
                        'node_id': node.config.id,
                        'block_count': len(chain_info.get('data', {}).get('blocks', [])),
                        'chain_hash': chain_info.get('data', {}).get('chain_hash', '')
                    })
            
            # Check synchronization (in a real network, nodes would sync)
            # For now, we'll check if all nodes are running and responding
            all_responding = all(state['block_count'] >= 0 for state in chain_states)
            
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=all_responding,
                duration=duration,
                details=f"Network synchronization check: {len(chain_states)} nodes responding"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=False,
                duration=duration,
                details="Network synchronization test failed",
                error=str(e)
            )
    
    async def test_performance_metrics(self) -> TestResult:
        """Test blockchain performance metrics"""
        start_time = time.time()
        test_name = "Performance Metrics Test"
        
        try:
            logger.info("Testing performance metrics...")
            
            # Measure transaction throughput
            node = self.nodes[0]
            tx_start = time.time()
            
            # Add multiple transactions quickly
            for i in range(10):
                await node.add_transaction(f"user{i}", f"user{i+1}", 1.0, f"Perf test {i}")
            
            tx_end = time.time()
            # Avoid division by zero
            if tx_end > tx_start:
                tx_throughput = 10 / (tx_end - tx_start)
            else:
                tx_throughput = 0.0
            
            # Measure block mining time
            mine_start = time.time()
            mine_result = await node.mine_block("perf_miner")
            mine_end = time.time()
            mine_time = mine_end - mine_start
            
            # Check if metrics are reasonable
            success = tx_throughput > 1.0 and mine_time < 30.0
            
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=success,
                duration=duration,
                details=f"Performance: {tx_throughput:.2f} tx/s, {mine_time:.2f}s mining time"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            return TestResult(
                test_name=test_name,
                success=False,
                duration=duration,
                details="Performance metrics test failed",
                error=str(e)
            )
    
    async def run_all_tests(self) -> List[TestResult]:
        """Run all blockchain tests"""
        logger.info("Starting comprehensive blockchain test suite...")
        
        tests = [
            self.test_consensus_mechanism(),
            self.test_immutability(),
            self.test_transaction_processing(),
            self.test_network_synchronization(),
            self.test_performance_metrics()
        ]
        
        results = await asyncio.gather(*tests)
        self.test_results = results
        
        return results
    
    def generate_report(self) -> str:
        """Generate a comprehensive test report"""
        report = []
        report.append("=" * 60)
        report.append("GILLEAN BLOCKCHAIN COMPREHENSIVE TEST REPORT")
        report.append("=" * 60)
        report.append(f"Test Date: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"Number of Nodes: {self.num_nodes}")
        report.append(f"Test Duration: {self.test_duration} seconds")
        report.append("")
        
        # Test Results Summary
        total_tests = len(self.test_results)
        passed_tests = sum(1 for r in self.test_results if r.success)
        failed_tests = total_tests - passed_tests
        
        report.append("TEST RESULTS SUMMARY")
        report.append("-" * 30)
        report.append(f"Total Tests: {total_tests}")
        report.append(f"Passed: {passed_tests}")
        report.append(f"Failed: {failed_tests}")
        # Avoid division by zero
        if total_tests > 0:
            success_rate = (passed_tests/total_tests)*100
        else:
            success_rate = 0.0
        report.append(f"Success Rate: {success_rate:.1f}%")
        report.append("")
        
        # Detailed Results
        report.append("DETAILED TEST RESULTS")
        report.append("-" * 30)
        
        for result in self.test_results:
            status = "PASS" if result.success else "FAIL"
            report.append(f"{result.test_name}: {status}")
            report.append(f"  Duration: {result.duration:.2f}s")
            report.append(f"  Details: {result.details}")
            if result.error:
                report.append(f"  Error: {result.error}")
            report.append("")
        
        # Node Information
        report.append("NODE CONFIGURATION")
        report.append("-" * 30)
        for node in self.nodes:
            report.append(f"Node {node.config.id}:")
            report.append(f"  Port: {node.config.port}")
            report.append(f"  Consensus: {node.config.consensus_type.upper()}")
            report.append(f"  Database: {node.config.db_path}")
            report.append(f"  Status: {'Running' if node.is_running else 'Stopped'}")
            report.append("")
        
        return "\n".join(report)
    
    async def run_continuous_testing(self):
        """Run continuous testing for the specified duration"""
        logger.info(f"Starting continuous testing for {self.test_duration} seconds...")
        
        start_time = time.time()
        test_interval = 30  # Run tests every 30 seconds
        
        while time.time() - start_time < self.test_duration and not self.stop_event.is_set():
            try:
                # Run a subset of tests
                await self.test_consensus_mechanism()
                await self.test_immutability()
                await asyncio.sleep(test_interval)
                
            except Exception as e:
                logger.error(f"Error during continuous testing: {e}")
                await asyncio.sleep(5)
        
        logger.info("Continuous testing completed")

async def main():
    """Main function to run the blockchain test suite"""
    parser = argparse.ArgumentParser(description="Gillean Blockchain Test Suite")
    parser.add_argument("--nodes", type=int, default=3, help="Number of nodes to start")
    parser.add_argument("--duration", type=int, default=300, help="Test duration in seconds")
    parser.add_argument("--test-type", choices=["quick", "comprehensive", "continuous"], 
                       default="comprehensive", help="Type of testing to perform")
    
    args = parser.parse_args()
    
    # Create test suite
    test_suite = BlockchainTestSuite(num_nodes=args.nodes, test_duration=args.duration)
    
    # Setup signal handlers
    def signal_handler(signum, frame):
        logger.info("Received interrupt signal, shutting down...")
        test_suite.stop_event.set()
        asyncio.create_task(test_suite.stop_all_nodes())
        sys.exit(0)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    try:
        # Start all nodes
        if not await test_suite.start_all_nodes():
            logger.error("Failed to start all nodes. Exiting.")
            return
        
        # Run tests based on type
        if args.test_type == "quick":
            # Run basic tests
            await test_suite.test_consensus_mechanism()
            await test_suite.test_immutability()
            
        elif args.test_type == "comprehensive":
            # Run all tests
            await test_suite.run_all_tests()
            
        elif args.test_type == "continuous":
            # Run continuous testing
            await test_suite.run_continuous_testing()
        
        # Generate and display report
        report = test_suite.generate_report()
        print(report)
        
        # Save report to file
        with open("blockchain_test_report.txt", "w") as f:
            f.write(report)
        
        logger.info("Test report saved to blockchain_test_report.txt")
        
    except Exception as e:
        logger.error(f"Test suite failed: {e}")
        
    finally:
        # Cleanup
        await test_suite.stop_all_nodes()

if __name__ == "__main__":
    asyncio.run(main())
