use fred::clients::RedisClient;
use fred::error::RedisError;
use fred::interfaces::{KeysInterface, SortedSetsInterface};

// ───── Cache content, PSEUDO CODE ───────────────────────────────────────── //

struct Request;
type Response = String;
type ItemId = u32;

/// Caching middleware
async fn cache_request<F>(
    client: &RedisClient,
    request: Request,
    callback: F,
) -> Result<Response, RedisError>
where
    F: Fn(&Request) -> Response,
{
    if !should_cache(client, &request).await? {
        return Ok(callback(&request));
    }
    let page_key = format!("cache:{}", hash_request(&request));
    let content: Option<String> = Some(client.get(&page_key).await?);
    if let None = content {
        let content = callback(&request);
        let () = client
            .set(
                &page_key,
                content,
                Some(fred::types::Expiration::EX(300)),
                None,
                false,
            )
            .await?;
    }
    return Ok(content.unwrap());
}

async fn should_cache(
    client: &RedisClient,
    request: &Request,
) -> Result<bool, RedisError> {
    // Get the item ID for the page, if any.
    let item_id = extract_item_id(&request);
    // Check whether the page can be statically cached and whether this is an item page.
    if item_id.is_none() || is_dynamic_page(&request) {
        return Ok(false);
    }
    // Get the rank of the item, from shopping_cart module
    let rank: Option<u32> = client.zrank("viewed:", item_id).await?;
    // Return whether the item has a high enough view count to be cached.
    // 0 the highest.
    if let Some(rank) = rank {
        Ok(rank < 10000)
    } else {
        Ok(false)
    }
}

fn extract_item_id(req: &Request) -> Option<ItemId> {
    Some(1)
}

fn is_dynamic_page(req: &Request) -> bool {
    false
}

fn hash_request(_request: &Request) -> String {
    String::new()
}
