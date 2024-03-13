use std::{collections::HashMap, time::SystemTime};

use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::ClientLike;
use fred::interfaces::HashesInterface;
use fred::interfaces::KeysInterface;
use fred::interfaces::SetsInterface;
use fred::interfaces::SortedSetsInterface;
use fred::types::RedisConfig;

// ───── Scheduling ───────────────────────────────────────────────────────── //

async fn schedule_row_cache(
    client: &RedisClient,
    row_id: i32,
    delay: f64,
) -> Result<(), RedisError> {
    let time = get_sys_time_in_secs() as f64;
    let () = client
        .zadd("delay:", None, None, false, false, vec![(delay, row_id)])
        .await?;
    let () = client
        .zadd("schedule:", None, None, false, false, vec![(time, row_id)])
        .await?;
    Ok(())
}

// ───── Caching ──────────────────────────────────────────────────────────── //

struct Db;
type Json = String;

impl Db {
    fn get(row_id: i32) -> Json {
        String::new()
    }
}

/// Updating all db rows in a queue
async fn cache_rows(client: &RedisClient) -> Result<(), RedisError> {
    type RowId = i32;
    loop {
        let now = get_sys_time_in_secs() as f64;

        // Find the next row that should be cached (if any), including the timestamp, as a list of tuples with zero or one items.
        let next: Vec<(f64, RowId)> = client
            .zrange(
                "schedule:",
                0,
                0,
                None,
                false,
                None,
                /* withscores */ true,
            )
            .await?;
        if next.is_empty() || next[0].0 > now {
            // No rows can be cached now, so wait 50 milliseconds and try again.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let row_id = next[0].1;

        // Get the delay before the next schedule. If delay is set to 0, caching is disable for that row.
        let delay: f64 = client.zscore("delay:", row_id).await?;
        if delay <= 0.0 {
            // The item shouldn’t be cached anymore; remove it from the cache.
            let () = client.zrem("delay:", row_id).await?;
            let () = client.zrem("schedule:", row_id).await?;
            let () = client.del(format!("inv:{}", row_id)).await?;
            continue;
        }

        // Get the database row.
        let json = Db::get(row_id);
        // Update the schedule and set the cache value.
        let () = client
            .zadd(
                "schedule:",
                None,
                None,
                false,
                false,
                vec![(now + delay, row_id)],
            )
            .await?;
        let () = client
            .set(format!("inv:{}", row_id), json, None, None, false)
            .await?;
    }
}
