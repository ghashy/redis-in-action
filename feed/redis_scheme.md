# Scheme

## Keys

### Articles
> **String** KEY: "article:"
> **HASH**   KEY: "article:92617"
> **ZSET**   KEY: "time:"
> **ZSET**   KEY: "score:"
> **SET**    KEY: "upvoted:123123"
> **SET**    KEY: "downvoted:123123"
> **SET**    KEY: "group:programming"
> **SET**    KEY: "score:programming"

## Structures

### Articles
- **String**, articles index, stores count of articles (or last index) 
> KEY: "article:"

- **HASH**, articles, 
> KEY: "article:92617"
  ```
  title: "title"
  link: "link.com"
  author: "user:83123"
  time: "1723.123"
  votes: "123"
  ```

- **ZSET**, sorted set of articles, ordered by time being posted 
> KEY: "time:"
  ```
  "123123.123:article"
  ```

- **ZSET**, sorted set of articles, ordered by item scores 
> KEY: "score:"
  ```
  "123123.123:article"
  ```

- **SET**, set with info who have voted for article 
> KEY: "voted:123123"
  ```
  "user:123123"
  ```

- **SET**, groups with articles (sets) 
> KEY: "group:programming"
  ```
  "article:123123"
  ```

- **SET**, intersections of sorted set of articles and group 
> KEY: "score:programming"
  ```
  "article:123123"
  ```


