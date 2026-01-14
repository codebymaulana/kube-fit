use std::collections::HashMap;
use crate::models::{MemoryRequest, MemoryUsage};
use colored::Colorize;

pub fn compare_data(requests: Vec<MemoryRequest>, usages: Vec<MemoryUsage>, filter: Option<&str>) {
    // 1. Define Column Widths
    const COL_POD: usize = 55;
    const COL_STATUS: usize = 20;
    const COL_USAGE: usize = 15;

    // 2. Prepare Data
    let mut rows_to_print = Vec::new();
    let mut request_map: HashMap<String, Vec<crate::models::MetricPoint>> = HashMap::new();
    
    for req in requests {
        request_map.insert(req.pod_name, req.metrics);
    }

    for usage in usages {
        if let Some(req_metrics) = request_map.get(&usage.pod_name) {
            let mut total_percentage = 0.0;
            let mut count = 0.0;

            for (u_point, r_point) in usage.metrics.iter().zip(req_metrics.iter()) {
                if r_point.value > 0.0 {
                    total_percentage += (u_point.value / r_point.value) * 100.0;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                let avg = total_percentage / count;
                
                let status_raw = if avg >= 90.0 { "Overutilized" } 
                                 else if avg <= 10.0 { "Underutilized" } 
                                 else { "Normal" };

                if let Some(user_filter) = filter {
                    if status_raw.to_uppercase() != user_filter.to_uppercase() {
                        continue;
                    }
                }

                rows_to_print.push((usage.pod_name, status_raw, avg));
            }
        }
    }

    // 3. PRINTING
    let title = format!("Pod Usage Analyzer(all)[{}]", rows_to_print.len());
    println!("{}", format!("{:-^100}", title).cyan()); 

    println!(
        "{:<w1$} {:<w2$} {:>w3$}", 
        "POD NAME".cyan().bold(), 
        "STATUS".cyan().bold(), 
        "AVG USAGE".cyan().bold(),
        w1 = COL_POD, 
        w2 = COL_STATUS, 
        w3 = COL_USAGE
    );

    for (pod_name, status_raw, avg) in rows_to_print {
        
        // Apply color to ALL columns based on status
        let (pod_display, status_display, avg_display) = match status_raw {
            "Overutilized" => (
                format!("{:<width$}", pod_name, width = COL_POD).red().bold(),
                format!("{:<width$}", status_raw, width = COL_STATUS).red().bold(),
                format!("{:>width$.2}%", avg, width = COL_USAGE - 1).red().bold(),
            ),
            "Underutilized" => (
                format!("{:<width$}", pod_name, width = COL_POD).yellow().bold(),
                format!("{:<width$}", status_raw, width = COL_STATUS).yellow().bold(),
                format!("{:>width$.2}%", avg, width = COL_USAGE - 1).yellow().bold(),
            ),
            _ => (
                format!("{:<width$}", pod_name, width = COL_POD).green(),
                format!("{:<width$}", status_raw, width = COL_STATUS).green(),
                format!("{:>width$.2}%", avg, width = COL_USAGE - 1).green(),
            ),
        };

        println!(
            "{} {} {}", 
            pod_display, 
            status_display, 
            avg_display
        );
    }
}