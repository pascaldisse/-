# 梯5 独立gate — Apasmara

基=`60ff915` · 枝=`apasmara/梯5-independent` · 審path=独立worktree。
被審殿`.scratch/建-梯5`非参照。mic=metadataのみ。

|門|実測|判|
|---|---|---|
|C12|2Hz: period=0.4997321429s, std=0.0006561133, duty=0.4992308657 · silence: RMS=0, peak=0, strict-zero=true · resume: 前半nonzero=0, 後半=47960|合|
|C14|出力RIFF/WAVE PCM=16bit mono 48000Hz。CoreAudio実device format/stereoは非起動|UNVERIFIED|
|C15|同入力二走 SHA-256一致 `24bfac…b1065`|合|
|C16|ts=0..119 vs ts=987654321.. のwav SHA-256一致|合|
|C17|RIFF/WAVE · format=1 · channels=1 · rate=48000 · block-align=2 · bits=16 · data=192000|合|
|C17b|有音data nonzero=95920/96000|合|
|C18|param/default surface=20行。意味論的hardcode全数判定器無し|UNVERIFIED|
|C19|`--秒 0` exit=2 · `--律 bogus` exit=1|合|
|C20|正典L=h/8: h0..7 FFT Hz=`219.965,239.934,261.593,285.320,311.160,339.251,369.967,403.502`。h5..7=theta負+lap=1を含む|合|
|C21|`--源 log --実音 off`のみ。録音API非起動。ambient active=0はOS全体独立観測不能|UNVERIFIED|
|C22|`建-梯5/.scratch/mic-20260811T172130+0200`: metadata=`128B, Aug 11 17:21:31 2026`のみ。read/playback/recording皆無|live voiced UNVERIFIED|
|R|八家+負theta正典LはC20合。Z malformed/nullは既存単体test緑。非正典`theta<0,lap=0`でh5..7低octaveは契約外表現、偽赤として棄却|合|

## 命令

```text
cargo test --manifest-path 機関/環音/Cargo.toml
cargo fmt --manifest-path 機関/環音/Cargo.toml -- --check
cargo build --manifest-path 機関/環音/Cargo.toml
cargo run --manifest-path 機関/環音/Cargo.toml -- --log-path .scratch/Apasmara-gate/gate.log --秒 4 --掃引 off --実音 off --wav .scratch/Apasmara-gate/gate2hz.wav
/opt/homebrew/bin/python3 docs/adversary/具/wav審.py .scratch/Apasmara-gate/gate2hz.wav
```

全三cargo門=exit0。exit0=可聴/実mic/live声の証明非。

## 偽合格・死枝

- 偽合格防壁: `afplay`/CoreAudio exit=0を可聴判定へ採用せず。
- 死枝: 非正典`theta=-π/4,lap=0`を八家7と誤仮定→220未満出力。正典L分解では`theta=-π/4,lap=1`、契約式`基音·2^lap·家率`と既存testに一致。修繕無し。
- 修繕=無し。
