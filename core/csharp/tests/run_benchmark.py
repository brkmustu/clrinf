import os
import sys
import time
import subprocess
import json
import urllib.request

NATIVE_PORT = 5005
MEDIATR_PORT = 5020
PURENATIVE_PORT = 5030
DOTNET_PATH = os.path.expanduser("~/.dotnet/dotnet")

def get_process_memory_mb(pid):
    try:
        with open(f"/proc/{pid}/statm", "r") as f:
            fields = f.read().split()
            # resident pages * page_size (4096) / 1024 / 1024
            res_pages = int(fields[1])
            return (res_pages * 4096) / (1024 * 1024)
    except Exception:
        return 0.0

def wait_for_endpoint(url, timeout_sec=15):
    start = time.time()
    while time.time() - start < timeout_sec:
        try:
            req = urllib.request.Request(url)
            with urllib.request.urlopen(req) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(0.3)
    return False

def warm_up():
    print("[WARMUP] Sending initial requests to warm up endpoints...")
    endpoints = [
        f"http://localhost:{NATIVE_PORT}/api/Benchmark/query?id=1",
        f"http://localhost:{MEDIATR_PORT}/api/Benchmark/query?id=1",
        f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/query?id=1"
    ]
    for ep in endpoints:
        try:
            urllib.request.urlopen(ep, timeout=2)
        except Exception as e:
            print(f"[WARMUP] Error warming up {ep}: {e}")

def run_k6_scenario(scenario_name, target_url, method, payload, vus, duration_sec):
    env = os.environ.copy()
    env["PATH"] = os.path.expanduser("~/.dotnet") + ":" + env.get("PATH", "")
    
    summary_json = f"/tmp/k6_summary_{scenario_name}_{vus}vus.json"
    
    # We create a temporary script for the single scenario with handleSummary
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
        
    cmd = ["/usr/bin/k6", "run", tmp_js]
    
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = proc.communicate()
    
    # Read summary json
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
                results["total_reqs"] = reqs.get("count", 0)
                results["avg_ms"] = dur.get("avg", 0.0)
                results["p95_ms"] = dur.get("p(95)", 0.0)
                results["p99_ms"] = dur.get("p(99)", 0.0)
                results["failures"] = fails.get("passes", 0)
        except Exception as ex:
            print(f"Error parsing k6 summary: {ex}")
    else:
        if err:
            print(f"[K6 ERROR] {err.strip()}")
            
    return results


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    
    print("==========================================================================")
    print(" K6 LOAD TEST SUITE: NativeApp vs MediatRApp vs PureNativeApp (Dispatcher-less)")
    print("==========================================================================")
    
    # 1. Start NativeApp
    native_cmd = [DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{NATIVE_PORT}"]
    print(f"[LAUNCH] Starting NativeApp on port {NATIVE_PORT}...")
    native_proc = subprocess.Popen(native_cmd, cwd=os.path.join(root_dir, "NativeApp"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    # 2. Start MediatRApp
    mediatr_cmd = [DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{MEDIATR_PORT}"]
    print(f"[LAUNCH] Starting MediatRApp on port {MEDIATR_PORT}...")
    mediatr_proc = subprocess.Popen(mediatr_cmd, cwd=os.path.join(root_dir, "MediatRApp"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    # 3. Start PureNativeApp
    purenative_cmd = [DOTNET_PATH, "run", "-c", "Release", "--urls", f"http://localhost:{PURENATIVE_PORT}"]
    print(f"[LAUNCH] Starting PureNativeApp on port {PURENATIVE_PORT}...")
    purenative_proc = subprocess.Popen(purenative_cmd, cwd=os.path.join(root_dir, "PureNativeApp"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    try:
        print("[LAUNCH] Waiting for Web API servers to be ready...")
        native_ok = wait_for_endpoint(f"http://localhost:{NATIVE_PORT}/api/Benchmark/query?id=1")
        mediatr_ok = wait_for_endpoint(f"http://localhost:{MEDIATR_PORT}/api/Benchmark/query?id=1")
        purenative_ok = wait_for_endpoint(f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/query?id=1")
        
        if not native_ok or not mediatr_ok or not purenative_ok:
            print(f"[ERROR] Failed to connect to servers! Native: {native_ok}, MediatR: {mediatr_ok}, PureNative: {purenative_ok}")
            return
        
        print("[LAUNCH] All Web API servers are UP and listening.")
        warm_up()
        time.sleep(1)
        
        vus_levels = [50, 100, 200, 500]
        duration_sec = 5
        
        all_results = []
        
        scenarios = [
            ("NativeApp Command", f"http://localhost:{NATIVE_PORT}/api/Benchmark/command", "POST", {"Message": "K6LoadTest"}, native_proc.pid),
            ("MediatRApp Command", f"http://localhost:{MEDIATR_PORT}/api/Benchmark/command", "POST", {"Message": "K6LoadTest"}, mediatr_proc.pid),
            ("PureNativeApp Command", f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/command", "POST", {"Message": "K6LoadTest"}, purenative_proc.pid),
            ("NativeApp Query", f"http://localhost:{NATIVE_PORT}/api/Benchmark/query?id=999", "GET", None, native_proc.pid),
            ("MediatRApp Query", f"http://localhost:{MEDIATR_PORT}/api/Benchmark/query?id=999", "GET", None, mediatr_proc.pid),
            ("PureNativeApp Query", f"http://localhost:{PURENATIVE_PORT}/api/Benchmark/query?id=999", "GET", None, purenative_proc.pid)
        ]
        
        for sc_name, url, method, body, app_pid in scenarios:
            print(f"\n--- Testing Scenario: {sc_name} ---")
            for vus in vus_levels:
                mem_before = get_process_memory_mb(app_pid)
                k6_res = run_k6_scenario(sc_name.replace(" ", "_"), url, method, body, vus, duration_sec)
                mem_after = get_process_memory_mb(app_pid)
                
                row = {
                    "scenario": sc_name,
                    "vus": vus,
                    "rps": k6_res["rps"],
                    "avg_ms": k6_res["avg_ms"],
                    "p95_ms": k6_res["p95_ms"],
                    "p99_ms": k6_res["p99_ms"],
                    "failures": k6_res["failures"],
                    "ram_mb": mem_after
                }
                all_results.append(row)
                print(f"  VUs={vus:3d} | RPS={row['rps']:9.0f} | Mean={row['avg_ms']:6.2f}ms | P95={row['p95_ms']:6.2f}ms | P99={row['p99_ms']:6.2f}ms | Fail={row['failures']} | RAM={row['ram_mb']:6.1f}MB")
                time.sleep(0.5)

        # Print Markdown Table
        print("\n" + "="*115)
        print(f"| {'Scenario':<22} | {'VUs':<6} | {'RPS':<10} | {'Mean (ms)':<10} | {'P95 (ms)':<9} | {'P99 (ms)':<9} | {'Failures':<9} | {'RAM (MB)':<10} |")
        print("|" + "-"*24 + "|" + "-"*8 + "|" + "-"*12 + "|" + "-"*12 + "|" + "-"*11 + "|" + "-"*11 + "|" + "-"*11 + "|" + "-"*12 + "|")
        for r in all_results:
            print(f"| {r['scenario']:<22} | {r['vus']:<6d} | {r['rps']:<10.0f} | {r['avg_ms']:<10.2f} | {r['p95_ms']:<9.2f} | {r['p99_ms']:<9.2f} | {r['failures']:<9d} | {r['ram_mb']:<10.1f} |")
        print("="*115)
        
        # Save JSON output
        out_path = os.path.join(root_dir, "tests", "benchmark_results_baseline.json")
        if os.path.exists(os.path.join(root_dir, "tests", "benchmark_results_baseline.json")):
            out_path = os.path.join(root_dir, "tests", "benchmark_results_optimized.json")
            
        with open(out_path, "w") as f:
            json.dump(all_results, f, indent=2)
        print(f"\nResults saved to {out_path}")

    finally:
        print("\n[CLEANUP] Terminating Web API server processes...")
        native_proc.terminate()
        mediatr_proc.terminate()
        purenative_proc.terminate()

if __name__ == "__main__":
    main()
