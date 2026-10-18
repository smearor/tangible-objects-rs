# Session Report: TestFigur-5P

- Recording: `recordings/profiles/testfigur-5p.jsonl` + `testfigur-5p.evtest`
- Figure: 5-pin
- Frames: 3121 | tracked: 3104 (99.5%)
- Total contacts: 12384 | active frames: 3097

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| TestFigur-5P | 5219 | 5 | 5045 | 0 | 0 |

`slot_churn`: slot 0: 1, slot 1: 5, slot 2: 3, slot 3: 1

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 5 | idx 0 (sole 0/5) |
| Figur7 | 0 | 5 | idx 0 (sole 0/5) |
| FM4 | 0 | 1 | idx 0 (sole 0/1) |
| Holzklotz2 | 0 | 1 | idx 1 (sole 0/1) |
| Holzklotz3 | 0 | 5 | idx 0 (sole 0/5) |
| Holzklotz4 | 0 | 5 | idx 0 (sole 0/5) |
| TestFigur-5P | 1 | 0 | idx 0 (sole 0/0) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Holzklotz2 ↔ TestFigur-5P | 4 | 9.59 | Safe |
| FM4 ↔ TestFigur-5P | 4 | 29.77 | Safe |
| TestFigur-4P ↔ TestFigur-5P | 4 | 33.39 | Safe |

Raw data: `docs/recordings/metrics/testfigur-5p-{metrics,deviation,similarity}.json`
