# Rahu·梯5 C8審

- 対象=`1336d8b418b6a9b5fa5850efd0e354a3f5e0b21c`・分殿=`rahu/梯5-c8-audit`。
- 審器=`docs/adversary/具/梯5_C8審.py`。65 RIFF corpus→`歌口 --源 wav --跳幅 2048`→Z行regex独立parse。exit=0以外に、Z行数・`x/y/theta/r`有限・C13 active=0・C8巻契約を判定。
- corpus生成=`歌口注入生成.py` (struct手書きRIFF)・目録=`注入目録.json`。生成物は`具/.gitignore`下、65件。

## 実測

```text
$ python3 docs/adversary/具/梯5_C8審.py
corpus=65 exit0=65 failures=3
C8_揺_440_120cent: Z=70 active=70 lap=[0,1] → 合
C8_揺_440_50cent: Z=70 active=70 lap=[0] → 不合 (期待lap=[1])
C8_滑走_220_880: Z=93 active=93 lap=[0,1] → 不合 (lap非単調; 期待0..2)
C8_滑走_880_220: Z=93 active=93 lap=[0,1] → 不合 (lap非単調; 期待0..2)
C13_低域_27_5: Z=46 active=0 → 合
C13_高域_14080: Z=46 active=0 → 合
C13_超高域_21000: Z=46 active=0 → 合
```

詳細=`Rahu梯5C8/table.json`・各Z log。失敗時exit=1は審器の判定、被審65走のexitは全0。

## C17b / R

- C17b: `C8_揺_440_120cent.log` を `環音 --源 log --律動 off --掃引 off --実音 off` へ投入。`C17b.wav` = 48000Hz/mono/16bit/3.0s、RIFF/data/byte-rate/block-align全整合、RMS=0.439893、peak=0.999969。Z parser接続=合。
- R: `theta=0,r=1,lap=1` 120行→環音wav→歌口。復元46 Z active、`lap=[0]`、`hz=219.621381..219.631394`。入力lap=1を保存せず、octave leap反例=不合。

## 門

```text
$ cargo test --manifest-path 機関/歌口/Cargo.toml
9 passed; 0 failed
```

## 凍結tracker

|反例|判定|
|---|---|
|octave leap|Rで再現・不合|
|lap|C8三件不合|
|非有限|C8全Z有限; 合|
|C13帯域|active=0三件; 合|
|C17b|下流wav生成; 合|
|律8→12|UNVERIFIED: C8不合を先閉じるまで比較不能|
|stale/hop-rate/frame drop|UNVERIFIED: 本審器hop=2048固定、別専用clock試験未作成|

判定=**不合**。C8三件とR octave損失が残るため、C17b単独成功・cargo緑は昇格根拠にならぬ。
