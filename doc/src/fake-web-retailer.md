# Schema

We use here `&` symbol as delimiter between zset score and value.

## Shopping cart cookies block

| Name                                    | Type     | Key                           | Expiration |
| --------------------------------------- | -------- | ----------------------------- | ---------- |
| [Hash with cookies](#hash-with-cookies) | **HASH** | `login:`                      | No         |
| [Recent](#recent)                       | **ZSet** | `recent:`                     | No         |
| [User viewed items](#user-viewed-items) | **ZSet** | `viewed:{uuid_session_token}` | No         |
| [Popular items](#popular-items)         | **ZSet** | `viewed:`                     | No         |
| [Quantity](#quantity)                   | **HASH** | `cart:{uuid_session_token}`   | No         |

## Database rows cache block

| Name                            | Type             | Key            | Expiration |
| ------------------------------- | ---------------- | -------------- | ---------- |
| [Database rows](#database-rows) | **String(json)** | `inv:{row_id}` | No         |
| [Schedule](#schedule)           | **ZSet**         | `schedule:`    | No         |
| [Delay](#delay)                 | **ZSet**         | `delay:`       | No         |

## Web page caching block

| Name                           | Type               | Key                      | Expiration |
| ------------------------------ | ------------------ | ------------------------ | ---------- |
| [Web page](#web-page)          | **String(string)** | `cache:{html_page_hash}` | No         |
| [Viewed paages](#viewed-pages) | **ZSet**           | `viewed:`                | No         |

> **ZSET** KEY: "viewed:"

### Hash with cookies

These are cookies, hash with pairs `token: user`, body example:

```json
"{uuid_session_token}": "{username}"
"{uuid_session_token}": "{goodboy1}"
```

### Recent

Stores timestamp when the token was last used to perform requests, body example:

```json
"{timestamp}" & "{uuid_session_token}"
"{123123123}" & "{uuid_session_token}"
```

### User viewed items

Records which user(uuid_user_token) seen which item, body example:

```json
"{timestamp}" & "{item1}"
"{123123231}" & "{item2}"
```

### Popular items

Records which item is the most popular. The most popular with the lowest score
We use that negative scale here to keep the most popular at the beginning of the zset, body example:

```json
"{score}" & "{item1}"
"-23.0"   & "{item2}"
"-1.0"    & "{item3}"
```

### Quantity

That maps an item ID to the quantity of that item, that the customer would like to purchase, body example:

```json
"{item}": "{quantity}"
"item2": "10"
"item3": "3"
```

### Database rows

Cached database row for an item to be sold online in JSON format, body example:

```json
{ "qty": 629, "name": "GTab 7inch", "description": "..." }
```

### Schedule

Row ID from the db row as the member of the ZSET. Scores is timestamp,
when the row should be copied to Redis next, body example:

```json
"{timestamp}"             & "{row_id}"
"123123123.123_timestamp" & "237_row_id"
```

### Delay

Row ID for the members, the score is how many seconds
to wait between cache updates, body example:

```json
"{seconds_amount}" & "{row_id}"
"10.0"             & "237"
```

### Web page

Cached html page

```txt
"html-content"
```

### Viewed pages

Top of most viewed items pages

```json
"{rating_score}" & "{item}"
```
