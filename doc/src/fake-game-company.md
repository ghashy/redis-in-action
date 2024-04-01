# Schema

> We use here `&` symbol as delimiter between zset score and value.

> We use here `->` symbol as delimiter between list elements.

## Shopping cart cookies block

| Name                                        | Type     | Key                   | Expiration | Module    |
| ------------------------------------------- | -------- | --------------------- | ---------- | --------- |
| [Hash with user data](#hash-with-user-data) | **HASH** | `users:{user_id}`     | No         | `main.rs` |
| [Set with inventory](#inventory-set)        | **Set**  | `inventory:{user_id}` | No         | `main.rs` |
| [Market](#market)                           | **ZSet** | `market:`             | No         | `main.rs` |

### Hash with user data

User information is stored as a HASH, with keys and values that store user attributes.

```json
"name": "{username}"
"funds": "{amount_of_funds}"
```

### Inventory set

A user’s inventory that holds unique identifiers for each item.

```json
"ItemL" -> "ItemM" -> "ItemN"
```

### Market

Market with goods and their prices

```json
"{item_price}" & "{item_name}.{user_id}"
```
