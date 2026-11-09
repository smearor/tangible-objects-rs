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
info-card-title = Info
info-touches = Touch points
info-figures = Figures
info-tracked = Tracked
info-fields = Fields
info-dead-zones = Dead zones
info-filtered = Suppressed → forwarded
info-stability = Stability
info-fields-value = { $occupied }/{ $total } ({ $sets } sets)
info-zones-value = { $zones } ({ $filtered } filtered)
info-filtered-value = { $suppressed } → { $forwarded }
info-stable = stable
info-stabilizing = { $stable }/{ $threshold }
pin-label = P{ $index } ({ $x },{ $y })
position-label = { $x_mm },{ $y_mm } mm  ·  { $x_px },{ $y_px } px
toggle-edge-margin = Edge
toggle-fields = Fields
toggle-ruler = Ruler
toggle-grid = Grid
toggle-positions = Positions
toggle-passthrough = Pass
filter-mode-off = off
filter-mode-immediate = immediate
filter-mode-gated = gated
filter-mode-guarded = guarded
filter-mode-tangibles-only = only
window-title = Tangible Objects - Surface Monitor

background-alpha = Alpha: { $percent }%

# GTK - cards
card-visualization-title = Visualization
card-filter-title = Filter mode
card-profiles-title = Profiles
card-field-sets-title = Field sets
col-name = Name
col-owner = Owner
col-enabled = Enabled
col-position = Position
col-fields = Fields
