use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::SortedSetsInterface;

/// Every 5 minutes, let’s go ahead and delete any item that isn’t in
/// the top 20,000 items, and rescale the view counts
/// to be half has much as they were before.
async fn rescale_viewed(client: &RedisClient) -> Result<(), RedisError> {
    loop {
        client.zremrangebyrank("viewed:", 20000, -1).await?;
        client.zinterstore("viewed:", "viewed:", 0.5, None).await?;
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}
