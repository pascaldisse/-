# Rudra-II HANDOFF

## 状態
- atom=L1 neutral JA canary是正走行·完
- branch=`terra/rudra-ii`
- cohort2=`80ce86b`·FAIL; cohort3=`f07f952`·訂正canary
- 旧`3594f63`=別cohort·混算禁
- Yama独立照合=sidecar8=raw8·prompt SHA再計算一致

## canary結果
- prompt SHA=`62bbe8ca211a5ad21cd3efc084886229ff05f95cd1e1d1e0ac6c141ffe266a2f`
- 固定=`temperature=0`·`top_p=1`·`max_tokens=512`·`seed=424242`·`stop=["\\n"]`
- DeepSeek=`deepseek-chat`·HTTP200×2·`finish_reason=stop`×2·final content=`1: 3`,`1: 4`·exact=2/2·採用候補
- Moonshot=`kimi-k2.6`·2lane×(初回+30s+30s)=HTTP429×6·exact=0/2·死枝
- Novita=cohort2 HTTP403×2·credential連打零·死枝
- GLM raw credential無·Ollama/local executable無

## gate
- 採用=DeepSeek一族のみ
- non-DeepSeek≥2未達·計3族均衡未達
- canary=`FAIL/UNVERIFIED`·M0判定禁·錨未走
- raw全死=非成立→gaia clean-lane fallback不要・未使用

## 残pass / 再開条件
1. Moonshot課金復旧 又は GLM raw credential 又は Ollama導入
2. 新族ごとneutral JA exact=2/2 canary; 同一schema/stop/sampling; raw+sidecar保存
3. DeepSeek+non-DeepSeek≥2=計≥3族・族同数を確定
4. gate後のみ4pass=`ja/en × symbol/plain`; `距(愛,無)`・`軸(生↔死;夢)`; 問×pass×族 N5·1問1clean lane
5. M0=両錨各≥2基底・族均衡; 未達=FAIL/UNVERIFIED

## 成果
- `地図/rudra-ii-canary/`=cohort2
- `地図/rudra-ii-canary-l1/`=cohort3·Yama審査済
- `地図/run_rudra_ii_canary_l1.py`=再現runner
