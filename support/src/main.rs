use std::time::SystemTime;

use fred::{clients::RedisClient, interfaces::ClientLike, types::RedisConfig};

mod counters;
mod logging;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let _client = init_redis_client().await;
}

// ───── Helpers ──────────────────────────────────────────────────────────── //

fn get_sys_time_in_secs() -> u64 {
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
