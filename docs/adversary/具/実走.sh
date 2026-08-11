#!/bin/bash
# 環統合·実走台本 — build/test全文·生成wav収集·afplay可聴証跡を一走で残す.
# /tmp不使用. 証跡根=docs/adversary/証/日時識別子.

set -uo pipefail

root="$(cd "$(dirname "$0")/../../.." && pwd)"
control="$root/機関/環制御"
stamp="$(date '+%Y%m%dT%H%M%S%z')-$$"
proof="$root/docs/adversary/証/$stamp"
wavs="$proof/wav"
marker="$proof/生成開始印"
build_log="$proof/cargo-build.log"
test_log="$proof/cargo-test.log"
record="$proof/実走記録.txt"
literals="$proof/裸数値literal全行.txt"
list="$proof/wav一覧.tsv"

mkdir -p "$wavs"
: > "$marker"

{
    printf '時=%s\n' "$stamp"
    printf '殿=%s\n' "$root"
    printf 'HEAD=%s\n' "$(git -C "$root" rev-parse HEAD)"
    printf '枝=%s\n' "$(git -C "$root" branch --show-current)"
    printf '制御=%s\n' "$control"
    printf '建log=%s\n' "$build_log"
    printf '試log=%s\n' "$test_log"
    printf '裸数=%s\n' "$literals"
} | tee "$record"

printf '実行: (cd %s && cargo build) 2>&1 | tee %s\n' "$control" "$build_log" | tee -a "$record"
if (cd "$control" && cargo build 2>&1) | tee "$build_log"; then
    build_status=0
else
    build_status=$?
fi
printf 'cargo_build終了=%s\n' "$build_status" | tee -a "$record"

printf '実行: (cd %s && cargo test) 2>&1 | tee %s\n' "$control" "$test_log" | tee -a "$record"
if (cd "$control" && cargo test 2>&1) | tee "$test_log"; then
    test_status=0
else
    test_status=$?
fi
printf 'cargo_test終了=%s\n' "$test_status" | tee -a "$record"

# Rust字句上の裸数値候補を全行保存. 判定は別審でparam既定有無を照合する.
rg --pcre2 -n --glob '*.rs' \
    '(?<![[:alnum:]_])[0-9]+(?:\.[0-9]+)?(?:_[0-9]+)*(?:_(?:f32|f64|u8|u32|u64|usize|i64|u128))?(?![[:alnum:]_])' \
    "$control/src" > "$literals" || true
printf '裸数値全行=%s\n' "$literals" | tee -a "$record"

printf 'path\tbytes\tafplay終了\t実再生秒\tafplaylog\n' > "$list"
# build/test開始後に機関配下へ生じたwavのみを証域へ複写. target等を除外しないことで生成先の逃避を検出する.
while IFS= read -r -d '' source; do
    relative="${source#"$root/機関/"}"
    destination="$wavs/$relative"
    mkdir -p "$(dirname "$destination")"
    cp -p "$source" "$destination"
    bytes="$(stat -f '%z' "$destination")"
    started="$(python3 -c 'import time; print(time.monotonic())')"
    afplay "$destination" > "$destination.afplay.log" 2>&1
    afplay_status=$?
    ended="$(python3 -c 'import time; print(time.monotonic())')"
    seconds="$(python3 - "$started" "$ended" <<'PY'
import sys
print(f"{float(sys.argv[2]) - float(sys.argv[1]):.6f}")
PY
)"
    printf '%s\t%s\t%s\t%s\t%s\n' "$destination" "$bytes" "$afplay_status" "$seconds" "$destination.afplay.log" | tee -a "$list" "$record"
done < <(find "$root/機関" -type f -iname '*.wav' -newer "$marker" -print0)

if [ "$(wc -l < "$list")" -eq 1 ]; then
    printf '生成wav=0; 可聴=UNVERIFIED\n' | tee -a "$record"
fi

printf '証跡=%s\n' "$proof" | tee -a "$record"
exit $(( build_status != 0 || test_status != 0 ))
