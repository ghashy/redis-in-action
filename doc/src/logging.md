# Shema

> We use here `&` symbol as delimiter between zset score and value.

> We use here `->` symbol as delimiter between list elements.

## Shopping cart cookies block

| Name                                                                | Type            | Key                               | Expiration | Module    |
| ------------------------------------------------------------------- | --------------- | --------------------------------- | ---------- | --------- |
| [Recent logs](#recent-logs)                                         | **List**        | `recent:{name}:{severity}`        | No         | `main.rs` |
| [Common logs](#common-logs)                                         | **ZSet**        | `common:{name}:{severity}`        | No         | `main.rs` |
| [Common logs for previous hour](#common-logs)                       | **ZSet**        | `common:{name}:{severity}:last`   | No         | `main.rs` |
| [Common logs start hour](#common-logs-start-hour)                   | **String(int)** | `common:{name}:{severity}:start`  | No         | `main.rs` |
| [Common logs start hour for previous hour](#common-logs-start-hour) | **String(int)** | `common:{name}:{severity}:pstart` | No         | `main.rs` |

### Recent logs

Set with logs.

```json
"{logline}" -> "{logline}"
```

### Common logs

How often we get this log.

```json
"{entriesd_count}" & "{message}"
```

### Common logs start hour

Start of the hour in unix timestamp format.

```json
"{timestamp}"
```
