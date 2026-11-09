# GTK - control bar
mode-live = Modus: Live
mode-training = Modus: Training
figure-name-placeholder = Figurenname...
save-profile = Profil speichern

# GTK - canvas
device-error = Gerät konnte nicht geöffnet werden: { $error }
no-calibration = Keine Kalibrierung verfügbar - GUI ohne Auflösungsinfo nicht nutzbar
place-figure = Lege die Figur auf die Touchfläche...
wait-for-points = Warte auf ≥3 Punkte... ({ $count })
stable-hint = STABIL - Name eingeben und Speichern klicken
stable-status = STABIL: { $count } Punkte - Name eingeben und speichern
stabilizing = Stabilisiere... { $stable }/{ $threshold }
stabilizing-status = Stabilisiere... { $stable }/{ $threshold } ({ $count } Punkte)
live-status = { $figures ->
    [one] { $figures } Figur erkannt
   *[other] { $figures } Figuren erkannt
} | { $touches ->
    [one] { $touches } Touch-Punkt
   *[other] { $touches } Touch-Punkte
} | Felder: { $occupied }/{ $fields } ({ $sets } Sets) | Dead-Zones: { $zones }, { $filtered } gefiltert | { $suppressed } unterdrückt, { $forwarded } weitergeleitet
info-card-title = Info
info-touches = Touch-Punkte
info-figures = Figuren
info-tracked = Getrackt
info-fields = Felder
info-dead-zones = Dead-Zones
info-filtered = Unterdrückt → weitergeleitet
info-stability = Stabilität
info-fields-value = { $occupied }/{ $total } ({ $sets } Sets)
info-zones-value = { $zones } ({ $filtered } gefiltert)
info-filtered-value = { $suppressed } → { $forwarded }
info-stable = stabil
info-stabilizing = { $stable }/{ $threshold }
pin-label = P{ $index } ({ $x },{ $y })
position-label = { $x_mm },{ $y_mm } mm  ·  { $x_px },{ $y_px } px
toggle-edge-margin = Rand
toggle-fields = Felder
toggle-ruler = Lineal
toggle-grid = Gitter
toggle-positions = Positionen
toggle-passthrough = Passthrough
filter-mode-off = aus
filter-mode-immediate = sofort
filter-mode-gated = gated
filter-mode-guarded = guarded
filter-mode-tangibles-only = nur Tangibles
window-title = Tangible Objects - Surface Monitor

background-alpha = Alpha: { $percent }%

# GTK - Cards
card-visualization-title = Visualisierung
card-filter-title = Filtermodus
card-profiles-title = Profile
card-field-sets-title = Feld-Sets
col-name = Name
col-owner = Besitzer
col-enabled = Aktiv
col-position = Position
col-fields = Felder
