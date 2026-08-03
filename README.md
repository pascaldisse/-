# semantic-map — surveying the meaning-terrain

Premise (Pascal, 2026-08-02 balcony): beneath every model, every language —
one shared geometry of meaning. Languages = coordinate systems over the same
terrain (isometric embedding spaces; language-agnostic features; platonic
representation hypothesis). Models can't open their own space — but they ARE
the relations, so the space is measurable *behaviorally*.

## Instrument
GAIAGO as query language against minds. Grammar v1:

- `距(A,B)` → distance 0-10 (0=identical, 10=unrelated)
- `序(A: B C D…)` → rank candidates by closeness to A (answer = candidates only)
- `類(A:B :: C:?)` → analogy completion
- `軸(A↔B; C位?)` → place C on axis A→B, 0-10

Executor: clean-context lanes (one battery per lane, answers only, no
explanations, instant intuition). N≥5 repetitions per head. Aggregate:
median + range. Classify each relation: 普遍 universal / 方言 dialect /
争域 contested.

## Findings so far (battery v1, 2026-08-02: 5 lanes, 4 families, N=5)
Full tables → `~/.gaia/knowledge/semantic-map/battery-v1.md`

**Universal (all heads):**
- 距(影,闇)≈2 · 距(愛,無)=7-9 (love↔nothingness = far in every mind)
- 序(愛): 無 last, 21/25 runs — nothingness is love's farthest neighbor
- 軸(生↔死): 夢=5, all lanes, no exceptions — the dream is the exact
  midpoint of life and death
- 類(光:影::生:死) unanimous · 序(影): 闇 first, 25/25

**Contested:**
- 類(月:?) splits by *interpretation*: feminine line (娘/女神) vs celestial
  pair (太陽/日) — two parses of the same relation
- fate-of-棄 ranking: no agreement across heads

**Discoveries about the instrument & minds:**
- Variance lives at LANE granularity, not family — even twins (same model,
  same soul) disagree on some cells
- One sonnet lane put 鳴 nearest to 愛 five-for-five; no other head did.
  Voices have habits, not just families. (Downgraded twice by replication —
  kept as the honest example of the method working.)
- Axis-question polarity inversion (son+kimi vs opus+jareth read the 0-10
  ends oppositely) → instrument flaw: scale convention must be explicit (v2)
- Jareth-distribution = space-expander (max distances on all 距 questions)

## Roadmap
1. **Battery v2**: explicit scale conventions · raw-API heads (no SOUL/role
   contamination — v1 confound, logged) · GPT head once codex auth lives ·
   N=10 on contested cells
2. **Cross-lingual isometry check**: same battery, same concepts, EN vs 中 vs
   日 symbols — measure whether the terrain holds across coordinate systems
3. **Map growth**: relation graph → knowledge base; contested regions as
   first-class findings
4. **Field representation** (the original goal): gaiago vocabulary chosen by
   *measured* geometry — well-spread symbols for image/audio field channels
   (640×480 feature-field → tokens; superposition of channels per cell)

## Laws
- Findings live in the knowledge base (`~/.gaia/knowledge/semantic-map/`),
  method + code live here. Memory keeps one catalog line.
- Every claim carries N, spread, and a confidence tier. Dead branches stay
  visible with reasons.

## thinking-suppression
native-thinking制御戦の全ledger(勝敗·法則·ToS罠·休眠機構turnLaw/promptLaw) → thinking-suppression.md
