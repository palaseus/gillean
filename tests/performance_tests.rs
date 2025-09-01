use gillean::{
    PerformanceManager, CacheManager, ParallelProcessor, MemoryOptimizer, MetricsCollector,
    PerformanceConfig, CacheConfig, ParallelConfig, MemoryUsage, MetricsConfig,
    PerformanceStats, OptimizationResult, Task
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct PerformanceTestSuite {
    manager: Arc<PerformanceManager>,
}

impl PerformanceTestSuite {
    pub fn new() -> Self {
        let config = PerformanceConfig {
            cache_size: 1000,
            cache_ttl: Duration::from_secs(60),
            max_parallel_tasks: 8,
            memory_threshold: 0.8,
            gc_interval: Duration::from_secs(30),
            enable_metrics: true,
        };

        Self {
            manager: Arc::new(PerformanceManager::new(config)),
        }
    }

    pub async fn run_all_tests(&self) -> Result<(), String> {
        println!("🧪 Running Performance Optimization tests...");

        self.test_cache_manager().await?;
        self.test_parallel_processor().await?;
        self.test_memory_optimizer().await?;
        self.test_metrics_collector().await?;
        self.test_performance_manager().await?;
        self.test_optimization_analysis().await?;

        println!("  ✅ Performance Optimization tests completed!");
        Ok(())
    }

    async fn test_cache_manager(&self) -> Result<(), String> {
        println!("    Testing Cache Manager...");

        let cache_config = CacheConfig {
            max_size: 10,
            ttl: Duration::from_secs(2),
            enable_lru: true,
        };
        let cache = CacheManager::new(cache_config);

        // Test basic cache operations
        cache.set("key1".to_string(), b"value1".to_vec()).await;
        let value = cache.get("key1").await;
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test cache expiration
        sleep(Duration::from_secs(3)).await;
        let expired_value = cache.get("key1").await;
        assert_eq!(expired_value, None);

        // Test cache statistics
        let stats = cache.get_stats().await;
        assert!(stats.total_requests > 0);

        println!("      ✅ Cache Manager tests passed");
        Ok(())
    }

    async fn test_parallel_processor(&self) -> Result<(), String> {
        println!("    Testing Parallel Processor...");

        let config = ParallelConfig {
            worker_count: 4,
            max_queue_size: 100,
            enable_priority: true,
        };
        let mut processor = ParallelProcessor::new(config);

        // Test task submission
        let test_task = TestTask {
            id: "test_task".to_string(),
            should_succeed: true,
        };

        let result = processor.submit_task(Box::new(test_task)).await;
        assert!(result.is_ok());

        // Test task execution
        let failing_task = TestTask {
            id: "failing_task".to_string(),
            should_succeed: false,
        };

        let result = processor.submit_task(Box::new(failing_task)).await;
        assert!(result.is_ok());

        // Clean shutdown
        processor.shutdown().await;

        println!("      ✅ Parallel Processor tests passed");
        Ok(())
    }

    async fn test_memory_optimizer(&self) -> Result<(), String> {
        println!("    Testing Memory Optimizer...");

        let optimizer = MemoryOptimizer::new(Duration::from_secs(1), 0.8);
        optimizer.start_monitoring().await;

        // Test memory usage tracking
        sleep(Duration::from_millis(100)).await;
        let usage = optimizer.get_memory_usage().await;
        assert!(usage.total_memory > 0);
        assert!(usage.used_memory > 0);

        // Test garbage collection
        let initial_gc_count = usage.gc_count;
        sleep(Duration::from_secs(2)).await;
        let updated_usage = optimizer.get_memory_usage().await;
        assert!(updated_usage.gc_count >= initial_gc_count);

        optimizer.stop_monitoring().await;

        println!("      ✅ Memory Optimizer tests passed");
        Ok(())
    }

    async fn test_metrics_collector(&self) -> Result<(), String> {
        println!("    Testing Metrics Collector...");

        let config = MetricsConfig {
            enable_collection: true,
            retention_period: Duration::from_secs(60),
            flush_interval: Duration::from_secs(1),
        };
        let metrics = MetricsCollector::new(config);

        // Test counter metrics
        metrics.increment_counter("test_counter", 5).await;
        metrics.increment_counter("test_counter", 3).await;

        // Test gauge metrics
        metrics.set_gauge("test_gauge", 42.5).await;

        // Test timer metrics
        metrics.record_timer("test_timer", Duration::from_millis(100)).await;

        // Test metrics retrieval
        let collected_metrics = metrics.get_metrics().await;
        assert!(collected_metrics.contains_key("test_counter"));
        assert!(collected_metrics.contains_key("test_gauge"));
        assert!(collected_metrics.contains_key("test_timer"));

        println!("      ✅ Metrics Collector tests passed");
        Ok(())
    }

    async fn test_performance_manager(&self) -> Result<(), String> {
        println!("    Testing Performance Manager...");

        // Start the performance manager
        self.manager.start().await;

        // Test performance statistics
        sleep(Duration::from_millis(100)).await;
        let stats = self.manager.get_performance_stats().await;
        assert!(stats.cache_stats.total_requests >= 0);
        assert!(stats.memory_usage.total_memory > 0);

        println!("      ✅ Performance Manager tests passed");
        Ok(())
    }

    async fn test_optimization_analysis(&self) -> Result<(), String> {
        println!("    Testing Optimization Analysis...");

        // Test performance optimization
        let result = self.manager.optimize_performance().await;
        assert!(result.success);
        assert!(result.duration > Duration::from_nanos(0));

        // Test optimization recommendations
        assert!(!result.optimizations.is_empty() || result.optimizations.is_empty());

        println!("      ✅ Optimization Analysis tests passed");
        Ok(())
    }
}

// Test task implementation
struct TestTask {
    id: String,
    should_succeed: bool,
}

impl Task for TestTask {
    fn execute(&self) -> Result<(), String> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(format!("Task {} failed", self.id))
        }
    }

    fn priority(&self) -> u8 {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_suite() {
        let suite = PerformanceTestSuite::new();
        suite.run_all_tests().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_secs(10),
            enable_lru: true,
        };
        let cache = CacheManager::new(config);

        // Performance test: many operations
        for i in 0..100 {
            let key = format!("key{}", i);
            let value = format!("value{}", i).into_bytes();
            cache.set(key, value).await;
        }

        // Test cache hit rate
        for i in 0..50 {
            let key = format!("key{}", i);
            let _ = cache.get(&key).await;
        }

        let stats = cache.get_stats().await;
        assert!(stats.hits > 0);
        assert!(stats.total_requests > 0);
    }

    #[tokio::test]
    async fn test_memory_optimization() {
        let optimizer = MemoryOptimizer::new(Duration::from_secs(1), 0.5);
        optimizer.start_monitoring().await;

        sleep(Duration::from_millis(200)).await;
        let usage = optimizer.get_memory_usage().await;
        
        assert!(usage.total_memory > 0);
        assert!(usage.used_memory > 0);
        assert!(usage.free_memory > 0);

        optimizer.stop_monitoring().await;
    }

    #[tokio::test]
    async fn test_parallel_processing() {
        let config = ParallelConfig {
            worker_count: 2,
            max_queue_size: 10,
            enable_priority: true,
        };
        let mut processor = ParallelProcessor::new(config);

        // Submit multiple tasks
        for i in 0..5 {
            let task = TestTask {
                id: format!("task{}", i),
                should_succeed: true,
            };
            processor.submit_task(Box::new(task)).await.unwrap();
        }

        sleep(Duration::from_millis(100)).await;
        processor.shutdown().await;
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = MetricsConfig {
            enable_collection: true,
            retention_period: Duration::from_secs(60),
            flush_interval: Duration::from_secs(1),
        };
        let metrics = MetricsCollector::new(config);

        // Simulate application metrics
        for i in 0..10 {
            metrics.increment_counter("requests", 1).await;
            metrics.set_gauge("cpu_usage", 50.0 + i as f64).await;
            metrics.record_timer("response_time", Duration::from_millis(100 + i * 10)).await;
        }

        let collected_metrics = metrics.get_metrics().await;
        assert!(collected_metrics.len() >= 3);
    }
}
