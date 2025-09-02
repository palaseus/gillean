#!/usr/bin/env python3
"""
Demo script for the Gillean Blockchain Test Suite

This script demonstrates basic usage of the blockchain test suite
without requiring the full Gillean blockchain to be running.
"""

import asyncio
import time
from blockchain_test_suite import BlockchainTestSuite, TestResult

async def demo_basic_usage():
    """Demonstrate basic usage of the test suite"""
    print("🚀 Gillean Blockchain Test Suite Demo")
    print("=" * 50)
    
    # Create a test suite with 2 nodes
    print("\n📋 Creating test suite with 2 nodes...")
    test_suite = BlockchainTestSuite(num_nodes=2, test_duration=60)
    
    print(f"✅ Created test suite with {test_suite.num_nodes} nodes")
    print(f"📊 Test duration: {test_suite.test_duration} seconds")
    
    # Show node configurations
    print("\n🔧 Node Configurations:")
    for i, node in enumerate(test_suite.nodes):
        config = node.config
        print(f"  Node {i}:")
        print(f"    Port: {config.port}")
        print(f"    Consensus: {config.consensus_type.upper()}")
        print(f"    Database: {config.db_path}")
        print(f"    Difficulty: {config.difficulty}")
        print(f"    Reward: {config.reward}")
        if config.consensus_type == "pos":
            print(f"    Min Stake: {config.min_stake}")
            print(f"    Max Validators: {config.max_validators}")
        print()
    
    # Demonstrate test result structure
    print("🧪 Test Result Structure Demo:")
    demo_result = TestResult(
        test_name="Demo Test",
        success=True,
        duration=1.23,
        details="This is a demonstration of the test result structure"
    )
    
    print(f"  Test Name: {demo_result.test_name}")
    print(f"  Success: {demo_result.success}")
    print(f"  Duration: {demo_result.duration}s")
    print(f"  Details: {demo_result.details}")
    
    # Show what tests would be run
    print("\n📋 Available Tests:")
    test_methods = [
        "Consensus Mechanism Test",
        "Immutability Test", 
        "Transaction Processing Test",
        "Network Synchronization Test",
        "Performance Metrics Test"
    ]
    
    for i, test_name in enumerate(test_methods, 1):
        print(f"  {i}. {test_name}")
    
    print("\n💡 To run actual tests:")
    print("  python3 blockchain_test_suite.py --nodes 2 --test-type quick")
    print("  python3 blockchain_test_suite.py --nodes 3 --test-type comprehensive")
    print("  python3 blockchain_test_suite.py --nodes 5 --test-type continuous --duration 600")

async def demo_custom_test():
    """Demonstrate how to create custom tests"""
    print("\n🔧 Custom Test Creation Demo:")
    print("-" * 30)
    
    class CustomTestSuite(BlockchainTestSuite):
        async def test_custom_scenario(self) -> TestResult:
            """Custom blockchain test scenario"""
            start_time = time.time()
            test_name = "Custom Scenario Test"
            
            try:
                # Simulate some test logic
                await asyncio.sleep(0.1)  # Simulate work
                
                # Custom success criteria
                success = True  # In real tests, this would be based on actual results
                
                duration = time.time() - start_time
                return TestResult(
                    test_name=test_name,
                    success=success,
                    duration=duration,
                    details="Custom scenario completed successfully"
                )
                
            except Exception as e:
                duration = time.time() - start_time
                return TestResult(
                    test_name=test_name,
                    success=False,
                    duration=duration,
                    details="Custom scenario test failed",
                    error=str(e)
                )
    
    # Create custom test suite
    custom_suite = CustomTestSuite(num_nodes=1, test_duration=30)
    
    # Run custom test
    print("Running custom test...")
    result = await custom_suite.test_custom_scenario()
    
    print(f"✅ Custom test completed:")
    print(f"  Name: {result.test_name}")
    print(f"  Success: {result.success}")
    print(f"  Duration: {result.duration:.3f}s")
    print(f"  Details: {result.details}")

async def demo_report_generation():
    """Demonstrate report generation"""
    print("\n📊 Report Generation Demo:")
    print("-" * 30)
    
    # Create a test suite with sample results
    test_suite = BlockchainTestSuite(num_nodes=2, test_duration=120)
    
    # Add some sample test results
    test_suite.test_results = [
        TestResult(
            test_name="Sample Test 1",
            success=True,
            duration=2.5,
            details="This is a sample test result"
        ),
        TestResult(
            test_name="Sample Test 2", 
            success=False,
            duration=1.8,
            details="This test failed",
            error="Sample error message"
        )
    ]
    
    # Generate and display report
    report = test_suite.generate_report()
    print("Generated Report:")
    print(report)

async def main():
    """Main demo function"""
    try:
        await demo_basic_usage()
        await demo_custom_test()
        await demo_report_generation()
        
        print("\n🎉 Demo completed successfully!")
        print("\n📚 For more information, see PYTHON_TEST_SUITE_README.md")
        print("🔧 To run actual tests, ensure Gillean blockchain is built and ready")
        
    except Exception as e:
        print(f"❌ Demo failed: {e}")
        print("This is expected if the Gillean blockchain is not running")

if __name__ == "__main__":
    asyncio.run(main())
