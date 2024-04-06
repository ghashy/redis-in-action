# Shema

> We use here `&` symbol as delimiter between zset score and value.

> We use here `->` symbol as delimiter between list elements.

## Shopping cart cookies block

| Name                                             | Type     | Key                      | Expiration | Module    |
| ------------------------------------------------ | -------- | ------------------------ | ---------- | --------- |
| [Hits hash](#hits-hash)                          | **Hash** | `count:{sec_count}:hits` | No         | `main.rs` |
| [Known counters](#zset-with-some-known-counters) | **ZSet** | `known:`                 | No         | `main.rs` |

### Hits hash

A Hash that shows the number of web page hits over 5-second time slices around 7:40 a.m. on May 7, 2012.

```json
{
  "1336376410": 45,
  "1336376405": 28,
  "1336376395": 17,
  "1336376395": 17,
  "1336376400": 29
}
```

### ZSet with some known counters

When scores are equal as they are in this ZSET, Redis sorts by member name.

```json
"0" & "1:hits"
"0" & "5:hits"
"0" & "60:hits"
```
