import json
import matplotlib.pyplot as plt
import os
from typing import Dict, List, Tuple
import re
from datetime import datetime
from pathlib import Path
from collections import defaultdict


def find_all_benchmarks() -> List[Path]:
    """Find all benchmark JSON files"""
    benchmark_dir = Path(".benchmarks")
    if not benchmark_dir.exists():
        raise FileNotFoundError(".benchmarks directory not found")

    # Get all benchmark files, excluding temporary files
    json_files = []
    for dir_path in benchmark_dir.iterdir():
        if dir_path.is_dir():
            files = [f for f in dir_path.glob("*.json")]
            json_files.extend(files)

    if not json_files:
        raise FileNotFoundError("No benchmark files found")

    return sorted(json_files, key=lambda x: x.stat().st_mtime)


def load_benchmark_data(file_path: Path) -> Dict:
    """Load benchmark data from JSON file"""
    with open(file_path, "r") as f:
        return json.load(f)


def format_time(seconds: float) -> str:
    """Format time in appropriate units"""
    if seconds < 0.001:
        return f"{seconds*1000000:.2f}µs"
    elif seconds < 1:
        return f"{seconds*1000:.2f}ms"
    else:
        return f"{seconds:.2f}s"


def create_historical_performance_graph(benchmark_files: List[Path]) -> str:
    """Create graph showing performance trends over time"""
    # Collect historical data
    history = defaultdict(lambda: {"dates": [], "means": [], "stddevs": []})

    for file_path in benchmark_files:
        data = load_benchmark_data(file_path)
        date = datetime.fromisoformat(data["datetime"].replace("Z", "+00:00"))

        for bench in data["benchmarks"]:
            name = bench["name"].replace("test_", "").replace("_", " ").title()
            history[name]["dates"].append(date)
            history[name]["means"].append(bench["stats"]["mean"])
            history[name]["stddevs"].append(bench["stats"]["stddev"])

    # Create plot
    plt.figure(figsize=(8, 6), dpi=100)

    for name, data in history.items():
        plt.errorbar(
            data["dates"],
            data["means"],
            yerr=data["stddevs"],
            label=name,
            marker="o",
            markersize=4,
            capsize=3,
        )

    plt.title("Performance History")
    plt.xlabel("Date")
    plt.ylabel("Time (seconds)")
    plt.legend()
    plt.grid(True, linestyle="--", alpha=0.7)

    # Rotate dates for better readability
    plt.gcf().autofmt_xdate()

    plt.tight_layout()

    # Save graph
    graph_path = "docs/images/benchmark_history.svg"
    os.makedirs(os.path.dirname(graph_path), exist_ok=True)
    plt.savefig(graph_path, format="svg")
    plt.close()

    return graph_path


def create_ops_history_graph(benchmark_files: List[Path]) -> str:
    """Create graph showing operations per second trends over time"""
    # Collect historical data
    history = defaultdict(lambda: {"dates": [], "ops": []})

    for file_path in benchmark_files:
        data = load_benchmark_data(file_path)
        date = datetime.fromisoformat(data["datetime"].replace("Z", "+00:00"))

        for bench in data["benchmarks"]:
            name = bench["name"].replace("test_", "").replace("_", " ").title()
            history[name]["dates"].append(date)
            history[name]["ops"].append(bench["stats"]["ops"])

    plt.figure(figsize=(8, 6), dpi=100)

    for name, data in history.items():
        plt.plot(data["dates"], data["ops"], label=name, marker="o", markersize=4)

    plt.title("Operations Per Second History")
    plt.xlabel("Date")
    plt.ylabel("Operations/Second")
    plt.legend()
    plt.grid(True, linestyle="--", alpha=0.7)

    plt.gcf().autofmt_xdate()

    plt.tight_layout()

    graph_path = "docs/images/benchmark_ops_history.svg"
    os.makedirs(os.path.dirname(graph_path), exist_ok=True)
    plt.savefig(graph_path, format="svg")
    plt.close()

    return graph_path


def create_benchmark_summary(benchmark_files: List[Path]) -> str:
    """Create markdown table summarizing benchmark history"""
    latest_data = load_benchmark_data(benchmark_files[-1])
    earliest_data = load_benchmark_data(benchmark_files[0])

    rows = []
    rows.append("| Test | Current | Min (Historical) | Max (Historical) | Trend |")
    rows.append("|------|---------|-----------------|------------------|-------|")

    # Collect historical stats
    history = defaultdict(lambda: {"means": []})
    for file_path in benchmark_files:
        data = load_benchmark_data(file_path)
        for bench in data["benchmarks"]:
            name = bench["name"].replace("test_", "").replace("_", " ").title()
            history[name]["means"].append(bench["stats"]["mean"])

    # Create summary rows
    for bench in latest_data["benchmarks"]:
        name = bench["name"].replace("test_", "").replace("_", " ").title()
        current = bench["stats"]["mean"]
        historical_means = history[name]["means"]

        # Calculate trend
        if len(historical_means) > 1:
            first_mean = historical_means[0]
            last_mean = historical_means[-1]
            if last_mean < first_mean:
                trend = "📈 Improved"
            elif last_mean > first_mean:
                trend = "📉 Declined"
            else:
                trend = "📊 Stable"
        else:
            trend = "📊 Single data point"

        row = "| {} | {} | {} | {} | {} |".format(
            name,
            format_time(current),
            format_time(min(historical_means)),
            format_time(max(historical_means)),
            trend,
        )
        rows.append(row)

    return "\n".join(rows)


def update_readme(
    benchmark_files: List[Path], perf_history: str, ops_history: str
) -> None:
    """Update README.md with benchmark results"""
    latest_data = load_benchmark_data(benchmark_files[-1])
    earliest_date = datetime.fromisoformat(
        load_benchmark_data(benchmark_files[0])["datetime"].replace("Z", "+00:00")
    )
    latest_date = datetime.fromisoformat(latest_data["datetime"].replace("Z", "+00:00"))

    benchmark_section = f"""## Benchmark Results

Benchmark history from {earliest_date.strftime('%Y-%m-%d')} to {latest_date.strftime('%Y-%m-%d')}

### Summary

{create_benchmark_summary(benchmark_files)}

### Latest System Information

- CPU: {latest_data["machine_info"]["cpu"]["brand_raw"]}
- Architecture: {latest_data["machine_info"]["cpu"]["arch_string_raw"]}
- Cores: {latest_data["machine_info"]["cpu"]["count"]}
- Python: {latest_data["machine_info"]["python_version"]}

### Performance History

![Performance History](./{perf_history})

### Operations Per Second History

![Operations History](./{ops_history})

*Note: Performance may vary based on system specifications and load.*
"""

    try:
        with open("README.md", "r") as f:
            content = f.read()

        # Find the last occurrence of "## Benchmark Results"
        start_idx = content.rfind("## Benchmark Results")
        if start_idx != -1:
            # Keep content up to the benchmark section
            content = content[:start_idx] + benchmark_section
        else:
            # No existing benchmark section, append to end
            content += f"\n\n{benchmark_section}"

        with open("README.md", "w") as f:
            f.write(content)

    except FileNotFoundError:
        print("README.md not found")


def main():
    try:
        # Find all benchmark files
        benchmark_files = find_all_benchmarks()

        # Create historical graphs
        perf_history = create_historical_performance_graph(benchmark_files)
        ops_history = create_ops_history_graph(benchmark_files)

        # Update README
        update_readme(benchmark_files, perf_history, ops_history)

        print(
            f"Benchmark report generated successfully from {len(benchmark_files)} benchmark files!"
        )

    except Exception as e:
        print(f"Error generating benchmark report: {e}")
        raise


if __name__ == "__main__":
    main()
