use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::{
    HashesInterface, KeysInterface, ListInterface, SortedSetsInterface,
};

use crate::get_sys_time_in_secs;

async fn check_token(
    client: &RedisClient,
    token: &str,
) -> Result<String, RedisError> {
    client.hget("login:", token).await
}

/// If user perform any request, we should update user's token,
/// this functions updates token-was-used request timestamp to `now`
/// and if user viewed some item, we store that item in
/// `viewed:{uuid_user_token}` zset with timestamp when it was seen.
/// We keep that zset size within bound of 0..25
async fn update_token(
    client: &RedisClient,
    token: &str,
    user: &str,
    item: Option<&str>,
) -> Result<(), RedisError> {
    // Get the timestamp.
    let timestamp = get_sys_time_in_secs();
    // Keep a mapping from the token to the logged-in user.
    let () = client.hset("login:", vec![(token, user)]).await?;
    // Record when the token was last seen.
    let () = client
        .zadd(
            "recent:",
            None,
            None,
            false,
            false,
            vec![(timestamp as f64, token)],
        )
        .await?;
    if let Some(item) = item {
        // Record that the user viewed the item.
        let recently_viewed_items = format!("viewed:{}", token);
        // ↓ REPLACED: client
        //     .zadd(
        //         &recently_viewed_items,
        //         None,
        //         None,
        //         false,
        //         false,
        //         vec![(timestamp as f64, item)],
        //     )
        //     .await?;
        client.lpush(&recently_viewed_items, item).await?;
        // Remove old items, keeping the most recent 25.
        // ↓ REPLACED: client.zremrangebyrank(format!("viewed:{}", token), 0, -26).await?;
        client.lrange(recently_viewed_items, 0, 26).await?;
        // With this one line added, we now have a record of all of the items that are viewed.
        // Even more useful, that list of items is ordered by the number of times that people
        // have seen the items, with the most-viewed item having the lowest score, and thus having an index of 0.
        client.zincrby("viewed:", -1.0, item).await?;
    }
    Ok(())
}

/// This task should run every `some_amount` period of time in background.
/// We’ll only keep the most recent 10 million sessions, so we track sessions
/// in the `recent:` zset. If there are more sessions than in `LIMIT`, we delete
/// first N sessions (oldest) fron that zset, remove them from
/// `viewed:{uuid_session_token}` zset, and from `login:` HASH.
async fn clean_sessions_task(client: &RedisClient) -> Result<(), RedisError> {
    static LIMIT: i64 = 10000000;

    loop {
        // Find out how many tokens are known (cardinality).
        let size: i64 = client.zcard("recent:").await?;
        if size <= LIMIT {
            return Ok(());
        }

        // Fetch the token IDs that should be removed.
        let end_index = (size - LIMIT).min(100);
        let tokens: Vec<String> = client
            .zrange(
                "recent:",
                0,
                end_index - 1,
                /* default sort */ None,
                /* not reversed, asc */ false,
                /* not related to rank, only to score */ None,
                /* without scores */ false,
            )
            .await?;

        // Prepare the key names for the tokens to delete.
        let mut session_keys = Vec::new();
        for token in tokens.iter() {
            // We will remove data from viewed zset
            session_keys.push(format!("viewed:{}", token));
            // We will remove items from shopping cart
            session_keys.push(format!("cart:{}", token));
        }

        // Remove the oldest tokens.
        let () = client.del(session_keys).await?;
        let () = client.hdel("login:", tokens.clone()).await?;
        let () = client.zrem("recent:", tokens).await?;
    }
}
