# Session Report: Holzklotz4

- Recording: `recordings/profiles/holzklotz4.jsonl` + `holzklotz4.evtest`
- Figure: 3-pin, per-distance tolerances
- Frames: 4529 | tracked: 4467 (98.6%)
- Total contacts: 13621 | active frames: 4443

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| Holzklotz4 | 4467 | 1 | 4420 | 0 | 0 |

`slot_churn`: slot 0: 2, slot 1: 3, slot 2: 1, slot 3: 9

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 7 | idx 0 (sole 0/7) |
| Figur7 | 0 | 7 | idx 0 (sole 0/7) |
| Holzklotz3 | 0 | 7 | idx 2 (sole 7/7) |
| Holzklotz4 | 1 | 6 | idx 2 (sole 6/6) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Holzklotz3 ↔ Holzklotz4 | 3 | 14.77 | Safe |
| Figur10 ↔ Holzklotz4 | 3 | 25.87 | Safe |
| Figur7 ↔ Holzklotz4 | 3 | 38.08 | Safe |

Raw data: `docs/recordings/metrics/holzklotz4-{metrics,deviation,similarity}.json`
