import os
import sys
import time
import math
import subprocess
import json
import random
import signal
import urllib.request

NATIVE_PORT = 5005
MEDIATR_PORT = 5020
PURENATIVE_PORT = 5030
DOTNET_PATH = os.path.expanduser("~/.dotnet/dotnet")

def fetch_json(url, timeout=5):
    try:
        req = urllib.request.Request(url, headers={"Accept": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            if resp.status == 200:
                return json.loads(resp.read().decode('utf-8'))
    except Exception:
        pass
    return None

def trigger_post(url, payload=None, timeout=5):
    try:
        data = json.dumps(payload).encode('utf-8') if payload else b""
        req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status == 200
    except Exception:
        return False

def wait_for_endpoint(url, timeout_sec=15):
    start = time.time()
    while time.time() - start < timeout_sec:
        if fetch_json(url) is not None or trigger_post(url, {"Message": "ping"}):
            return True
        time.sleep(0.3)
    return False

def force_gc_collect(port):
    trigger_post(f"http://localhost:{port}/api/Benchmark/gc-collect")
    time.sleep(0.5)

def get_gc_stats(port):
    res = fetch_json(f"http://localhost:{port}/api/Benchmark/gc-stats")
    if res:
        return {
            "allocated_bytes": res.get("totalAllocatedBytes", 0),
            "gen0": res.get("gen0", 0),
            "gen1": res.get("gen1", 0),
            "gen2": res.get("gen2", 0),
            "working_set_mb": res.get("workingSetMb", 0.0)
        }
    return {"allocated_bytes": 0, "gen0": 0, "gen1": 0, "gen2": 0, "working_set_mb": 0.0}

def warm_up_endpoint(target_url, method, payload=None):
    # Short 1-second warmup run using k6
    js = f"""
import http from 'k6/http';
export const options = {{ vus: 10, duration: '1s' }};
const payload = {json.dumps(payload) if payload else 'null'};
const params = {{ headers: {{ 'Content-Type': 'application/json' }} }};
export default function() {{
  if ('{method}' === 'POST') {{
    http.post('{target_url}', JSON.stringify(payload), params);
  }} else {{
    http.get('{target_url}');
  }}
}}
"""
    tmp_js = "/tmp/k6_warmup.js"
    with open(tmp_js, "w") as f:
        f.write(js)
    cmd = ["taskset", "-c", "8-15", "/usr/bin/k6", "run", "--quiet", tmp_js]
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def run_k6_scenario(scenario_name, target_url, method, payload, vus, duration_sec):
    summary_json = f"/tmp/k6_summary_{scenario_name}_{vus}vus_{time.time_ns()}.json"
    
    js_content = f"""
import http from 'k6/http';
import {{ check }} from 'k6';

export const options = {{
  vus: {vus},
  duration: '{duration_sec}s',
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)'],
}};

const payload = {json.dumps(payload) if payload else 'null'};
const params = {{ headers: {{ 'Content-Type': 'application/json' }} }};

export default function() {{
  let res;
  if ('{method}' === 'POST') {{
    res = http.post('{target_url}', JSON.stringify(payload), params);
  }} else {{
    res = http.get('{target_url}');
  }}
  check(res, {{ 'status is 200': (r) => r.status === 200 }});
}}

export function handleSummary(data) {{
  return {{
    '{summary_json}': JSON.stringify(data)
  }};
}}
"""
    tmp_js = f"/tmp/k6_{scenario_name}_{vus}vus.js"
    with open(tmp_js, "w") as f:
        f.write(js_content)
        
    cmd = ["taskset", "-c", "8-15", "/usr/bin/k6", "run", tmp_js]
    
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = proc.communicate()
    
    results = {
        "rps": 0.0,
        "avg_ms": 0.0,
        "p95_ms": 0.0,
        "p99_ms": 0.0,
        "failures": 0,
        "total_reqs": 0
    }
    
    if os.path.exists(summary_json):
        try:
            with open(summary_json, "r") as f:
                data = json.load(f)
                metrics = data.get("metrics", {})
                reqs = metrics.get("http_reqs", {}).get("values", {})
                dur = metrics.get("http_req_duration", {}).get("values", {})
                fails = metrics.get("http_req_failed", {}).get("values", {})
                
                results["rps"] = reqs.get("rate", 0.0)
                results["total_reqs"] = int(reqs.get("count", 0))
                results["avg_ms"] = dur.get("avg", 0.0)
                results["p95_ms"] = dur.get("p(95)", 0.0)
                results["p99_ms"] = dur.get("p(99)", 0.0)
                results["failures"] = int(fails.get("passes", 0))
            os.remove(summary_json)
        except Exception as ex:
            print(f"Error parsing k6 summary: {ex}")
            
    return results

def compute_stats(values):
    if not values:
        return 0.0, 0.0
    sorted_vals = sorted(values)
    n = len(sorted_vals)
    median = sorted_vals[n // 2]
    mean = sum(values) / n
    variance = sum((x - mean) ** 2 for x in values) / n
    stddev = math.sqrt(variance)
    return median, stddev

def graceful_shutdown(proc, name):
    if proc and proc.poll() is None:
        try:
            print(f"[CLEANUP] Gracefully stopping {name} (SIGINT)...")
            proc.send_signal(signal.SIGINT)
            proc.wait(timeout=3)
        except Exception:
            print(f"[CLEANUP] Force terminating {name}...")
            proc.terminate()

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    
    print("==========================================================================")
    print(" RIGOROUS BENCHMARK SUITE: CPU PINNING + 5 REPEATS (MEDIAN/STDDEV) + GC ALLOC")
    print(" NativeApp (Cores 0-3, Port 5005) vs MediatRApp (Cores 4-7, Port 5020)")
    print(" vs PureNativeApp (Cores 0-3, Port 5030 - Dispatcher-less)")
    print(" k6 Client Load Generator (Cores 8-15)")
    print("==========================================================================")
    
    native_env = os.environ.copy()
    native_env["DOTNET_ROOT"] = os.path.expanduser("~/.dotnet")
    mediatr_env = os.environ.copy()
    mediatr_env["DOTNET_ROOT"] = os.path.expanduser("~/.dotnet")
    purenative_env = os.environ.copy()
    purenative_env["DOTNET_ROOT"] = os.path.expanduser("~/.dotnet")

    # 1. Launch NativeApp on Cores 0-3
    native_cmd = ["taskset", "-c", "0-3", DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{NATIVE_PORT}"]
    print(f"[LAUNCH] Starting NativeApp on Cores 0-3 (port {NATIVE_PORT})...")
    native_proc = subprocess.Popen(native_cmd, cwd=os.path.join(root_dir, "NativeApp"), env=native_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    # 2. Launch MediatRApp on Cores 4-7
    mediatr_cmd = ["taskset", "-c", "4-7", DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{MEDIATR_PORT}"]
    print(f"[LAUNCH] Starting MediatRApp on Cores 4-7 (port {MEDIATR_PORT})...")
    mediatr_proc = subprocess.Popen(mediatr_cmd, cwd=os.path.join(root_dir, "MediatRApp"), env=mediatr_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    # 3. Launch PureNativeApp on Cores 0-3
    purenative_cmd = ["taskset", "-c", "0-3", DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{PURENATIVE_PORT}"]
    print(f"[LAUNCH] Starting PureNativeApp on Cores 0-3 (port {PURENATIVE_PORT})...")
    purenative_proc = subprocess.Popen(purenative_cmd, cwd=os.path.join(root_dir, "PureNativeApp"), env=purenative_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    try:
        print("[LAUNCH] Waiting for Web API servers to respond...")
        native_ok = wait_for_endpoint(f"http://localhost:{NATIVE_PORT}/api/Benchmark/query?id=1")
        mediatr_ok = wait_for_endpoint(f"http://localhost:{MEDIATR_PORT}/api/Benchmark/query?id=1")
        purenative_ok = wait_for_endpoint(f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/query?id=1")
        
        if not native_ok or not mediatr_ok or not purenative_ok:
            print(f"[ERROR] Connection failed! Native: {native_ok}, MediatR: {mediatr_ok}, PureNative: {purenative_ok}")
            return
        
        print("[LAUNCH] All servers are UP and ready.")
        
        vus_levels = [50, 100, 200, 500]
        duration_sec = 4
        repeats = 5
        
        scenarios_def = [
            ("NativeApp Command", f"http://localhost:{NATIVE_PORT}/api/Benchmark/command", "POST", {"Message": "RigorousTest"}, NATIVE_PORT),
            ("MediatRApp Command", f"http://localhost:{MEDIATR_PORT}/api/Benchmark/command", "POST", {"Message": "RigorousTest"}, MEDIATR_PORT),
            ("PureNativeApp Command", f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/command", "POST", {"Message": "RigorousTest"}, PURENATIVE_PORT),
            ("NativeApp Query", f"http://localhost:{NATIVE_PORT}/api/Benchmark/query?id=999", "GET", None, NATIVE_PORT),
            ("MediatRApp Query", f"http://localhost:{MEDIATR_PORT}/api/Benchmark/query?id=999", "GET", None, MEDIATR_PORT),
            ("PureNativeApp Query", f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/query?id=999", "GET", None, PURENATIVE_PORT)
        ]
        
        # Store all raw repeat data
        raw_data = { (sc[0], vus): [] for sc in scenarios_def for vus in vus_levels }
        
        print(f"\n==========================================================================")
        print(f" EXECUTING {repeats} REPEAT ROUNDS (INTERLEAVED SCENARIO ORDERING)")
        print(f"==========================================================================")
        
        for r in range(1, repeats + 1):
            print(f"\n--- REPEAT ROUND {r}/{repeats} ---")
            
            for vus in vus_levels:
                # Interleave / shuffle scenario order per round
                round_scenarios = list(scenarios_def)
                random.shuffle(round_scenarios)
                
                for sc_name, url, method, payload, port in round_scenarios:
                    # Endpoint-specific warmup
                    warm_up_endpoint(url, method, payload)
                    
                    # Force GC cleanup
                    force_gc_collect(port)
                    
                    # Read baseline GC stats
                    gc_before = get_gc_stats(port)
                    
                    # Run k6 scenario
                    k6_res = run_k6_scenario(sc_name.replace(" ", "_"), url, method, payload, vus, duration_sec)
                    
                    # Read end GC stats
                    gc_after = get_gc_stats(port)
                    
                    delta_alloc = max(0, gc_after["allocated_bytes"] - gc_before["allocated_bytes"])
                    delta_gen0 = max(0, gc_after["gen0"] - gc_before["gen0"])
                    total_reqs = max(1, k6_res["total_reqs"])
                    bytes_per_req = delta_alloc / total_reqs
                    
                    item = {
                        "rps": k6_res["rps"],
                        "avg_ms": k6_res["avg_ms"],
                        "p95_ms": k6_res["p95_ms"],
                        "p99_ms": k6_res["p99_ms"],
                        "failures": k6_res["failures"],
                        "ram_mb": gc_after["working_set_mb"],
                        "bytes_per_req": bytes_per_req,
                        "gen0_count": delta_gen0
                    }
                    raw_data[(sc_name, vus)].append(item)
                    print(f"  Round {r} | {sc_name:<22} | VUs={vus:3d} | RPS={item['rps']:8.0f} | P95={item['p95_ms']:5.2f}ms | P99={item['p99_ms']:5.2f}ms | Alloc/Op={item['bytes_per_req']:6.1f} B | Gen0={item['gen0_count']:2d} | RAM={item['ram_mb']:5.1f}MB")

        # Aggregate Results
        aggregated_results = []
        for sc_name, url, method, payload, port in scenarios_def:
            for vus in vus_levels:
                runs = raw_data[(sc_name, vus)]
                rps_list = [x["rps"] for x in runs]
                p95_list = [x["p95_ms"] for x in runs]
                p99_list = [x["p99_ms"] for x in runs]
                bytes_list = [x["bytes_per_req"] for x in runs]
                gen0_list = [x["gen0_count"] for x in runs]
                ram_list = [x["ram_mb"] for x in runs]
                fails_list = [x["failures"] for x in runs]
                
                med_rps, std_rps = compute_stats(rps_list)
                med_p95, _ = compute_stats(p95_list)
                med_p99, _ = compute_stats(p99_list)
                med_bytes, _ = compute_stats(bytes_list)
                med_gen0, _ = compute_stats(gen0_list)
                med_ram, _ = compute_stats(ram_list)
                
                aggregated_results.append({
                    "scenario": sc_name,
                    "vus": vus,
                    "median_rps": med_rps,
                    "stddev_rps": std_rps,
                    "median_p95_ms": med_p95,
                    "median_p99_ms": med_p99,
                    "max_failures": max(fails_list),
                    "bytes_per_req": med_bytes,
                    "gen0_collections": med_gen0,
                    "median_ram_mb": med_ram
                })

        # Print Markdown Table
        print("\n" + "="*145)
        print(" RIGOROUS BENCHMARK RESULTS (5-REPEAT MEDIAN ± STDDEV & MANAGED GC ALLOC/OP)")
        print("="*145)
        print(f"| {'Scenario':<22} | {'VUs':<5} | {'Median RPS ± StdDev':<24} | {'P95 (ms)':<9} | {'P99 (ms)':<9} | {'Alloc/Op (Bytes/req)':<22} | {'Gen0 GCs':<9} | {'RAM (MB)':<10} |")
        print("|" + "-"*24 + "|" + "-"*7 + "|" + "-"*26 + "|" + "-"*11 + "|" + "-"*11 + "|" + "-"*24 + "|" + "-"*11 + "|" + "-"*12 + "|")
        for r in aggregated_results:
            rps_str = f"{r['median_rps']:,.0f} ± {r['stddev_rps']:,.0f}"
            print(f"| {r['scenario']:<22} | {r['vus']:<5d} | {rps_str:<24} | {r['median_p95_ms']:<9.2f} | {r['median_p99_ms']:<9.2f} | {r['bytes_per_req']:<22.1f} | {r['gen0_collections']:<9.0f} | {r['median_ram_mb']:<10.1f} |")
        print("="*145)
        
        # Save JSON output
        out_path = os.path.join(root_dir, "tests", "benchmark_rigorous_results.json")
        with open(out_path, "w") as f:
            json.dump(aggregated_results, f, indent=2)
        print(f"\nRigorous benchmark results saved to {out_path}")

    finally:
        print("\n[CLEANUP] Stopping Web API server processes...")
        graceful_shutdown(native_proc, "NativeApp")
        graceful_shutdown(mediatr_proc, "MediatRApp")
        graceful_shutdown(purenative_proc, "PureNativeApp")

if __name__ == "__main__":
    main()
