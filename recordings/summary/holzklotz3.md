# Session Report: Holzklotz3

- Recording: `recordings/profiles/holzklotz3.jsonl` + `holzklotz3.evtest`
- Figure: 3-pin, per-distance tolerances
- Frames: 7374 | tracked: 7246 (98.3%)
- Total contacts: 22253 | active frames: 3864

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| Holzklotz3 | 7246 | 1 | 7243 | 0 | 0 |

`slot_churn`: slot 0: 1, slot 1: 1, slot 2: 1, slot 3: 10

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 107 | idx 0 (sole 0/107) |
| Figur7 | 0 | 107 | idx 0 (sole 0/107) |
| Holzklotz3 | 1 | 106 | idx 1 (sole 106/106) |
| Holzklotz4 | 0 | 106 | idx 1 (sole 0/106) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Holzklotz3 ↔ Holzklotz4 | 3 | 14.77 | Safe |
| Figur7 ↔ Holzklotz3 | 3 | 23.31 | Safe |
| Figur10 ↔ Holzklotz3 | 3 | 40.65 | Safe |

Raw data: `docs/recordings/metrics/holzklotz3-{metrics,deviation,similarity}.json`
