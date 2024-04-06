use std::collections::HashMap;

use crate::get_sys_time_in_secs;
use fred::{
    clients::{RedisClient, Transaction},
    error::RedisError,
    interfaces::{
        HashesInterface, KeysInterface, ListInterface, SortedSetsInterface,
        TransactionInterface,
    },
};
use time::OffsetDateTime;
use time::Time;

/// The precision of the counters in seconds: 1 second,
/// 5 seconds, 1 minute, 5 minutes, 1 hour, 5 hours, 1 day.
const PRECISION: [i64; 7] = [1, 5, 60, 300, 3600, 18000, 86400];

/// This function updates all counters with all precisions
async fn update_counter(
    client: &RedisClient,
    name: &str,
    count: i64,
    now: Option<OffsetDateTime>,
) -> Result<(), RedisError> {
    // Get the current time to know which time slice to increment
    let now = now.unwrap_or(OffsetDateTime::now_utc()).unix_timestamp();
    let pipe = client.pipeline();
    // Add entries for all precisions that we record
    for prec in PRECISION {
        // Get the start of the current time slice, using INTEGER operations
        let pnow = (now / prec) * prec;
        // Create the named hash where this data will be stored
        let hash = format!("{prec}:{name}");
        // Record a reference to the counters into a ZSET with the score 0
        // so we can clean up after ourselves
        pipe.zadd("known:", None, None, false, false, (0 as f64, &hash))
            .await?;
        // Update the counter for the given name and time precision
        pipe.hincrby(format!("count:{hash}"), pnow, count).await?;
    }
    pipe.all().await?;
    Ok(())
}

/// Get all counters with all precisions
async fn get_counter(
    client: &RedisClient,
    name: &str,
    prec: i64,
) -> Result<Vec<(i64, i64)>, RedisError> {
    // Get the name of the key where we’ll be storing counter data
    let hash = format!("{prec}:{name}");
    // Fetch the counter data from Redis
    let mut data: Vec<(i64, i64)> =
        client.hgetall(format!("count:{hash}")).await?;
    // Sort our data so that older samples are first
    data.sort();
    Ok(data)
}
