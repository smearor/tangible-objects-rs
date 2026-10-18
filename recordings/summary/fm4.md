# Session Report: FM4

- Recording: `recordings/profiles/fm4.jsonl` + `fm4.evtest`
- Figure: 4-pin; spawn_on_partial=true, spawn_stability=15
- Frames: 808 | tracked: 779 (96.4%)
- Total contacts: 3589 | active frames: 764

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| FM4 (partial) | 779 | 4 | 0 | 611 | 42 |

`slot_churn`: slot 0: 4, slot 1: 7, slot 2: 3, slot 3: 6, slot 4: 7, slot 5: 1

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 183 | idx 2 (sole 0/183) |
| Figur7 | 0 | 183 | idx 0 (sole 0/183) |
| FM4 | 0 | 24 | idx 2 (sole 1/24) |
| Holzklotz2 | 0 | 17 | idx 2 (sole 0/17) |
| Holzklotz3 | 0 | 164 | idx 0 (sole 2/164) |
| Holzklotz4 | 0 | 164 | idx 0 (sole 5/164) |
| TestFigur-5P | 0 | 17 | idx 2 (sole 0/17) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| FM4 ↔ Holzklotz2 | 4 | 21.89 | Safe |
| Figur7 ↔ FM4 | 3 | 26.70 | Safe |
| FM4 ↔ TestFigur-5P | 4 | 29.77 | Safe |
| Figur10 ↔ FM4 | 3 | 42.09 | Safe |
| FM4 ↔ TestFigur-4P | 4 | 60.07 | Safe |

Raw data: `docs/recordings/metrics/fm4-{metrics,deviation,similarity}.json`
