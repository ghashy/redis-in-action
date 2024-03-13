# Schema

## Keys

### Shopping cart cookies
> **HASH**   KEY: "login:"
> **ZSET**   KEY: "recent:"
> **ZSET**   KEY: "viewed:EB9382CA-0376-4558-B271-EA230BDB7EAF"
> **ZSET**   KEY: "viewed:"
> **HASH**   KEY: "cart:EB9382CA-0376-4558-B271-EA230BDB7EAF"

### Database rows cache
> **String** KEY: "inv:273"
> **ZSET**   KEY: "delay:"
> **ZSET**   KEY: "schedule:"

### Web page caching
> **String** KEY: "cache:236712hhl3213yu21_hash"
> **ZSET**   KEY: "viewed:"

## Structures

### Shopping cart cookies
- **HASH**, hash with pairs `token: user` 
> KEY: "login:"
  ```
  EB9382CA-0376-4558-B271-EA230BDB7EAF: "user1"
  E1F27C8C-527C-42CC-BCD0-1F8A433F7A44: "user2"
  ```

- **ZSET**, stores timestamp when the token was last seen
> KEY: "recent:"
  ```
  1234567890_timestamp: "EB9382CA-0376-4558-B271-EA230BDB7EAF"
  1234567890_timestamp: "E1F27C8C-527C-42CC-BCD0-1F8A433F7A44"
  ```

- **ZSET**, records which user(token) seen which item
> KEY: "viewed: EB9382CA-0376-4558-B271-EA230BDB7EAF"
  ```
  1234567890_timestamp: "item1"
  1234567890_timestamp: "item2"
  ```

- **ZSET**, records which item is the most popular
> KEY: "viewed:"
  ```
  -23.0: "item1"
  -1.0: "item2"
  ```

- **HASH**, that maps an item ID to the quantity of that item that the customer would like to purchase.
> KEY: "cart:EB9382CA-0376-4558-B271-EA230BDB7EAF"
  ```
  item1: "10"
  item2: "3"
  ```

### Database rows cache
- **String**, cached database row for an item to be sold online in JSON format
> KEY: "inv:273"
  ```
  '{"qty":629, "name":
  "GTab 7inch", "description": "..."}'
  ```

- **ZSET**, row ID for the members, the score is how many seconds to wait between cache updates
> KEY: "delay:"
  ```
  "10.0:237_row_id"
  ```

- **ZSET**, row ID from the db row as the member of the ZSET. Scores is timestamp (when the row should be copied to Redis next)
> KEY: "schedule:"
  ```
  "123123123.123_timestamp:237_row_id"
  ```

### Web page caching
- **String**, cached html pages
> KEY: "cache:236712hhl3213yu21_hash"
  ```
  "html-content"
  ```
