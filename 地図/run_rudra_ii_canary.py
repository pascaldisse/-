#!/usr/bin/env python3
"""Rudra-II: raw-API, neutral Japanese formatting canary; no anchor questions."""
from __future__ import annotations

import hashlib
import json
import re
import sys
import threading
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent / "rudra-ii-canary"
RAW = ROOT / "raw"
SIDECAR = ROOT / "sidecar.jsonl"
EVALUATOR = ROOT / "evaluator.tsv"
AUTH = Path.home() / ".pi" / "agent" / "auth.json"
PROMPT = "答のみ。説明・前置き・後書き禁止。\n0=同一。10=無関係。\n距(机,椅子)を0〜10の整数で答える。\n形式: 1: 0〜10の整数"
PROMPT_SHA256 = hashlib.sha256(PROMPT.encode()).hexdigest()
SAMPLING = {"temperature": 0, "top_p": 1, "max_tokens": 8, "seed": 424242, "stop": ["\n"]}
# All candidates use this same OpenAI-compatible request schema and sampling.
CANDIDATES = (
    {"candidate": "deepseek-v4-flash", "family": "DeepSeek", "provider": "deepseek", "url": "https://api.deepseek.com/chat/completions", "model": "deepseek-v4-flash"},
    {"candidate": "kimi-k2.6", "family": "Moonshot", "provider": "moonshotai", "url": "https://api.moonshot.ai/v1/chat/completions", "model": "kimi-k2.6"},
    {"candidate": "novita-llama-3.1-8b", "family": "Novita", "provider": "novita", "url": "https://api.novita.ai/v3/openai/chat/completions", "model": "meta-llama/llama-3.1-8b-instruct"},
)
EXACT = re.compile(r"^1: (?:[0-9]|10)$")
LOCK = threading.Lock()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def output_path(lane_id: str, family: str) -> Path:
    p = RAW / family / f"{lane_id}.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    return p


def make_body(model: str) -> dict:
    return {
        "model": model,
        "messages": [{"role": "user", "content": PROMPT}],
        **SAMPLING,
    }


def invoke(candidate: dict, replica: int, auth: dict) -> dict:
    lane_id = f"canary-ja-neutral-{candidate['family'].lower()}-r{replica}-{uuid.uuid4()}"
    body = make_body(candidate["model"])
    raw_path = output_path(lane_id, candidate["family"])
    started_at = utc_now()
    status = None
    response = None
    err = None
    try:
        wire = json.dumps(body, ensure_ascii=False, separators=(",", ":")).encode()
        req = Request(candidate["url"], data=wire, method="POST", headers={"Authorization": f"Bearer {auth[candidate['provider']]['key']}", "Content-Type": "application/json"})
        with urlopen(req, timeout=90) as res:
            status = res.status
            response = json.loads(res.read().decode())
    except HTTPError as exc:
        status = exc.code
        try:
            response = json.loads(exc.read().decode())
        except Exception:
            response = {"error": {"type": "HTTPError", "message": str(exc)}}
    except (URLError, TimeoutError, OSError, json.JSONDecodeError) as exc:
        err = f"{type(exc).__name__}: {exc}"
        response = {"error": {"type": type(exc).__name__, "message": str(exc)}}
    raw_path.write_text(json.dumps(response, ensure_ascii=False, indent=2) + "\n")
    choice = (response.get("choices") or [{}])[0]
    message = choice.get("message") or {}
    raw_answer = message.get("content")
    exact = bool(status == 200 and isinstance(raw_answer, str) and EXACT.fullmatch(raw_answer))
    if exact:
        exclusion = ""
    elif status != 200:
        exclusion = f"HTTP{status}" if status else "transport-error"
    else:
        exclusion = "exact-format-violation"
    record = {
        "run": "Rudra-II-canary-cohort2",
        "timestamp_utc": started_at,
        "lane_id": lane_id,
        "candidate": candidate["candidate"],
        "family": candidate["family"],
        "provider": candidate["provider"],
        "transport": "OpenAI-compatible raw",
        "question_id": "canary-neutral-ja-1",
        "language_id": "ja",
        "frame_id": "neutral",
        "prompt": PROMPT,
        "prompt_sha256": PROMPT_SHA256,
        "request_body": body,
        "temperature": SAMPLING["temperature"],
        "top_p": SAMPLING["top_p"],
        "max_tokens": SAMPLING["max_tokens"],
        "seed": SAMPLING["seed"],
        "seed_present": True,
        "stop": SAMPLING["stop"],
        "http_status": status,
        "response_id": response.get("id"),
        "response_model": response.get("model"),
        "system_fingerprint": response.get("system_fingerprint"),
        "finish_reason": choice.get("finish_reason"),
        "raw_answer": raw_answer,
        "exact_schema": "^1: (?:[0-9]|10)$",
        "exact_valid": exact,
        "acceptance": "accepted" if exact else "rejected",
        "exclusion_reason": exclusion,
        "error": err,
        "raw_path": str(raw_path.relative_to(ROOT)),
    }
    with LOCK:
        with SIDECAR.open("a") as f:
            f.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
    return record


def main() -> int:
    if ROOT.exists():
        print(f"refusing to overwrite existing artifact directory: {ROOT}", file=sys.stderr)
        return 2
    ROOT.mkdir(parents=True)
    auth = json.loads(AUTH.read_text())
    with ThreadPoolExecutor(max_workers=6) as pool:
        jobs = [pool.submit(invoke, c, r, auth) for c in CANDIDATES for r in (1, 2)]
        rows = [job.result() for job in as_completed(jobs)]
    rows.sort(key=lambda x: (x["family"], x["lane_id"]))
    by_family = {c["family"]: [r for r in rows if r["family"] == c["family"]] for c in CANDIDATES}
    lines = ["family\tcandidate\tvalid\ttotal\texact_rate\tgate\thttp_statuses\tmodel_responses\tmodel_configured"]
    for c in CANDIDATES:
        group = by_family[c["family"]]
        n = sum(r["exact_valid"] for r in group)
        statuses = ",".join(str(r["http_status"]) for r in group)
        models = ",".join(str(r["response_model"]) for r in group)
        gate = "PASS" if n == 2 else "FAIL"
        lines.append(f"{c['family']}\t{c['candidate']}\t{n}\t{len(group)}\t{n}/{len(group)}\t{gate}\t{statuses}\t{models}\t{c['model']}")
    EVALUATOR.write_text("\n".join(lines) + "\n")
    print(EVALUATOR.read_text(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
