#!/usr/bin/env python3
"""Rudra-II L1: corrected raw canary; neutral JA only, anchors prohibited."""
from __future__ import annotations

import hashlib
import json
import re
import sys
import threading
import time
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent / "rudra-ii-canary-l1"
RAW = ROOT / "raw"
SIDECAR = ROOT / "sidecar.jsonl"
EVALUATOR = ROOT / "evaluator.tsv"
AVAILABILITY = ROOT / "availability.tsv"
AUTH = Path.home() / ".pi" / "agent" / "auth.json"
PROMPT = "答のみ。説明・前置き・後書き禁止。\n0=同一。10=無関係。\n距(机,椅子)を0〜10の整数で答える。\n形式: 1: 0〜10の整数"
PROMPT_SHA256 = hashlib.sha256(PROMPT.encode()).hexdigest()
# Shared response schema, stop condition, and sampling for every raw candidate in this cohort.
SAMPLING = {"temperature": 0, "top_p": 1, "max_tokens": 512, "seed": 424242, "stop": ["\n"]}
CANDIDATES = (
    {"candidate": "deepseek-chat", "family": "DeepSeek", "provider": "deepseek", "url": "https://api.deepseek.com/chat/completions", "model": "deepseek-chat", "retry_429": 0},
    {"candidate": "kimi-k2.6", "family": "Moonshot", "provider": "moonshotai", "url": "https://api.moonshot.ai/v1/chat/completions", "model": "kimi-k2.6", "retry_429": 2},
)
EXACT = re.compile(r"^1: (?:[0-9]|10)$")
LOCK = threading.Lock()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def final_line(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    lines = [line.strip() for line in value.splitlines() if line.strip()]
    return lines[-1] if lines else ""


def body(model: str) -> dict:
    return {"model": model, "messages": [{"role": "user", "content": PROMPT}], **SAMPLING}


def single_attempt(candidate: dict, lane_id: str, attempt: int, auth: dict) -> dict:
    request_body = body(candidate["model"])
    raw_path = RAW / candidate["family"] / f"{lane_id}-a{attempt}.json"
    raw_path.parent.mkdir(parents=True, exist_ok=True)
    status, response, error = None, None, None
    try:
        wire = json.dumps(request_body, ensure_ascii=False, separators=(",", ":")).encode()
        request = Request(candidate["url"], data=wire, method="POST", headers={"Authorization": f"Bearer {auth[candidate['provider']]['key']}", "Content-Type": "application/json"})
        with urlopen(request, timeout=120) as result:
            status = result.status
            response = json.loads(result.read().decode())
    except HTTPError as exc:
        status = exc.code
        payload = exc.read().decode()
        try:
            response = json.loads(payload)
        except json.JSONDecodeError:
            response = {"error": {"type": "HTTPError", "message": payload or str(exc)}}
    except (URLError, TimeoutError, OSError, json.JSONDecodeError) as exc:
        error = f"{type(exc).__name__}: {exc}"
        response = {"error": {"type": type(exc).__name__, "message": str(exc)}}
    raw_path.write_text(json.dumps(response, ensure_ascii=False, indent=2) + "\n")
    choice = (response.get("choices") or [{}])[0]
    message = choice.get("message") or {}
    raw_answer = message.get("content")
    last = final_line(raw_answer)
    exact = bool(status == 200 and last is not None and EXACT.fullmatch(last))
    if exact:
        exclusion = ""
    elif status != 200:
        exclusion = f"HTTP{status}" if status else "transport-error"
    else:
        exclusion = "exact-final-content-line-violation"
    record = {
        "run": "Rudra-II-canary-L1-cohort3",
        "timestamp_utc": utc_now(),
        "lane_id": lane_id,
        "attempt": attempt,
        "candidate": candidate["candidate"],
        "family": candidate["family"],
        "provider": candidate["provider"],
        "transport": "OpenAI-compatible raw",
        "question_id": "canary-neutral-ja-1",
        "language_id": "ja",
        "frame_id": "neutral",
        "prompt": PROMPT,
        "prompt_sha256": PROMPT_SHA256,
        "request_body": request_body,
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
        "final_content_line": last,
        "exact_schema": "^1: (?:[0-9]|10)$",
        "exact_valid": exact,
        "acceptance": "accepted" if exact else "rejected",
        "exclusion_reason": exclusion,
        "error": error,
        "raw_path": str(raw_path.relative_to(ROOT)),
    }
    with LOCK:
        with SIDECAR.open("a") as f:
            f.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
    return record


def lane(candidate: dict, replica: int, auth: dict) -> list[dict]:
    lane_id = f"canary-l1-ja-neutral-{candidate['family'].lower()}-r{replica}-{uuid.uuid4()}"
    records = [single_attempt(candidate, lane_id, 1, auth)]
    for attempt in range(2, candidate["retry_429"] + 2):
        if records[-1]["http_status"] != 429:
            break
        time.sleep(30)
        records.append(single_attempt(candidate, lane_id, attempt, auth))
    return records


def main() -> int:
    if ROOT.exists():
        print(f"refusing to overwrite existing artifact directory: {ROOT}", file=sys.stderr)
        return 2
    ROOT.mkdir(parents=True)
    auth = json.loads(AUTH.read_text())
    with ThreadPoolExecutor(max_workers=4) as pool:
        jobs = [pool.submit(lane, candidate, replica, auth) for candidate in CANDIDATES for replica in (1, 2)]
        records = [record for job in as_completed(jobs) for record in job.result()]
    records.sort(key=lambda r: (r["family"], r["lane_id"], r["attempt"]))
    finals = {}
    for record in records:
        finals[record["lane_id"]] = record
    lines = ["family\tcandidate\texact_valid_lanes\ttotal_lanes\texact_rate\tgate\tfinal_http\tfinal_content_lines\tmodel_configured"]
    for candidate in CANDIDATES:
        group = [r for r in finals.values() if r["family"] == candidate["family"]]
        valid = sum(r["exact_valid"] for r in group)
        lines.append("\t".join((candidate["family"], candidate["candidate"], str(valid), str(len(group)), f"{valid}/{len(group)}", "PASS" if valid == 2 else "FAIL", ",".join(str(r["http_status"]) for r in group), ",".join(str(r["final_content_line"]) for r in group), candidate["model"])))
    EVALUATOR.write_text("\n".join(lines) + "\n")
    AVAILABILITY.write_text("family\ttransport\tavailability\treason\nNovita\traw API\tdead\tcohort2 HTTP403 twice\nMoonshot\traw API\tmeasured\tcohort3 heartbeat canary\nDeepSeek\traw API\tmeasured\tcohort3 deepseek-chat 512-token canary\nGLM\traw API\tunavailable\tno configured raw credential\nOllama/local\traw API\tunavailable\tno ollama executable\n")
    print(EVALUATOR.read_text(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
