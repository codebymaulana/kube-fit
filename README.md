# kube-fit 🚀 - Kubernetes Pod Right-Sizing Analyzer

**kube-fit** is a lightweight, CLI tool written in Rust. It connects to your Prometheus server, fetches historical usage data, and compares it against your Kubernetes resource requests to identify over-provisioned (wasted money) or under-provisioned (risk of OOM) pods. based on my personal experience operating Kubernetes workloads to help on right-size your pods by comparing resource requests vs. actual usage

## ✨ Features
> [!WARNING]
> **Performance Note**: This tool has not yet been benchmarked against clusters with a high volume of pods. Use with caution in production environments with thousands of active pods.

- **⚡ Fast & Async**: Built with Rust, Tokio, and Clap.
- **📊 Range Analysis**: Analyzes data over a specific time window (e.g., 7 days) rather than just "now".
- **🔍 Smart Categorization**: Automatically flags pods as `CRITICAL` (High Usage), `IDLE` (Low Usage), or `Normal`.
- **📈 Backends**: Works with standard Prometheus (At this moment)
- **📋 Data**: Only memory (At this moment)


## 🛠 Installation

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install) installed (1.92.0).

### Build from Source
```bash
# Clone the repository
git clone https://github.com/codebymaulana/kube-fit.git
cd kube-fit

# Build release binary
cargo build --release

# Run
./target/release/kube-fit --help
```

### Download the binary
You can use the binary in bin folder. 

### 🚀 Usage
Ensure you have a Prometheus instance accessible (use kubectl port-forward if needed).
```
$ kube-fit --metric-server=http://prometheus.fajar --interval=15m
```

### Arguments
Argument | Description | Default | Example | Required 
--- | --- | --- | --- |---
--metric-server | URL of your Prometheus server | Null | http://localhost:9090 | True
--interval | Internal time for analysis in format m,h,d | Null | 15m | True
--filter | Filter data based on the status | Null | "Normal", "Underutilized", "Overutilized" | false
--namespace | Filter data by kubernetes namespace | Null | kube-system | false



### 📊 How It Works
The tool executes specific PromQL queries to fetch Requests (what you asked K8s for) and Usage (what the app actually used).

Logic Steps
1. Fetch Data: Queries Prometheus for resource requests and usage over the specified --interval.
2. Align Time Series: Matches timestamped usage data with the corresponding request configuration.
3. Compare: Calculates the percentage of requested resources actually being utilized.
4. Categorize:
Overutilized (>90%): Pod is struggling; requests might be too low.
Underutilized (<10%): Pod is over-provisioned; you are wasting resources.
Normal: Healthy utilization.

### PromQL Queries Used
Memory Request
```
max(kube_pod_container_resource_requests{resource="memory"}) by (pod)
```

Memory Used
```
max(container_memory_working_set_bytes{name!=""}) by (pod)
```

### 📝 Example Output
```
-------------------------------------Pod Usage Analyzer(all)[5]-------------------------------------
POD NAME                                                STATUS                     AVG USAGE
app-with-limits-b568c767d-sxjcz                         Underutilized                  2.00%
coredns-66bc5c9577-j77th                                Normal                        62.64%
coredns-66bc5c9577-stgbt                                Normal                        68.43%
etcd-master                                             Overutilized                 102.08%
metrics-server-7d694f9fb5-l9ncl                         Normal                        34.75%
```

🤝 Contributing
Contributions are welcome!

👤 Author

This project is primarily designed and implemented by the author.
AI tools were used occasionally for code suggestions and refactoring, similar to using documentation or IDE assistance.

GitHub: @codebymaulana