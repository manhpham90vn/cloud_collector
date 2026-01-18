// Generic parallel execution utilities
//
// This module provides reusable parallel execution patterns that can be used
// in any Rust project requiring concurrent async operations.

use futures::stream::{self, StreamExt};
use std::future::Future;

/// Execute multiple async operations in parallel with a concurrency limit
///
/// This is a generic parallel execution function that can be used for any
/// async operations. Results are returned in arbitrary order.
///
/// # Arguments
/// * `items` - Vector of items to process
/// * `concurrency` - Maximum number of concurrent operations
/// * `executor` - Async function to execute for each item
///
/// # Returns
/// Vector of results in arbitrary order
///
/// # Example
/// ```ignore
/// let results = execute_parallel(
///     vec![1, 2, 3, 4, 5],
///     3,
///     |num| async move { num * 2 }
/// ).await;
/// ```
pub async fn execute_parallel<T, F, Fut, R>(
    items: Vec<T>,
    concurrency: usize,
    executor: F,
) -> Vec<R>
where
    T: Send,
    F: Fn(T) -> Fut,
    Fut: Future<Output = R> + Send,
    R: Send,
{
    stream::iter(items.into_iter().map(executor))
        .buffer_unordered(concurrency)
        .collect()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_parallel() {
        let items = vec![1, 2, 3, 4, 5];
        let results = execute_parallel(items, 3, |num| async move { num * 2 }).await;

        assert_eq!(results.len(), 5);
        assert!(results.contains(&2));
        assert!(results.contains(&10));
    }
}
