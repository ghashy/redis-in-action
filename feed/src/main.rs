#![allow(dead_code)]
#![allow(clippy::needless_return)]

use std::{collections::HashMap, time::SystemTime};

use fred::clients::{RedisClient, Transaction};
use fred::error::RedisError;
use fred::interfaces::{
    ClientLike, HashesInterface, KeysInterface, SetsInterface,
    SortedSetsInterface, TransactionInterface,
};
use fred::types::RedisConfig;

const SECONDS_IN_DAY: i64 = 86_400;
const VOTES_REQUIRED: i64 = 200;
const RATIO: i64 = SECONDS_IN_DAY / VOTES_REQUIRED;
const ONE_WEEK_IN_SECONDS: i64 = SECONDS_IN_DAY * 7;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let client = init_redis_client().await;

    // post_article(&client, "johhn", "cats-of-world", "google.com/kittens120")
    //     .await
    //     .unwrap();
    // add_remove_groups(&client, 2, &["programming"], &[]).await.unwrap();
    // for i in 11..100 {
    //     article_voting(&client, &format!("user:{i}"), "article:1")
    //         .await
    //         .unwrap();
    // }
    // let articles = get_article_order_by_score(&client, 1).await.unwrap();
    let articles = get_group_articles_by_score(&client, "programming", 1)
        .await
        .unwrap();
    dbg!(articles);
}

/// This function posts a new article, adds hset with article information,
/// then add article to the `time:` and `score` zsets.
async fn post_article(
    client: &RedisClient,
    user: &str,
    title: &str,
    link: &str,
) -> Result<u32, RedisError> {
    let article_id = client.incr::<u32, _>("article:").await?;
    let voted = format!("upvoted:{article_id}");
    client.sadd(&voted, user).await?;
    client.expire::<(), _>(voted, ONE_WEEK_IN_SECONDS).await?;

    let now = get_sys_time_in_secs();
    let article = format!("article:{article_id}");
    client
        .hmset::<(), _, _>(
            &article,
            vec![
                ("title", title),
                ("link", link),
                ("author", user),
                ("time", &now.to_string()),
                ("upvotes", "1"),
                ("downvotes", "0"),
            ],
        )
        .await?;
    client
        .zadd(
            "score:",
            None,
            None,
            true,
            true,
            vec![(now as f64 + RATIO as f64, &article)],
        )
        .await?;
    client
        .zadd("time:", None, None, true, true, vec![(now as f64, article)])
        .await?;

    Ok(article_id)
}

/// Add or remove groups
async fn add_remove_groups(
    client: &RedisClient,
    article_id: usize,
    to_add: &[&str],
    to_remove: &[&str],
) -> Result<(), RedisError> {
    let article = format!("article:{article_id}");
    for group in to_add.into_iter() {
        client
            .sadd::<(), _, _>(format!("group:{group}"), &article)
            .await?;
    }

    for group in to_remove.into_iter() {
        client
            .srem::<(), _, _>(format!("group:{group}"), &article)
            .await?;
    }

    Ok(())
}

/// This function caches articles of the same group in the
/// `score:{group_name}` zset for 1 minute.
/// And returns these articles using `get_articles_ordered_by_score`
async fn get_group_articles_by_score(
    client: &RedisClient,
    group: &str,
    page: i64,
) -> Result<Vec<HashMap<String, String>>, RedisError> {
    let destination = format!("score:{}", group);
    let multi = client.multi();

    // Check is there temporary zset of articles
    if !multi.exists(&destination).await? {
        // If no, create new
        multi
            .zinterstore(
                &destination,
                &[format!("group:{}", group), "score:".to_string()],
                vec![1., 1.],
                Some(fred::types::AggregateOptions::Max),
            )
            .await?;
        multi.expire(&destination, 60).await?;
    }

    get_articles_ordered_by_score(&multi, page, &destination).await
}

/// This function fetches articles info ordered by score, using
/// temporary zset `score:{group_name}`, and each article's hset.
async fn get_articles_ordered_by_score(
    multi: &Transaction,
    page: i64,
    zset_key: &str,
) -> Result<Vec<HashMap<String, String>>, RedisError> {
    type ArticleKey = String;
    type ArticleScore = i64;

    const ARTICLES_PER_PAGE: i64 = 25;

    let start = (page - 1) * ARTICLES_PER_PAGE;
    let end = start + ARTICLES_PER_PAGE - 1;

    // Use vec here to perserve order
    // We get article scores and keys from given temporary zset.
    let ids = multi
        .zrevrange::<Vec<(ArticleKey, ArticleScore)>, _>(
            zset_key, start, end, true,
        )
        .await?;

    // Fetch all articles data, one afther one
    let mut articles = Vec::new();
    for (key, _score) in ids.into_iter() {
        let mut article_data =
            multi.hgetall::<HashMap<String, String>, _>(&key).await?;
        article_data.insert("id".to_string(), key);
        articles.push(article_data);
    }
    Ok(articles)
}

/// Vote for certain article
/// We can add vote if there are no vote for given user
/// or change vote from upvote -> downvote or downvote -> upvote.
async fn article_vote(
    client: &RedisClient,
    user: &str,
    article: &str,
    is_upvote: bool,
) -> Result<(), RedisError> {
    let week_ago = get_sys_time_in_secs() - ONE_WEEK_IN_SECONDS as u64;

    // Check that article was not created too many time ago
    let timestamp = client.zscore::<u64, _, _>("time:", article).await.unwrap();
    if timestamp < week_ago {
        return Ok(());
    }

    // Get article id
    let article_id = article.split_terminator(':').skip(1).next().unwrap();

    let multi = client.multi();

    // Get current vote status
    // That expression returns 1 if inserted or 0 if not
    // * `true` here is upvote
    // * `false` is downvote
    // * `None` means no any vote
    let current = if multi
        .sismember(format!("upvoted:{article_id}"), user)
        .await?
    {
        Some(true)
    } else if multi
        .sismember(format!("downvoted:{article_id}"), user)
        .await?
    {
        Some(false)
    } else {
        None
    };

    match (current, is_upvote) {
        // No changes
        (Some(true), true) | (Some(false), false) => {
            return Ok(());
        }

        // Toggle
        (Some(true), false) | (Some(false), true) => {
            let (from, to, upvote_count_diff, downvote_count_diff, score_diff) =
                if is_upvote {
                    ("downvoted", "upvoted", 1, -1, -RATIO as f64 * 2.)
                } else {
                    ("upvoted", "downvoted", -1, 1, RATIO as f64 * 2.)
                };

            multi
                .smove(
                    format!("{}:{}", from, article_id),
                    format!("{}:{}", to, article_id),
                    user,
                )
                .await?;

            multi
                .hincrby::<(), _, _>(article, "upvotes", upvote_count_diff)
                .await?;
            multi
                .hincrby::<(), _, _>(article, "downvotes", downvote_count_diff)
                .await?;
            multi
                .zincrby::<(), _, _>("score:", score_diff, article)
                .await?;
        }

        // Add vote
        _ => {
            let (key, ratio) = if is_upvote {
                ("upvoted", RATIO as f64)
            } else {
                ("downvoted", -RATIO as f64)
            };
            let _: bool =
                multi.sadd(format!("{key}:{article_id}"), user).await?;
            let () = multi.zincrby("score:", ratio, article).await?;
            let () = multi.hincrby(article, key, 1).await?;
        }
    }
    multi.exec(true).await?;

    Ok(())
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
