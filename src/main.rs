use std::collections::HashMap;
use std::time::SystemTime;

use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::{
    ClientLike, HashesInterface, KeysInterface, SortedSetsInterface,
};
use fred::types::RedisConfig;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let client = init_redis_client().await;
}
