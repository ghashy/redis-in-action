# Feed schema

We use here `&` symbol as delimiter between zset score and value.

## Articles block

| Name                                                                    | Type            | Key Example          | Expiration | Module    |
| ----------------------------------------------------------------------- | --------------- | -------------------- | ---------- | --------- |
| [Articles count](#articles-count)                                       | **String(int)** | `article:`           | No         | `main.rs` |
| [Articles](#articles)                                                   | **Hash**        | `article:92617`      | No         | `main.rs` |
| [Articles, time-ordered](#articles-time-ordered)                        | **ZSet**        | `time:`              | No         | `main.rs` |
| [Articles, item-score-ordered](#articles-item-score-ordered)            | **ZSet**        | `score:`             | No         | `main.rs` |
| [Article votes](#article-votes)                                         | **Set**         | `upvoted:123123`     | No         | `main.rs` |
| `Same`                                                                  | **Set**         | `downvoted:123123`   | No         | `main.rs` |
| [Article groups](#article-groups)                                       | **Set**         | `group:{group_name}` | No         | `main.rs` |
| [Group of articles sorted by score](#group-of-articles-sorted-by-score) | **ZSet**        | `score:{group_name}` | 1 min      | `main.rs` |

### Articles count

Stores count of articles (or last index), body example:

```json
10
```

### Articles

Article content, body example:

```json
title: "title"
link: "link.com"
author: "user:83123"
time: "1723.123"
votes: "123"
```

### Articles, time-ordered

Sorted set of articles, ordered by time being posted, body example:

```json
"123123.123 & article:{article_id}"
```

### Articles, item-score-ordered

Sorted set of articles, ordered by item scores, body example:

```json
"123123.123" & "article:{article_id}"
```

### Article votes

Set with info who have voted for article, body example:

```json
"user:123123"
```

### Article groups

Groups with articles (sets), body example:

```json
"article:{article_id}"
```

### Group of articles sorted by score

Intersections of sorted by score set of articles and group, from that we got
sorted articles of certain group, body example:

```json
"article:{article_id}"
```
