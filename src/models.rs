// src/models.rs

#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryRequest {
    pub pod_name: String,
    pub namespace: String, // <--- Add this
    pub metrics: Vec<MetricPoint>,
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub pod_name: String,
    pub namespace: String, // <--- Add this
    pub metrics: Vec<MetricPoint>,
}