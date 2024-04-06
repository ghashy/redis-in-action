# Schema

> We use here `&` symbol as delimiter between zset score and value.

> We use here `->` symbol as delimiter between list elements.

## Shopping cart cookies block

| Name                                            | Type     | Key                           | Expiration | Module                                                                 |
| ----------------------------------------------- | -------- | ----------------------------- | ---------- | ---------------------------------------------------------------------- |
| [Hash with cookies](#hash-with-cookies)         | **HASH** | `login:`                      | No         | `crate::session_cookie`                                                |
| [Recently used tokens](#recently-used-tokens)   | **ZSet** | `recent:`                     | No         | `crate::session_cookie`                                                |
| [Recently viewed items](#recently-viewed-items) | **List** | `viewed:{uuid_session_token}` | No         | `crate::session_cookie`                                                |
| [Popular items](#popular-items)                 | **ZSet** | `viewed:`                     | No         | `crate::session_cookie`, `crate::analytics`, `crate::web_page_caching` |
| [Quantity](#quantity)                           | **HASH** | `cart:{uuid_session_token}`   | No         | `crate::session_cookie`, `crate::shopping_cart`                        |

## Database rows cache block

| Name                            | Type             | Key            | Expiration | Module                      |
| ------------------------------- | ---------------- | -------------- | ---------- | --------------------------- |
| [Database rows](#database-rows) | **String(json)** | `inv:{row_id}` | No         | `crate::database_row_cache` |
| [Schedule](#schedule)           | **ZSet**         | `schedule:`    | No         | `crate::database_row_cache` |
| [Delay](#delay)                 | **ZSet**         | `delay:`       | No         | `crate::database_row_cache` |

## Web page caching block

| Name                  | Type               | Key                      | Expiration | Module                    |
| --------------------- | ------------------ | ------------------------ | ---------- | ------------------------- |
| [Web page](#web-page) | **String(string)** | `cache:{html_page_hash}` | No         | `crate::web_page_caching` |

### Hash with cookies

These are cookies, hash with pairs `token: user`.

```json
"{uuid_session_token}": "{username}"
"{uuid_session_token}": "{goodboy1}"
```

### Recently used tokens

Stores timestamp when the token was last used to perform requests.

```json
"{unix_timestamp}"  & "{uuid_session_token}"
"1711794081.470433" & "{uuid_session_token}"
```

### Recently viewed items

Records which user(uuid_user_token) seen which item.

```json
"{item1}" ->  "{item2}"
```

### Popular items

Records which item is the most popular. The most popular with the lowest score
We use that negative scale here to keep the most popular at the beginning of the zset.

```json
"{rating_score}" & "{item1}"
"-23.0"   & "{item2}"
"-1.0"    & "{item3}"
```

### Quantity

That maps an item ID to the quantity of that item, that the customer would like to purchase.

```json
"{item}": "{quantity}"
"item2": "10"
"item3": "3"
```

### Database rows

Cached database row for an item to be sold online in JSON format.

```json
{ "qty": 629, "name": "GTab 7inch", "description": "..." }
```

### Schedule

Row ID from the db row as the member of the ZSET. Scores is timestamp,
when the row should be copied to Redis next.

```json
"{unix_timestamp}"  & "{row_id}"
"1711794081.470433" & "237_row_id"
```

### Delay

Row ID for the members, the score is how many seconds
to wait between cache updates.

```json
"{seconds_amount}" & "{row_id}"
"10.0"             & "237"
```

### Web page

Cached html page.

```txt
"html-content"
```

### Viewed pages

Top of most viewed items pages.

```json
"{rating_score}" & "{item}"
```
