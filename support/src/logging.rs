use crate::get_sys_time_in_secs;
use time::Time;

use fred::{
    clients::{RedisClient, Transaction},
    error::RedisError,
    interfaces::{
        KeysInterface, ListInterface, SortedSetsInterface, TransactionInterface,
    },
};
use time::OffsetDateTime;

#[derive(Clone, Copy)]
enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Debug => f.write_str("Debug"),
            Severity::Info => f.write_str("Info"),
            Severity::Warning => f.write_str("Warning"),
            Severity::Error => f.write_str("Error"),
            Severity::Critical => f.write_str("Critical"),
        }
    }
}

async fn log_recent(
    client: &Transaction,
    name: &str,
    message: &str,
    severity: Severity,
) -> Result<(), RedisError> {
    let destination = format!("recent:{}:{}", name, severity);
    let message = formatted_datetime(OffsetDateTime::now_utc()) + message;
    client.lpush(&destination, message).await?;
    client.ltrim(destination, 0, 99).await?;
    client.exec(true).await?;
    Ok(())
}

async fn log_common(
    client: &RedisClient,
    name: &str,
    message: &str,
    severity: Severity,
    timeout: u64,
) -> Result<(), RedisError> {
    // Set up the destination key for keeping recent logs.
    let destination = format!("common:{name}:{severity}");
    // Keep a record of the start of the hour for this set of messages.
    let start_key = format!("{destination}:start");
    let end = get_sys_time_in_secs() + timeout;
    while get_sys_time_in_secs() < end {
        // We’ll watch the start of the hour key for changes that only happen at the beginning of the hour.
        client.watch(&start_key).await?;
        // Get the current time and find the current start hour.
        let hour_start = datetime_trunc_hour(OffsetDateTime::now_utc());
        let existing: Option<i64> = client.get(&start_key).await?;
        let multi = client.multi();
        if let Some(existing) = existing {
            let existing =
                OffsetDateTime::from_unix_timestamp(existing).unwrap();
            // If the current list of common logs is for a previous hour
            if existing < hour_start {
                multi
                    .rename(&destination, format!("{destination}:last"))
                    .await?;
                multi
                    .rename(&start_key, format!("{destination}:pstart"))
                    .await?;
                // Update the start of the current hour for the common logs.
                multi
                    .set(
                        &start_key,
                        hour_start.unix_timestamp(),
                        None,
                        None,
                        false,
                    )
                    .await?;
                // Actually increment our common counter.
                multi.zincrby(&destination, 1.0, message).await?;
                // Call the log_recent() function to record these, and rely on its call to execute().
                log_recent(&multi, name, message, severity).await?;
            }
        }
    }
    Ok(())
}

// ───── Helpers ──────────────────────────────────────────────────────────── //

fn formatted_datetime(timestamp: OffsetDateTime) -> String {
    timestamp
        .format(&time::format_description::well_known::Rfc2822)
        .unwrap()
}

fn datetime_trunc_hour(timestamp: OffsetDateTime) -> OffsetDateTime {
    let hour = timestamp.hour();
    let datetime = time::OffsetDateTime::now_utc();
    datetime.replace_time(Time::from_hms(hour, 0, 0).unwrap())
}
