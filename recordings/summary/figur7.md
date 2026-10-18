# Session Report: Figur7

- Recording: `recordings/profiles/figur7.jsonl` + `figur7.evtest`
- Figure: 3-pin
- Frames: 2032 | tracked: 1889 (93.0%)
- Total contacts: 5886 | active frames: 1849

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| Figur7 | 1889 | 4 | 1639 | 0 | 0 |

`slot_churn`: slot 0: 3, slot 1: 2, slot 2: 7, slot 3: 8

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 13 | idx 0 (sole 0/13) |
| Figur7 | 4 | 9 | idx 0 (sole 4/9) |
| Holzklotz3 | 0 | 9 | idx 0 (sole 0/9) |
| Holzklotz4 | 0 | 9 | idx 0 (sole 0/9) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Figur7 ↔ Holzklotz3 | 3 | 23.31 | Safe |
| Figur7 ↔ FM4 | 3 | 26.70 | Safe |
| Figur7 ↔ Holzklotz4 | 3 | 38.08 | Safe |
| Figur10 ↔ Figur7 | 3 | 63.96 | Safe |

Raw data: `docs/recordings/metrics/figur7-{metrics,deviation,similarity}.json`
