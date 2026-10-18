# Session Report: Figur10

- Recording: `recordings/profiles/figur10.jsonl` + `figur10.evtest`
- Figure: 3-pin
- Frames: 3298 | tracked: 3194 (96.8%)
- Total contacts: 9878 | active frames: 3248

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| Figur10 | 3194 | 1 | 3099 | 0 | 0 |

`slot_churn`: slot 0: 1, slot 1: 2, slot 2: 1

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 1 | 102 | idx 0 (sole 71/102) |
| Figur7 | 0 | 102 | idx 0 (sole 0/102) |
| Holzklotz3 | 0 | 102 | idx 0 (sole 0/102) |
| Holzklotz4 | 0 | 102 | idx 0 (sole 0/102) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Figur10 ↔ Holzklotz4 | 3 | 25.87 | Safe |
| Figur10 ↔ Holzklotz3 | 3 | 40.65 | Safe |
| Figur10 ↔ FM4 | 3 | 42.09 | Safe |
| Figur10 ↔ Figur7 | 3 | 63.96 | Safe |

Raw data: `docs/recordings/metrics/figur10-{metrics,deviation,similarity}.json`
