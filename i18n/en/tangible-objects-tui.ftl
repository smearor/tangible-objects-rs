# TUI - status line
status-connecting = connecting…
status-filter-rejected = setFilterMode { $mode } rejected
status-daemon-error = daemon error: { $error }
status-ready = { $touches } touch points, { $tangibles } tangibles - f filter  r refresh  q quit

# TUI - pane titles
title-slots = slots ({ $count })
title-tangibles = tangibles ({ $count })
title-fields = fields
title-field-sets = field sets ({ $count })
title-profiles = profiles ({ $count })
title-filter-metrics = filter/metrics
title-events = events
title-surface = surface { $width }x{ $height } mm

# TUI - tabs + inline markers
tab-overview = [1] overview
tab-live = [2] live
marker-pending = pending
marker-window = +window
field-suppress = suppress
field-info = info

# TUI - metrics pane + log
metrics-filter = filter: { $mode }   [f cycles]
metrics-stages = stages: { $count }
metrics-frames = frames { $frames } tracked { $tracked }
metrics-contacts = contacts { $contacts } tangibles { $tangibles }
log-lagged = lagged - { $count } events skipped
