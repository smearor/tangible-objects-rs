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
} | Felder: { $occupied }/{ $fields } ({ $sets } Sets) | Dead-Zones: { $zones }, { $filtered } gefiltert
live-info = { $figures ->
    [one] { $figures } Figur
   *[other] { $figures } Figuren
} | { $touches ->
    [one] { $touches } Touch-Punkt
   *[other] { $touches } Touch-Punkte
} | Felder { $occupied }/{ $fields } | DZ { $zones } ({ $filtered } gefiltert)
pin-label = P{ $index } ({ $x },{ $y })
window-title = Tangible Objects - Surface Monitor

background-alpha = Alpha: { $percent }%
