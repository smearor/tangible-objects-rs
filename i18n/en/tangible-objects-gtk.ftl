# GTK - control bar
mode-live = Mode: Live
mode-training = Mode: Training
figure-name-placeholder = Figure name...
save-profile = Save profile

# GTK - canvas
device-error = Device could not be opened: { $error }
no-calibration = No calibration available - GUI needs resolution info to work
place-figure = Place the figure on the touch surface...
wait-for-points = Waiting for ≥3 points... ({ $count })
stable-hint = STABLE - enter a name and click save
stable-status = STABLE: { $count } points - enter a name and save
stabilizing = Stabilizing... { $stable }/{ $threshold }
stabilizing-status = Stabilizing... { $stable }/{ $threshold } ({ $count } points)
live-status = { $figures ->
    [one] { $figures } figure recognized
   *[other] { $figures } figures recognized
} | { $touches ->
    [one] { $touches } touch point
   *[other] { $touches } touch points
} | fields: { $occupied }/{ $fields } ({ $sets } sets) | dead zones: { $zones }, { $filtered } filtered | { $suppressed } suppressed, { $forwarded } forwarded
live-info = { $figures ->
    [one] { $figures } figure
   *[other] { $figures } figures
} | { $touches ->
    [one] { $touches } touch point
   *[other] { $touches } touch points
} | fields { $occupied }/{ $fields } | DZ { $zones } ({ $filtered } filtered) | { $suppressed } suppr. → { $forwarded } fwd
pin-label = P{ $index } ({ $x },{ $y })
position-label = { $x_mm },{ $y_mm } mm  ·  { $x_px },{ $y_px } px
toggle-edge-margin = Edge
toggle-fields = Fields
toggle-ruler = Ruler
toggle-positions = Positions
toggle-passthrough = Pass
filter-mode-off = off
filter-mode-immediate = immediate
filter-mode-gated = gated
filter-mode-guarded = guarded
filter-mode-tangibles-only = only
window-title = Tangible Objects - Surface Monitor

background-alpha = Alpha: { $percent }%
