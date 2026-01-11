use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricPoint {
    pub timestamp: f64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryRequest {
    pub pod_name: String,
    pub metrics: Vec<MetricPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub pod_name: String,
    pub metrics: Vec<MetricPoint>,
}