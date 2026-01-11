use clap::Parser;

mod promql_client;
mod process_data;
mod models;

use models::{MemoryRequest, MemoryUsage, MetricPoint};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    metric_server: String,
    #[arg(long)]
    interval: String

}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    
    let query_request = r#"max(kube_pod_container_resource_requests{resource="memory"}) by (pod)"#;
    let query_usage = r#"max(container_memory_working_set_bytes{name!=""}) by (pod)"#;

    let mut request_data_vec: Vec<MemoryRequest> = Vec::new();
    let mut usage_data_vec: Vec<MemoryUsage> = Vec::new();

    // ----------------- Get Memory Request Data -----------------
    println!("Fetching Memory Requests...");
    match promql_client::get_metric_range_data(&args.metric_server, &args.interval, query_request).await {
        Ok(Some(matrix)) => {
            for series in matrix {
                let pod_name = series.metric().get("pod").map(|s| s.as_str().to_string()).unwrap_or_default();
                let metrics = series.samples().iter().map(|s| MetricPoint { timestamp: s.timestamp(), value: s.value() }).collect();
                
                request_data_vec.push(MemoryRequest { pod_name, metrics });
            }
        },
        _ => eprintln!("Failed to get requests"),
    }

    // ----------------- Get Memory Usage Data -----------------
    println!("Fetching Memory Usage...");
    match promql_client::get_metric_range_data(&args.metric_server, &args.interval, query_usage).await {
        Ok(Some(matrix)) => {
            for series in matrix {
                let pod_name = series.metric().get("pod").map(|s| s.as_str().to_string()).unwrap_or_default();
                let metrics = series.samples().iter().map(|s| MetricPoint { timestamp: s.timestamp(), value: s.value() }).collect();
                
                usage_data_vec.push(MemoryUsage { pod_name, metrics });
            }
        },
        _ => eprintln!("Failed to get usage"),
    }

    // ----------------- Process & Compare -----------------
    if !request_data_vec.is_empty() && !usage_data_vec.is_empty() {
        process_data::compare_data(request_data_vec, usage_data_vec);
    } else {
        println!("Insufficient data to compare.");
    }
}