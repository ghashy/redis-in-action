use std::time::{Duration, SystemTime};

use fred::{
    clients::RedisClient,
    error::RedisError,
    interfaces::{
        ClientLike, HashesInterface, KeysInterface, SetsInterface,
        SortedSetsInterface, TransactionInterface,
    },
    types::{RedisConfig, RedisValue},
};

#[tokio::main]
async fn main() {
    let client = init_redis_client().await;
    let result = list_item(&client, "item1", 17, 100.0).await.unwrap();
    dbg!(result);
}

/// Move item from user's inventory to the market for selling.
async fn list_item(
    client: &RedisClient,
    item_name: &str,
    seller_id: i32,
    price: f64,
) -> Result<Option<bool>, RedisError> {
    let inventory = format!("inventory:{}", seller_id);
    let item = format!("{}.{}", item_name, seller_id);
    let end = now() + 5;

    while now() < end {
        // We use client here instead of pipeline because client itself
        // perform operations immediately, and it can watch for key.
        client.watch(&inventory).await?;
        // Immediately check that item is in the user's inventory
        if !client.sismember(&inventory, item_name).await? {
            client.unwatch().await?;
            return Ok(None);
        }
        // Enter the transaction, remember that we are still watching for inventory:seller_id!
        let multi = client.multi();
        multi
            .zadd("market:", None, None, false, false, (price, &item))
            .await?;
        multi.srem(&inventory, item_name).await?;
        // If we got nil from redis, it means that someone interfered to our inventory:seller_id,
        // start from beginning
        if multi.exec::<RedisValue>(false).await?.is_null() {
            continue;
        }
        return Ok(Some(true));
    }

    Ok(Some(false))
}

/// Exchange funds to item using `market:` ZSet
async fn purchase_item(
    client: &RedisClient,
    buyer_id: i32,
    item_name: &str,
    seller_id: i32,
    lprice: f64,
) -> Result<Option<bool>, RedisError> {
    // Hash with buyer data key
    let buyer = format!("users:{}", buyer_id);
    // Hash with seller data key
    let seller = format!("users:{}", seller_id);
    // Item code in the market
    let item = format!("{}.{}", item_name, seller_id);
    // Buyer's inventory key
    let inventory = format!("inventory:{}", buyer_id);
    let end = now() + 10;

    while now() < end {
        client.watch(("market:", &buyer)).await?;
        let price: f64 = client.zscore("market:", &item).await?;
        let funds: f64 = client.hget(&buyer, "funds").await?;
        if price != lprice || price > funds {
            client.unwatch().await?;
            return Ok(None);
        }
        let multi = client.multi();
        // Move funds from buyer to seller
        multi.hincrby(&seller, "funds", price as i64).await?;
        multi.hincrby(&buyer, "funds", -price as i64).await?;
        // Move item from market to buyer's inventory
        multi.sadd(&inventory, item_name).await?;
        multi.zrem("market:", &item).await?;
        // Try to execute transaction
        if multi.exec::<RedisValue>(false).await?.is_null() {
            continue;
        }
        return Ok(Some(true));
    }
    Ok(None)
}

// ───── Helpers ──────────────────────────────────────────────────────────── //

// fn handle_watch_err<T>(res: Result<T, RedisError>) ->

/// Get system time in unix timestamp format
fn now() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

async fn init_redis_client() -> RedisClient {
    let config = RedisConfig::from_url_centralized(
        "redis://:ghashy@myredis.orb.local:6379",
    )
    .unwrap();
    let client = RedisClient::new(config, None, None, None);
    let _ = client.init().await.unwrap();
    client
}
