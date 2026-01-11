use prometheus_http_query::{Client, response::RangeVector};
use std::error::Error;
use chrono::{Utc, Duration};

fn parse_duration(input: &str) -> Duration {
    let (num_part, unit) = input.split_at(input.len() - 1);
    let num: i64 = num_part.parse().unwrap_or(1);

    match unit {
        "d" => Duration::days(num),
        "h" => Duration::hours(num),
        "m" => Duration::minutes(num),
        _ => Duration::days(7),
    }
}

pub async fn get_metric_range_data(metric_server: &str, interval: &str, query: &str) -> Result<Option<Vec<RangeVector>>, Box<dyn Error>> {
    let client = Client::try_from(metric_server)?;
    let duration = parse_duration(interval);

    let end = Utc::now();
    let start = end - duration;
    let step = 15.0;

    let response = client
        .query_range(query, start.timestamp(), end.timestamp(), step)
        .get()
        .await?;

    let data = response.data().as_matrix().cloned();

    Ok(data)
}