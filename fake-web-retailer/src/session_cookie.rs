use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::{HashesInterface, KeysInterface, SortedSetsInterface};

use crate::get_sys_time_in_secs;

async fn check_token(
    client: &RedisClient,
    token: &str,
) -> Result<String, RedisError> {
    client.hget("login:", token).await
}

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
        client
            .zadd(
                format!("viewed:{}", token),
                None,
                None,
                false,
                false,
                vec![(timestamp as f64, item)],
            )
            .await?;
        // Remove old items, keeping the most recent 25.
        client.zremrangebyrank(format!("viewed:{}", token), 0, -26).await?;
        // With this one line added, we now have a record of all of the items that are viewed.
        // Even more useful, that list of items is ordered by the number of times that people
        // have seen the items, with the most-viewed item having the lowest score, and thus hav- ing an index of 0.
        client.zincrby("viewed:", -1.0, item).await?;
    }
    Ok(())
}

async fn clean_sessions_task(client: &RedisClient) -> Result<(), RedisError> {
    static LIMIT: i64 = 10000000;

    loop {
        // Find out how many tokens are known.
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
