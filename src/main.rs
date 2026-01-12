use clap::Parser;

mod promql_client;
mod process_data;
mod models;
mod tui;

use models::{MemoryRequest, MemoryUsage, MetricPoint};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    metric_server: String,
    #[arg(long)]
    interval: String,
    #[arg(long)]
    filter: Option<String>,
    #[arg(long)]
    namespace: Option<String>,
    #[arg(long)]
    tui: bool,    

}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    
    // init filter namespace for prometheus query 
    let ns_filter_request = if args.namespace.is_none() {
        r#"resource="memory""#.to_string()
    } else {
        format!(r#"resource="memory", namespace="{}""#, args.namespace.as_ref().unwrap())
    };

    let ns_filter_usage = if args.namespace.is_none() {
        r#"name!="""#.to_string()
    } else {
        format!(r#"name!="", namespace="{}""#, args.namespace.as_ref().unwrap())
    };

    // Prometheus query
    let query_request = format!(r#"max(kube_pod_container_resource_requests{{{}}}) by (namespace, pod)"#, ns_filter_request);
    let query_usage = format!(r#"max(container_memory_working_set_bytes{{{}}}) by (namespace, pod)"#, ns_filter_usage);

    let mut request_data_vec: Vec<MemoryRequest> = Vec::new();
    let mut usage_data_vec: Vec<MemoryUsage> = Vec::new();

    // Fetch memory request data
    println!("Fetching Memory Requests...");
    match promql_client::get_metric_range_data(&args.metric_server, &args.interval, &query_request).await {
        Ok(Some(matrix)) => {
            for series in matrix {
                let pod_name = series.metric().get("pod").map(|s| s.as_str().to_string()).unwrap_or_default();
                // Extract Namespace here
                let namespace = series.metric().get("namespace").map(|s| s.as_str().to_string()).unwrap_or_default();
                
                let metrics = series.samples().iter().map(|s| MetricPoint { timestamp: s.timestamp(), value: s.value() }).collect();
                
                request_data_vec.push(MemoryRequest { pod_name, namespace, metrics });
            }
        },
        _ => eprintln!("Failed to get requests"),
    }

    // Fetch memory usage data
    println!("Fetching Memory Usage...");
    match promql_client::get_metric_range_data(&args.metric_server, &args.interval, &query_usage).await {
        Ok(Some(matrix)) => {
            for series in matrix {
                let pod_name = series.metric().get("pod").map(|s| s.as_str().to_string()).unwrap_or_default();
                // Extract Namespace here
                let namespace = series.metric().get("namespace").map(|s| s.as_str().to_string()).unwrap_or_default();
                
                let metrics = series.samples().iter().map(|s| MetricPoint { timestamp: s.timestamp(), value: s.value() }).collect();
                
                usage_data_vec.push(MemoryUsage { pod_name, namespace, metrics });
            }
        },
        _ => eprintln!("Failed to get usage"),
    }

    // ... run TUI logic ...
    if !request_data_vec.is_empty() && !usage_data_vec.is_empty() {
        if args.tui {
            // Mode TUI
            match tui::run_tui(request_data_vec, usage_data_vec) {
                Ok(_) => {},
                Err(e) => eprintln!("Error running TUI: {}", e),
            }
        } else {
            // Mode CLI
            process_data::compare_data(
                request_data_vec, 
                usage_data_vec, 
                args.filter.as_deref()
            );
        }
    }    
}