#!/usr/bin/env python3
"""Load generator for Blinker's Solana Action endpoints (stdlib only).

Hammers GET /api/actions/<id> (what wallets/unfurlers call) and, optionally,
POST /api/actions/<id>?amount=… (builds a transaction), with N keep-alive
connections in threads. Reports throughput and latency percentiles, plus the
server's own CPU/RSS/queue numbers from /api/system before and after.

    python3 scripts/loadtest.py --blink <id> --seconds 15 --conns 32 --post-ratio 0.2
"""
import argparse
import http.client
import json
import statistics
import threading
import time
import urllib.parse


def system(host, port):
    c = http.client.HTTPConnection(host, port, timeout=5)
    c.request("GET", "/api/system")
    return json.loads(c.getresponse().read())


def worker(host, port, blink, account, deadline, post_ratio, out, idx):
    conn = http.client.HTTPConnection(host, port, timeout=10)
    lat_get, lat_post, errors, n = [], [], 0, 0
    body = json.dumps({"account": account})
    while time.perf_counter() < deadline:
        n += 1
        is_post = post_ratio > 0 and (n * 7919 + idx) % 1000 < post_ratio * 1000
        t = time.perf_counter()
        try:
            if is_post:
                conn.request("POST", f"/api/actions/{blink}?amount=0.01", body, {"Content-Type": "application/json"})
            else:
                conn.request("GET", f"/api/actions/{blink}")
            r = conn.getresponse()
            r.read()
            if r.status >= 400:
                errors += 1
        except (OSError, http.client.HTTPException):
            errors += 1
            conn.close()
            conn = http.client.HTTPConnection(host, port, timeout=10)
            continue
        (lat_post if is_post else lat_get).append((time.perf_counter() - t) * 1e6)
    out[idx] = (lat_get, lat_post, errors)


def pct(xs, p):
    if not xs:
        return 0.0
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(len(xs) * p))]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default="http://127.0.0.1:3000")
    ap.add_argument("--blink", required=True, help="an ACTIVE blink id")
    ap.add_argument("--account", default="2ZTnCzVPMDfSgNWk6iPc57ghC4NrZsEChoyCXVRsP8pV",
                    help="payer pubkey for POSTs (never signs; tx is just built)")
    ap.add_argument("--seconds", type=float, default=10)
    ap.add_argument("--conns", type=int, default=16)
    ap.add_argument("--post-ratio", type=float, default=0.0)
    a = ap.parse_args()

    u = urllib.parse.urlparse(a.url)
    host, port = u.hostname, u.port or 80
    before = system(host, port)
    deadline = time.perf_counter() + a.seconds
    out = [None] * a.conns
    threads = [threading.Thread(target=worker, args=(host, port, a.blink, a.account, deadline, a.post_ratio, out, i))
               for i in range(a.conns)]
    t0 = time.perf_counter()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    elapsed = time.perf_counter() - t0
    after = system(host, port)

    gets = [x for o in out for x in o[0]]
    posts = [x for o in out for x in o[1]]
    errors = sum(o[2] for o in out)
    total = len(gets) + len(posts)
    print(f"\n{total} requests in {elapsed:.1f}s over {a.conns} connections → {total / elapsed:,.0f} req/s, {errors} errors")
    for name, xs in (("GET  action", gets), ("POST action", posts)):
        if xs:
            print(f"  {name}: n={len(xs):>7}  mean={statistics.fmean(xs):7.0f}µs  p50={pct(xs, .5):7.0f}µs"
                  f"  p99={pct(xs, .99):7.0f}µs  max={max(xs):7.0f}µs")
    p0, p1 = before["process"], after["process"]
    c0, c1 = before["counters"], after["counters"]
    cpu_ms = (p1["cpu_user_ms"] + p1["cpu_sys_ms"]) - (p0["cpu_user_ms"] + p0["cpu_sys_ms"])
    print(f"\nserver: cpu {cpu_ms / 1000:.2f}s over {elapsed:.1f}s ({cpu_ms / elapsed / 10:.0f}% of one core),"
          f" rss {p1['rss_bytes'] / 2**20:.1f} MiB, cpu-µs/request {cpu_ms * 1000 / max(total, 1):.0f}")
    print(f"pipeline: +{c1['events_committed'] - c0['events_committed']} events committed in"
          f" +{c1['log_batches'] - c0['log_batches']} group commits"
          f" (avg batch {(c1['events_committed'] - c0['events_committed']) / max(1, c1['log_batches'] - c0['log_batches']):.1f}),"
          f" {c1['events_dropped'] - c0['events_dropped']} shed, max batch {after['pipeline']['max_batch']}")


if __name__ == "__main__":
    main()
