# Session Report: Holzklotz2

- Recording: `recordings/profiles/holzklotz2.jsonl` + `holzklotz2.evtest`
- Figure: 4-pin
- Frames: 1772 | tracked: 1547 (87.3%)
- Total contacts: 6963 | active frames: 1686

## SessionMetrics

| Pattern | tracked | IDs | exact | partial | claims |
|---|---|---|---|---|---|
| Holzklotz2 | 1547 | 1 | 1504 | 0 | 0 |

`slot_churn`: slot 0: 2, slot 1: 1, slot 2: 1, slot 3: 1, slot 4: 4

## DeviationReport (generic full candidates)

| Pattern | matched | missed | top blocker (index, sole) |
|---|---|---|---|
| Figur10 | 0 | 660 | idx 1 (sole 0/660) |
| Figur7 | 0 | 660 | idx 0 (sole 0/660) |
| FM4 | 0 | 151 | idx 0 (sole 0/151) |
| Holzklotz2 | 1 | 150 | idx 5 (sole 24/150) |
| Holzklotz3 | 0 | 656 | idx 0 (sole 0/656) |
| Holzklotz4 | 0 | 656 | idx 0 (sole 0/656) |
| TestFigur-5P | 0 | 150 | idx 2 (sole 0/150) |

## SimilarityReport (pairs involving this figure)

| Pair | k | min max-dev (mm) | risk |
|---|---|---|---|
| Holzklotz2 ↔ TestFigur-5P | 4 | 9.59 | Safe |
| FM4 ↔ Holzklotz2 | 4 | 21.89 | Safe |
| Holzklotz2 ↔ TestFigur-4P | 4 | 42.98 | Safe |

Raw data: `docs/recordings/metrics/holzklotz2-{metrics,deviation,similarity}.json`
