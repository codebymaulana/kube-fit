use std::collections::HashMap;
use crate::models::{MemoryRequest, MemoryUsage};

pub fn compare_data(requests: Vec<MemoryRequest>, usages: Vec<MemoryUsage>) {
    println!("\n####################################################################################################");
    println!("#########                               Pod Usage Analyzer                                 #########");
    println!("####################################################################################################");
    
    // Table Header
    println!("| {:<50} | {:<25} | {:>13} |", "Pod Name", "Status", "Avg Usage (%)");
    println!("|{:-<52}|{:-<27}|{:-<15}|", "", "", "");

    let mut request_map: HashMap<String, Vec<crate::models::MetricPoint>> = HashMap::new();
    for req in requests {
        request_map.insert(req.pod_name, req.metrics);
    }

    // Process Compare Data
    for usage in usages {
        if let Some(req_metrics) = request_map.get(&usage.pod_name) {
            
            let mut total_percentage = 0.0;
            let mut count = 0.0;

            for (u_point, r_point) in usage.metrics.iter().zip(req_metrics.iter()) {
                if r_point.value > 0.0 {
                    let percent = (u_point.value / r_point.value) * 100.0;
                    total_percentage += percent;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                let avg_usage_percent = total_percentage / count;
                // Categorize Data
                let status = if avg_usage_percent >= 90.0 {
                    "CRITICAL (High)"
                } else if avg_usage_percent <= 10.0 {
                    "IDLE (Low)"
                } else {
                    "Normal"
                };

                // Print row
                println!(
                    "| {:<50} | {:<25} | {:>12.2}% |", 
                    usage.pod_name, 
                    status, 
                    avg_usage_percent
                );
            }
        }
    }
    // Print Footer
    println!("|{:-<52}|{:-<27}|{:-<15}|", "", "", "");
}