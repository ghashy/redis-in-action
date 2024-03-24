use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::HashesInterface;

async fn add_to_cart(
    client: &RedisClient,
    session_token: &str,
    item: &str,
    count: u64,
) -> Result<(), RedisError> {
    let key = format!("cart:{}", session_token);
    if count <= 0 {
        client.hdel(key, item).await?;
    } else {
        client.hset(key, (item, count)).await?;
    }
    Ok(())
}
