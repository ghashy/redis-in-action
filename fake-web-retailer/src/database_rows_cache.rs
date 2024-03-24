use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::KeysInterface;
use fred::interfaces::SortedSetsInterface;

use crate::get_sys_time_in_secs;

// ───── Scheduling ───────────────────────────────────────────────────────── //

/// This function adds row id in queue for caching.
///
/// * `row_id` - row id for cache.
/// * `delay` - delay between updating row in cache using db.
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

// ───── Pseudo-code types ────────────────────────────────────────────────── //

struct Db;
type Json = String;

impl Db {
    fn get(_row_id: i32) -> Json {
        String::new()
    }
}
// ───── Caching ──────────────────────────────────────────────────────────── //

/// Updating all db rows in a queue, using `schedule:` and `delay:` zsets.
/// Data is taken from json, under key `inv:{row_id}`. `inv` means `inventory`.
async fn cache_rows(client: &RedisClient) -> Result<(), RedisError> {
    type RowId = i32;
    type Timestamp = f64;
    loop {
        let now = get_sys_time_in_secs() as f64;

        // Find the next row that should be cached (if any),
        // including the timestamp, as a list of tuples with zero or one items
        let next: Vec<(Timestamp, RowId)> = client
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
            // No rows can be cached now, so wait 50 milliseconds and try again
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let row_id = next[0].1;

        // Get the delay before the next schedule
        // If delay is set to 0, caching is disable for that row
        let delay: f64 = client.zscore("delay:", row_id).await?;
        if delay <= 0.0 {
            // The item shouldn’t be cached anymore; remove it from the cache
            let () = client.zrem("delay:", row_id).await?;
            let () = client.zrem("schedule:", row_id).await?;
            let () = client.del(format!("inv:{}", row_id)).await?;
            continue;
        }

        // Get the database row.
        let json = Db::get(row_id);
        // Schedule that row for new update later
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
        // Set the cache value
        let () = client
            .set(format!("inv:{}", row_id), json, None, None, false)
            .await?;
    }
}
