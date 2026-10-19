# frename UI text in German. Same ids, attributes and $arguments as i18n/en/frename.ftl, in the
# same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms; the reader is addressed
# with the informal «du», as in most video and creative software.
#   tag / tags                        Tag / Tags (never «Label»: that is Premiere's Label)
#   in / out points                   In- und Out-Punkte
#   comment                           Kommentar
#   subtitles                         Untertitel
#   screenshot                        Screenshot
#   marker (Premiere)                 Marker
#   range (a marker with a duration)  Bereich
#   subclip (Premiere)                Teilclip (as in German Premiere Pro)
#   Description column (Premiere)     Spalte „Beschreibung“
#   checked (files)                   ausgewählt
#   All / Invert (check boxes)        Alle / Umkehren
#   video / clip                      Video / Clip
#   tag panel / tag list              Tag-Leiste / Tag-Liste (the folder's list of tags)
#   file name                         Dateiname
#   a tag name inside a sentence      in „…“ quotes: „Commented“
#   untagged (filter)                 Ohne Tags
#   changed / unchanged / failed      geändert / unverändert / fehlgeschlagen
#   Cancel / Stopping…                Abbrechen / Wird beendet…
#   log, "see the log"                Protokoll, „siehe Protokoll“
#   System (language list)            System (…)
#   batch action / batch mode         Stapelaktion / Stapelmodus
#   request (to the AI)               Anfrage
#   moments (AI)                      Momente
#   file list / folder                Dateiliste / Ordner
#   settings                          Einstellungen
#   update (a newer frename)          Update
#   AI description (Describe with AI) KI-Beschreibung
#   subtitles generation (Soniox)     Spracherkennung / Untertitel
#
# Plurals: German has the categories one / other, other as the default.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français

## Settings window

settings-window-title = Einstellungen
settings-page-interface = Oberfläche
settings-page-saving = Speichern
settings-page-ai = Mit KI beschreiben
settings-close = Schließen
settings-apply-note = Änderungen gelten sofort. Tab wechselt zwischen den Optionen, Strg+Tab zwischen den Seiten.
settings-language = Sprache
settings-language-system = System ({ $language })
settings-video = Video
settings-video-autoplay = Videos beim Öffnen automatisch abspielen
settings-subtitle-size = Untertitelgröße
settings-subtitle-size-small = Klein
settings-subtitle-size-medium = Mittel
settings-subtitle-size-large = Groß
settings-subtitle-size-extra-large = Sehr groß
settings-tags = Tag-Farben
settings-tags-monochrome = Einfarbig
settings-tags-monochrome-hint = Von dir hinzugefügte Tags erscheinen in dunklerem Grau, die übrigen in hellerem – keine anderen Farben.
settings-file-names = Dateinamen
settings-tags-space-after = Leerzeichen nach jedem Tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Dateien behalten ihre Namen, bis sie umbenannt oder gespeichert werden.
settings-tags-space-add = Leerzeichen zu den Dateinamen hinzufügen…
settings-tags-space-remove = Leerzeichen aus den Dateinamen entfernen…
settings-comments = Kommentare
settings-comments-in-video = In der Videodatei
settings-comments-in-video-hint = XMP, die Spalte „Beschreibung“ in Premiere Pro
settings-comments-text-file = In einer Textdatei neben dem Video
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Dateien behalten ihre Kommentare, wo sie sind, bis sie verschoben werden.
settings-comments-move-into-videos = Kommentare in die Videos verschieben…
settings-comments-move-into-text-files = Kommentare in Textdateien verschieben…
settings-commented-tag = Videos mit Kommentar taggen
settings-commented-tag-name = Tag
settings-commented-tag-hint = Wird gesetzt, wenn du einen Kommentar zu einem Video schreibst, z. B. { $tag }.IMG_0424.MOV, und entfernt, wenn du ihn löschst (KI-Beschreibungen zählen nicht). Sonst kannst du ihn selbst ändern.
settings-commented-tag-off = Videos mit Kommentar bekommen keinen Tag.
settings-in-out = In-/Out-Punkte
settings-in-out-in-video = In der Videodatei
settings-in-out-in-video-hint = Ein Teilclip-Marker in Premiere Pro
settings-in-out-comment = Im Kommentar, in einer Zeile
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Dateien behalten ihre In-/Out-Punkte, wo sie sind, bis sie verschoben werden.
settings-in-out-move-into-videos = In-/Out-Punkte in die Videos verschieben…
settings-in-out-move-into-comments = In-/Out-Punkte in die Kommentare verschieben…
settings-markers = Marker und Bereiche
settings-markers-in-video = In der Videodatei
settings-markers-in-video-hint = XMP, in Premiere Pro am Clip sichtbar
settings-markers-comment = Im Kommentar, je eine Zeile
settings-markers-comment-example = 0:41–0:47 — Löwe
settings-markers-note = Dateien behalten ihre Marker, wo sie sind, bis sie verschoben werden.
settings-markers-move-into-videos = Marker in die Videos verschieben…
settings-markers-copy-into-comment = Marker in die Kommentare kopieren…
settings-markers-hint = Punkte und Bereiche gleichermaßen, auch die Momente, die die KI findet.
settings-updates = Updates
settings-version = Version
settings-old-title = Einstellungen einer älteren frename-Version
settings-old-hint = Übernimm die Einstellungen eines frename, das du aus einem ZIP-Ordner gestartet hast.
settings-old-scheduled = Die Einstellungen werden beim Neustart von frename importiert
settings-old-not-found = Keine frename.exe mit einer frename.db in { $folder }
settings-old-failed = Import nicht möglich: { $reason }
settings-old-import-button = Aus altem frename-Ordner importieren…
settings-ai = Mit KI beschreiben
settings-ai-key-label = Anthropic-API-Schlüssel
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Schlüssel unter console.anthropic.com → API keys holen.
settings-ai-key-remove-confirm = Den gespeicherten Anthropic-Schlüssel entfernen?
settings-ai-used-by = Wird von „Mit KI beschreiben“ im Stapelmodus und bei Markern (✨, Strg+F2) verwendet, wenn ein Claude-Modell gewählt ist.
settings-ai-openai-key-label = OpenAI-API-Schlüssel
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Schlüssel unter platform.openai.com/api-keys holen.
settings-ai-openai-key-remove-confirm = Den gespeicherten OpenAI-Schlüssel entfernen?
settings-ai-openai-used-by = Wird von „Mit KI beschreiben“ im Stapelmodus und bei Markern (✨, Strg+F2) verwendet, wenn ein GPT-Modell gewählt ist.
settings-ai-gemini-key-label = Gemini-API-Schlüssel
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Schlüssel unter aistudio.google.com/apikey holen.
settings-ai-gemini-key-remove-confirm = Den gespeicherten Gemini-Schlüssel entfernen?
settings-ai-gemini-used-by = Wird von „Mit KI beschreiben“ im Stapelmodus und bei Markern (✨, Strg+F2) verwendet, wenn ein Gemini-Modell gewählt ist.
settings-ai-model-needs-key = { $model } – zuerst einen { $provider }-Schlüssel hinzufügen
settings-ai-model-label = Modell
settings-ai-language-label = Sprache der Beschreibung
settings-ai-hint = Haiku, GPT-4.1 mini und Gemini Flash-Lite sind am günstigsten; Sonnet, Opus, GPT-4.1 und Gemini Flash erkennen mehr. Ein Modell braucht den Schlüssel seines eigenen Anbieters.
settings-ai-moments-label = Momente
settings-ai-moments-important = Nur das Auffällige
settings-ai-moments-full = Den ganzen Clip abdecken
settings-ai-moments-important-hint = Schlägt auch vor, wo In und Out gesetzt werden (die KI-Schaltfläche neben IN und OUT)
settings-ai-moments-full-hint = Ein Moment für jeden Abschnitt, auch für ruhige; kein Vorschlag für In und Out
settings-ai-tag-suggestions = Auch Tags aus den Tags des Ordners vorschlagen
settings-ai-tag-suggestions-hint = In derselben Anfrage sagt die KI, welche Tags des Ordners zu jedem Clip passen. Ein vorgeschlagener Tag wird nur hinzugefügt, wenn du ihn anklickst.
settings-subtitles = Untertitel
settings-subtitles-key-label = Soniox-API-Schlüssel
settings-subtitles-key-placeholder = Schlüssel einfügen
settings-subtitles-key-get = Schlüssel unter console.soniox.com holen. Das Audio wird an Soniox gesendet.
settings-subtitles-key-remove-confirm = Den gespeicherten Soniox-Schlüssel entfernen?
settings-subtitles-languages-label = Sprachen
settings-subtitles-languages-locked = Speichere den Schlüssel, um aus allen Sprachen zu wählen, die Soniox kennt.
settings-subtitles-languages-loading = Sprachen werden von Soniox geholt…
settings-subtitles-languages-none = Keine ausgewählt: automatische Erkennung.
settings-subtitles-languages-hint = Die im Material gesprochenen Sprachen, als Hinweise.
settings-subtitles-cue-length-label = Länge der Untertitel
settings-subtitles-cue-short = Kurz
settings-subtitles-cue-short-hint = Eine Zeile, bis zu 8 s
settings-subtitles-cue-sentence = Ein Satz
settings-subtitles-hint = Wird von „Untertitel erzeugen“ im Stapelmodus verwendet. Die Länge gilt für neue Untertitel.
settings-key-store-windows = der Windows-Anmeldeinformationsverwaltung
settings-key-store-macos = dem macOS-Schlüsselbund
settings-key-store-other = dem System-Schlüsselbund
settings-key-unavailable = Der System-Schlüsselbund konnte nicht geöffnet werden
settings-key-unavailable-hint = Er ist vielleicht gesperrt, oder es gibt keinen (wie GNOME Keyring oder KWallet). Die Einstellungen prüfen es bei jedem Öffnen neu.
settings-key-remove-confirm-hint = Du musst ihn dann erneut einfügen.
settings-key-remove-ask = Entfernen…
settings-key-remove = Schlüssel entfernen
settings-key-keep = Behalten
settings-key-replace = Ersetzen…
settings-key-hide = Verbergen
settings-key-show = Anzeigen
settings-key-save = Schlüssel speichern
settings-key-get = Schlüssel holen
settings-key-billing = Guthaben aufladen
settings-key-cancel = Abbrechen
settings-key-saved-in = Auf diesem Computer in { $store } gespeichert
settings-key-save-into = „Schlüssel speichern“ legt ihn auf diesem Computer in { $store } ab.

## Batch mode

batch-checked-count = { $count ->
    [one] { $count } Datei ausgewählt
   *[other] { $count } Dateien ausgewählt
}
batch-no-key = Kein Schlüssel
batch-group-move = Zwischen Orten verschieben
batch-group-fix = Namen und Videos korrigieren
batch-group-paid = Kostenpflichtige Dienste
batch-reason-none-checked = Keine Dateien ausgewählt
batch-reason-reading = Cliplängen werden gelesen…
batch-check-all = Alle { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} auswählen
batch-job-running = läuft
batch-job-finished = fertig
batch-job-stopped = beendet
batch-progress-files = { $finished } von { $total ->
    [one] { $total } Datei
   *[other] { $total } Dateien
}
batch-time-left = noch etwa { $time }
batch-time-estimating = Zeit wird geschätzt…
batch-time-spent = bisher { $time }
batch-runs-in-background = Du kannst weiterarbeiten, während es läuft
batch-clip-in-work = In Arbeit: { $action }
batch-folder-held = Beschäftigt: { $action }
batch-indicator-hint = Klicken für das Fenster und „Abbrechen“
batch-action-prepare = Clips vorbereiten
batch-action-prepare-hint = Für jeden ausgewählten Clip in einem Durchgang: Untertitel (Soniox), falls er keine hat, dann die Beschreibung mit KI: eine Zusammenfassung, ihre Momente als Marker, der Kommentar und Tag-Vorschläge. Ein Preis, ein Limit und ein Fortschritt für alles.
batch-prepare-step-off = ausgeschaltet
batch-prepare-nothing = Für diese Clips gibt es nichts zu tun
batch-prepare-report-cap = { $spent } ausgegeben (Limit { $cap })
batch-option-steps = Schritte
batch-prepare-step-subtitles = Zuerst die Untertitel erzeugen (Soniox)
batch-prepare-step-description = Die Clips mit KI beschreiben
batch-prepare-cap = Ausgaben stoppen bei
batch-prepare-cap-none = Kein Limit (US-Dollar)
batch-prepare-cap-invalid = Das Limit ist ein Dollarbetrag, z. B. 5
batch-prepare-left = Der letzte Lauf wurde beendet; { $count ->
    [one] 1 Clip ist
   *[other] { $count } Clips sind
} übrig
batch-prepare-check-left = Auswählen
batch-prepare-estimating = Es wird ermittelt, was jeder Schritt tut…
batch-prepare-plan-subtitles = Untertitel
batch-prepare-plan-description = Beschreibung
batch-prepare-plan-none = nichts zu tun
batch-prepare-plan-subtitles-line = { $videos ->
    [one] 1 Video
   *[other] { $videos } Videos
} zu transkribieren, { $duration } Audio
batch-prepare-plan-description-line = { $videos ->
    [one] 1 Video
   *[other] { $videos } Videos
} zu beschreiben
batch-prepare-resume-note = Jeder Schritt überspringt, was schon da ist; ein beendeter Lauf wird also fortgesetzt, indem du ihn erneut startest: nichts wird doppelt bezahlt.
batch-prepare-run = { $clips ->
    [one] 1 Clip
   *[other] { $clips } Clips
} vorbereiten · etwa { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Ähnliches Material
batch-prepare-groups-hint = Clips, die wie dieselbe Einstellung aussehen: Duplikate, erneute Exporte, eine Kamera, die sich nicht bewegt hat.
batch-prepare-group-clips = { $count } Clips
batch-prepare-check-group = Diese Clips auswählen
batch-solo-queued = In der Warteschlange: { $action }
batch-suggest-in-job = Eine Beschreibung dieses Clips ist schon im laufenden Auftrag, ohne Tag-Vorschläge. Frage erneut, wenn er beendet ist
batch-suggest-no-tags = In diesem Ordner gibt es noch keine Tags, aus denen vorgeschlagen werden kann: füge zuerst welche hinzu
batch-solo-gave-up = Konnte nicht vorbereitet werden
batch-solo-withdrawn = Zurückgezogen: { $action }
batch-solo-in-work = { $action } ist in Arbeit. Brich es über die Leiste oder die Auftragsseite ab
batch-waiting-title = { $count ->
    [one] 1 Lauf wartet
   *[other] { $count } Läufe warten
}
batch-waiting-remove = Entfernen
batch-indicator-waiting = { $count ->
    [one] { $count } wartet
   *[other] { $count } warten
}
file-workspace-describe-clip = Diesen Clip mit KI beschreiben
file-workspace-subtitles-clip = Untertitel für diesen Clip erzeugen
file-workspace-describe-clip-queued = Mit KI beschreiben: wartet auf seinen Zug. Klicken zum Zurückziehen
file-workspace-describe-clip-running = Mit KI beschreiben: in Arbeit
file-workspace-suggest-tags-clip = Tags für diesen Clip mit KI vorschlagen (Strg+F6)
file-workspace-suggest-tags-clip-queued = Tags vorschlagen: wartet auf seinen Zug. Klicken zum Zurückziehen
file-workspace-suggest-tags-clip-running = Tags vorschlagen: in Arbeit
file-workspace-subtitles-clip-queued = Untertitel erzeugen: wartet auf seinen Zug. Klicken zum Zurückziehen
file-workspace-subtitles-clip-running = Untertitel erzeugen: in Arbeit
batch-run-again = Erneut für { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} ausführen
batch-result-done = Fertig: { $total ->
    [one] { $total } Datei
   *[other] { $total } Dateien
}
batch-result-done-detail = { $changed } geändert, bei { $unchanged } gab es nichts zu ändern
batch-result-problems = Mit Problemen fertig: { $failed } von { $total ->
    [one] { $total } Datei
   *[other] { $total } Dateien
} nicht erledigt
batch-figure-unchanged = unverändert
batch-figure-not-done = nicht erledigt
batch-figure-not-reached = nicht erreicht
batch-files-not-done = Nicht erledigte Dateien
batch-table-file = Datei
batch-table-why = Grund

batch-run-move-comments = { $count ->
    [one] { $count } Kommentar
   *[other] { $count } Kommentare
} verschieben
batch-run-move-in-out = In/Out von { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} verschieben
batch-run-convert = { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} umwandeln
batch-run-rotate = { $count ->
    [one] { $count } Video
   *[other] { $count } Videos
} drehen
batch-run-tag = { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} taggen
batch-run-fix-tags = Tags in { $count ->
    [one] { $count } Namen
   *[other] { $count } Namen
} neu ordnen
batch-run-rename = { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} umbenennen
batch-run-reload = { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} neu laden
batch-run-export-markers = Marker von { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} exportieren
batch-service-soniox = Soniox
batch-change-renames = Benennt Dateien um
batch-change-videos = Schreibt in die Videos
batch-change-text-files = Schreibt Textdateien neben die Videos
batch-change-comments = Schreibt in die Kommentare
batch-change-subtitles = Schreibt Untertiteldateien neben die Videos
batch-change-records = Ändert nur die eigenen Einträge von frename
batch-option-direction = Richtung
batch-option-language = Sprache
batch-option-tag-suggestions = Tag-Vorschläge
batch-option-described = Beschriebene Videos
batch-option-subtitled = Videos mit Untertiteln
batch-option-subtitles-write = Zu schreibende Dateien
batch-option-spacing = Abstand
batch-option-tag = Tag
batch-option-turn = Drehung
batch-plan-videos = Videos
batch-plan-length = Länge
batch-plan-cost = Kosten
batch-plan-time = Zeit
batch-reason-estimate = Warten auf die Schätzung
batch-reason-subtitles-no-format = Wähle mindestens eine zu schreibende Datei aus
batch-set-key = Schlüssel festlegen…
batch-check-key = Schlüssel prüfen…
batch-subtitles-languages-auto = In jedem Video erkannt
batch-action-describe-ai-run-waiting = Videos beschreiben
batch-action-fix-tags-order = Reihenfolge: wie in der Tag-Liste, unbekannte Tags zuerst.
batch-action-markers-comment-hint-short = Macht aus Kommentarzeilen mit Zeitangabe Marker oder kopiert die Marker in den Kommentar.
batch-action-markers-to-comment-hint = Die Marker bleiben; ein erneuter Lauf fügt nichts doppelt hinzu.
batch-action-rotate-hint-short = Dreht MP4- und MOV-Clips über ihr Rotations-Flag, ohne neu zu kodieren.
batch-action-tag-commented-off = Das Taggen kommentierter Videos ist ausgeschaltet
batch-action-tag-commented-choose = Tag wählen…

batch-title = Stapelaktionen
batch-done-label-changed = geändert
batch-done-label-subtitled = mit Untertiteln
batch-ai-at-least = mindestens
batch-cancel = Abbrechen
batch-stopping = Wird beendet…
batch-stop-over-budget = Beendet: Der nächste Clip könnte den Lauf über das Limit von { $cap } bringen ({ $spent } ausgegeben). Erhöhe das Limit, um den Rest zu erledigen.
batch-stopped = Nach { $finished } von { $total ->
    [one] { $total } Datei
   *[other] { $total } Dateien
} beendet.
batch-close = Schließen
batch-failed-subtitles = Ohne Untertitel:
batch-written-subtitles = Geschriebene Dateien:
batch-table-written = Geschrieben
batch-merged-markers = Doppelte Marker zusammengeführt:
batch-table-merged = Zusammengeführt
batch-markers-merged = { $count ->
    [one] { $count } doppelter Marker zusammengeführt
   *[other] { $count } doppelte Marker zusammengeführt
}
batch-add-credit = Guthaben aufladen
batch-open-log = Protokoll öffnen

batch-action-move-comments = Kommentare verschieben
batch-action-move-comments-hint = Verschiebt den Kommentar jeder ausgewählten Datei an den gewählten Ort, samt den In-/Out-Punkten darin. Im Video gespeicherte Tags und In-/Out-Punkte bleiben, wo sie sind.
batch-action-move-comments-into-videos = Aus Textdateien in die Videos (XMP)
batch-action-move-comments-into-text-files = Aus den Videos (XMP) in Textdateien

batch-action-move-in-out = In-/Out-Punkte: Kommentar ⇄ Video (XMP)
batch-action-move-in-out-hint = Verschiebt die In-/Out-Punkte jeder ausgewählten Datei an den gewählten Ort. Der Rest des Kommentars bleibt, wo er ist.
batch-action-move-in-out-into-videos = Aus den Kommentaren in die Videos (Adobe-XMP-Marker)
batch-action-move-in-out-into-comments = Aus den Videos (XMP-Marker) in die Kommentare

batch-action-markers-comment = Marker ⇄ Kommentar
batch-action-markers-comment-to-markers = Kommentarzeilen mit Zeitangabe in Marker
batch-action-markers-to-comment = Marker in den Kommentar (eine Kopie: die Marker bleiben)
batch-action-markers-comment-hint = Eine Zeile wie „03:24 — Take 3 — schönes Licht“ ist ein Marker bei 3:24 mit dem Namen „Take 3“ und dem Kommentar „schönes Licht“; „0:41-0:47 — Löwe“ ist ein Marker von 0:41 bis 0:47. Name und Kommentar werden am ersten „ — “ oder „ -- “ getrennt, nicht an einem einfachen „ - “. Die Momente einer KI-Beschreibung („0:00–0:14 Straße.“) werden zu weißen Markern und bleiben in der Beschreibung. Ein erneuter Lauf in beiden Richtungen fügt nichts doppelt hinzu.

batch-action-tag-commented = Kommentierte Videos taggen
batch-action-tag-commented-hint = Fügt jedem ausgewählten Video mit einem Kommentar von dir den Tag „{ $tag }“ hinzu (KI-Beschreibungen zählen nicht) und entfernt ihn von denen ohne. Dateien, deren Tag sich ändert, werden umbenannt.
batch-action-tag-commented-hint-off = Fügt jedem ausgewählten Video mit einem Kommentar von dir den Tag für Videos mit Kommentar hinzu und entfernt ihn von denen ohne. Der Tag ist in den Einstellungen ausgeschaltet.
batch-action-fix-tags = Tags nach Priorität korrigieren
batch-action-fix-tags-hint = Bringt die Tags im Namen jeder ausgewählten Datei in die Reihenfolge der Tag-Leiste: Je höher ein Tag dort steht, desto früher kommt er im Namen. Tags, die der Ordner noch nicht kennt, kommen zuerst, wie in der Tag-Leiste. Dateien, deren Reihenfolge sich ändert, werden umbenannt.

batch-action-respace-tags = Tag-Abstand anwenden
batch-action-respace-tags-hint-space = Benennt jede ausgewählte Datei so um, dass nach jedem Tag ein Leerzeichen steht, wie in den Einstellungen festgelegt: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Benennt jede ausgewählte Datei so um, dass nach ihren Tags kein Leerzeichen steht, wie in den Einstellungen festgelegt: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Abstand: ein Leerzeichen nach jedem Tag
batch-action-respace-tags-status-no-space = Abstand: kein Leerzeichen nach Tags
batch-action-export-markers = Marker als Text exportieren
batch-action-export-markers-hint = Schreibt die Marker jeder ausgewählten Datei als Textdatei neben sie, clip.markers.txt: eine Zeile pro Marker in zeitlicher Reihenfolge, etwa „0:41–0:47 [red] — Löwe quert — geht von links nach rechts“. Eine andere Farbe als Grün steht in eckigen Klammern; der Kommentar folgt auf den Namen. „Marker ⇄ Kommentar“ liest solche Zeilen wieder als Marker ein. Eine Datei ohne Marker bekommt keine Textdatei, und eine Textdatei aus einem früheren Lauf wird ersetzt.
batch-export-markers-cannot-hold = Diese Datei kann keine Marker aufnehmen
batch-action-reload-files = Cache zurücksetzen und neu laden
batch-action-reload-files-hint = Liest den Kommentar und die In-/Out-Punkte jeder ausgewählten Datei erneut aus der Datei selbst und ersetzt, was sich der Ordner dafür gemerkt hat. Nutze es, nachdem die Dateien in einem anderen Programm geändert wurden. Dateien, deren gemerkte Werte fehlten oder veraltet waren, zählen als geändert.

batch-action-describe-ai = Mit KI beschreiben
batch-action-describe-ai-run = { $videos } beschreiben · etwa { $dollars }
batch-action-describe-ai-estimating = Wird geschätzt… { $known } / { $total }
batch-action-describe-ai-none = Keine Videos zu beschreiben.
batch-action-describe-ai-hint = { batch-runs-in-background }. „Abbrechen“ behält die bereits beschriebenen Videos; ein erneuter Lauf überspringt sie.
batch-action-describe-ai-no-subtitles = Ohne Untertitel (nur das Bild wird beschrieben): { $videos }.
batch-action-describe-ai-redo = Videos mit vorhandener KI-Beschreibung erneut bearbeiten
batch-action-describe-ai-hint-panel = Beschreibt, was in jedem ausgewählten Video passiert und wann: Eine Zusammenfassung und Abschnitte mit Zeitbereichen kommen in die KI-Beschreibung seines Kommentars; dein eigener Text bleibt erhalten. Bilder und Untertitel werden an { $provider } gesendet, den Anbieter des gewählten Modells.
batch-ai-change = Ändern
batch-ai-tag-suggestions-off = Aus
batch-ai-tag-suggestions-no-tags = An, aber dieser Ordner hat noch keine Tags
batch-ai-tag-suggestions-on = { $tags ->
    [one] Aus dem { $tags } Tag des Ordners
   *[other] Aus den { $tags } Tags des Ordners
}
batch-ai-key-missing = Lege in den Einstellungen einen { $provider }-API-Schlüssel fest
batch-ai-key-unavailable = Der System-Schlüsselbund konnte nicht geöffnet werden: Er ist vielleicht gesperrt, oder es gibt keinen (wie GNOME Keyring oder KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Wie die Untertitel
ai-language-english = Englisch
ai-language-russian = Russisch
ai-language-ukrainian = Ukrainisch
ai-language-german = Deutsch
ai-language-spanish = Spanisch
ai-language-french = Französisch
batch-videos-count = { $n ->
    [one] { $n } Video
   *[other] { $n } Videos
}
batch-minutes = { $n } Min.
batch-ai-minutes-under = < 1 Min.
batch-ai-duration-under-minute = unter einer Minute
batch-ai-duration-hours = { $h } Std.
batch-ai-duration-hours-minutes = { $h } Std. { $m } Min.
batch-ai-dollars-under = unter $0.01
batch-ai-skip-described = schon beschrieben
batch-ai-skip-too-long = über 30 Min.
batch-ai-skip-unreadable = nicht lesbar
batch-ai-skipped = Übersprungen: { $parts }.
batch-ai-progress-frame = Bild { $done } von { $total }
batch-ai-progress-waiting = warte auf { $provider }
batch-ai-progress-saving = wird gespeichert
batch-ai-progress-retry-rate-limit = neuer Versuch in { $seconds } s (Ratenbegrenzung)
batch-ai-progress-retry-temporary = neuer Versuch in { $seconds } s (vorübergehender Fehler)
batch-ai-stop-no-key = Beendet: kein { $provider }-API-Schlüssel. Lege in den Einstellungen einen fest.
batch-ai-fail-no-key = Kein API-Schlüssel
batch-ai-fail-unreadable = Das Video konnte nicht gelesen werden
batch-ai-fail-too-long = Der Clip ist zu lang für die KI (über 30 Min.)
batch-ai-fail-not-saved = Die Beschreibung konnte nicht gespeichert werden
batch-ai-stop-offline = Beendet: keine Verbindung zu { $provider }. Starte erneut, um den Rest zu beschreiben.

batch-action-generate-subtitles = Untertitel erzeugen
batch-action-generate-subtitles-install-ffmpeg = um .mkv, .m2ts, .avi … zu lesen, installiere ffmpeg von ffmpeg.org, füge es zu PATH hinzu und starte frename neu
batch-action-generate-subtitles-replace = Vorhandene Untertitel ersetzen
batch-action-generate-subtitles-replace-hint = Transkribiert erneut; kostet wie angezeigt.
batch-action-generate-subtitles-srt = SRT-Untertitel (clip.srt)
batch-action-generate-subtitles-premiere = Premiere-Pro-Transkript (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = In Premiere Pro: Bedienfeld „Text“ → Transkript → Statisches Transkript importieren.
batch-action-generate-subtitles-privacy = Das Audio dieser Videos wird an Soniox gesendet und dort danach gelöscht.
batch-action-generate-subtitles-duration-hint = Dauert einige Minuten pro Stunde Audio. { batch-runs-in-background }. Wenn frename geschlossen wird, endet der Lauf; fertige Untertitel bleiben erhalten.
batch-action-generate-subtitles-hint = Transkribiert die Sprache jedes ausgewählten Videos mit Soniox und speichert das Ergebnis daneben: Untertitel (clip.srt), wo frename sie anzeigt, und/oder ein Premiere-Pro-Transkript.
batch-subtitles-transcribe = Transkribieren
batch-subtitles-transcribe-count = { $videos } transkribieren
batch-subtitles-build-free = { $count } erstellen (kostenlos)
batch-subtitles-nothing = Nichts zu transkribieren
batch-subtitles-estimating = Wird geschätzt…
batch-subtitles-key-missing = Lege in den Einstellungen einen Soniox-API-Schlüssel fest
batch-subtitles-key-rejected = Soniox hat den Schlüssel abgelehnt
batch-subtitles-cost-unknown = Kosten unbekannt
batch-subtitles-typical-price = { $usd } (üblicher Preis)
batch-subtitles-unknown-length = + { $n } mit unbekannter Länge
batch-subtitles-plan-line = { $videos } zu transkribieren, { $duration } Audio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } hat schon Untertitel
   *[other] { $n } haben schon Untertitel
}
batch-subtitles-rebuilt-free = { $n } kostenlos aus einem gespeicherten Transkript neu erstellt
batch-subtitles-no-speech-before = { $n ->
    [one] { $n } hatte beim letzten Mal keine Sprache
   *[other] { $n } hatten beim letzten Mal keine Sprache
}
batch-subtitles-no-audio = { $n ->
    [one] { $n } hat kein Audio
   *[other] { $n } haben kein Audio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } teilt den Untertitelnamen mit einem anderen Video
   *[other] { $n } teilen den Untertitelnamen mit einem anderen Video
}
batch-subtitles-unreadable = { $n } nicht lesbar ({ $how_to })
batch-subtitles-shared-name-reason = teilt den Untertitelnamen mit einem anderen Video
batch-subtitles-unsupported-reason = Audioformat nicht unterstützt
batch-subtitles-progress-transcribing = wird transkribiert…
batch-subtitles-progress-extracting = Audio wird extrahiert { $percent } %
batch-subtitles-progress-extracting-unknown = Audio wird extrahiert
batch-subtitles-progress-uploading = wird hochgeladen { $percent } %
batch-subtitles-progress-uploading-unknown = wird hochgeladen
batch-subtitles-progress-transcribing-left = wird transkribiert… noch etwa { $time }
batch-subtitles-progress-saving = wird gespeichert
batch-subtitles-reason-already = hatte schon Untertitel
batch-subtitles-reason-no-audio = kein Audio
batch-subtitles-reason-no-speech = keine Sprache
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox nicht erreichbar
batch-subtitles-stop-key-rejected = Beendet: Soniox hat den Schlüssel abgelehnt. Prüfe ihn unter Einstellungen → Untertitel.
batch-subtitles-stop-balance-empty = Beendet: Das Soniox-Guthaben ist aufgebraucht. Lade es auf console.soniox.com auf.
batch-subtitles-stop-budget-used = Beendet: Das monatliche Soniox-Budget ist aufgebraucht. Erhöhe es auf console.soniox.com.
batch-subtitles-stop-other = Beendet: Soniox hat die Fortsetzung verweigert ({ $error_type }).
batch-subtitles-stop-unreachable = Beendet: Soniox ist nicht erreichbar. Prüfe die Internetverbindung.
batch-subtitles-report-spend = Soniox: mindestens { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } Upload konnte
   *[other] { $n } Uploads konnten
} bei Soniox nicht gelöscht werden (siehe Protokoll)
batch-subtitles-languages-need-key = Speichere einen Soniox-Schlüssel, um aus allen Sprachen zu wählen.
batch-subtitles-languages-rejected = Soniox hat den Schlüssel abgelehnt.
batch-subtitles-languages-failed = Die Sprachen konnten nicht von Soniox geholt werden.
batch-subtitles-count = { $n ->
    [one] { $n } Untertitel
   *[other] { $n } Untertitel
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = weniger als $0.01
batch-subtitles-usd-about = etwa ${ $amount }

## File list
folder-has-subtitles = Hat Untertitel
folder-closed-by-job = Gesperrt, solange eine Stapelaktion es ändert
folder-filter-tip = Nur anzeigen…

folder-search-placeholder = Datei oder Kommentar suchen
folder-search-comments-loading = Kommentare werden durchsucht… noch { $n }
folder-search-clear = Leeren
folder-all = Alle
folder-invert = Umkehren
folder-checked = { $count } ausgewählt
folder-outcome-changed = Geändert
folder-outcome-unchanged = Nichts zu ändern
folder-outcome-failed = Fehlgeschlagen, siehe Protokoll
folder-rename-error-empty = Der Name ist leer
folder-rename-error-bad-character = Nicht erlaubt: \ / : * ? " < > |
folder-rename-error-trailing = Darf nicht mit Punkt oder Leerzeichen enden
folder-rename-error-exists = Eine Datei mit diesem Namen existiert bereits
folder-markers-not-saved = Marker nicht gespeichert: Die Datei ist schreibgeschützt oder in Gebrauch (schließe sie in Premiere, öffne dann die Datei und verlasse sie wieder)
drag-out-not-saved = Nicht gezogen: Die Datei kann nicht gespeichert werden (schreibgeschützt oder in Premiere geöffnet)
folder-opening = Ordner wird geöffnet…
folder-empty-title = Keine Videos in diesem Ordner
folder-empty-line = frename zeigt MP4-, MOV-, MKV- und andere Videodateien.
folder-open-another = Anderen Ordner öffnen…
folder-no-match = Keine passenden Dateien
folder-show-all = Alle anzeigen
folder-locked = Die Auswahl ist gesperrt, solange { $action } läuft
folder-checked-hidden = { $count } ausgewählt · { $hidden } ausgeblendet
folder-checked-hidden-tip = { $hidden ->
    [one] 1 ausgewählte Datei ist durch die Suche oder einen Filter ausgeblendet; die Aktion läuft auch für sie
   *[other] { $hidden } ausgewählte Dateien sind durch die Suche oder einen Filter ausgeblendet; die Aktion läuft auch für sie
}
folder-outcome-working = In Arbeit
folder-outcome-not-reached = Nicht erreicht
folder-window-empty-title = Einen Ordner mit Clips öffnen
folder-window-empty-line = Oder ziehe einen Ordner auf das Fenster. Ein Rechtsklick auf die Schaltfläche öffnet eine einzelne Datei.
folder-window-open = Ordner öffnen…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Zuletzt geöffnete Ordner
recent-folders-title = Zuletzt geöffnete Ordner
recent-folders-none = Noch keine Ordner geöffnet
recent-folders-not-found = Nicht gefunden
recent-folders-remove-tip = Aus der Liste entfernen
recent-folders-clear = Liste leeren
recent-folders-remove = Entfernen
recent-folders-keep = Behalten
recent-folders-missing-line = Nicht gefunden. Entfernen?
recent-folders-just-now = Gerade eben
recent-folders-minutes = vor { $count } Min.
recent-folders-hours = vor { $count } Std.
recent-folders-days = { $count ->
    [one] vor { $count } Tag
   *[other] vor { $count } Tagen
}
recent-folders-weeks = { $count ->
    [one] vor { $count } Woche
   *[other] vor { $count } Wochen
}
recent-folders-months = { $count ->
    [one] vor { $count } Monat
   *[other] vor { $count } Monaten
}
recent-folders-years = { $count ->
    [one] vor { $count } Jahr
   *[other] vor { $count } Jahren
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Im Explorer anzeigen
file-menu-show-in-file-manager = Im Dateimanager anzeigen
file-menu-copy-path = Vollständigen Pfad kopieren
file-menu-copy-name = Dateinamen kopieren
file-menu-copied = Kopiert
file-menu-not-copied = Nicht kopiert: Die Zwischenablage ist nicht verfügbar
file-menu-not-shown = Der Dateimanager konnte nicht geöffnet werden

## Controls bar under the file list

folder-controls-filter-untagged = Ohne Tags
folder-controls-filter-subtitles = Untertitel
folder-controls-filter-comments = Kommentare
folder-controls-filter-markers = Marker
folder-controls-scroll = Die geöffnete Datei in der Liste zeigen
folder-controls-open = Ordner öffnen
folder-controls-batch = Stapelaktionen für ausgewählte Dateien
folder-controls-batch-back = Zurück zur geöffneten Datei
folder-controls-update-available = Update verfügbar: { $version }
folder-controls-previous = Vorherige Datei
folder-controls-next = Nächste Datei
folder-controls-more = Mehr
folder-controls-open-file = Rechtsklick: einzelne Datei öffnen

## Video

video-controls-back = 10 s zurück
video-controls-play = Abspielen
video-controls-pause = Pause
video-controls-forward = 10 s vor
video-controls-frame-back = Ein Bild zurück
video-controls-frame-forward = Ein Bild vor
video-controls-set-in = In-Punkt setzen
video-controls-set-out = Out-Punkt setzen
video-controls-screenshot = Dieses Bild speichern
video-controls-add-marker = Marker hinzufügen
video-controls-add-marker-hold = Gedrückt halten für einen Bereich; erneut drücken zum Benennen
video-controls-cannot-hold-markers = Diese Datei kann keine Marker aufnehmen
video-controls-add-a-name = Namen hinzufügen
video-controls-rotate-left = Nach links drehen
video-controls-rotate-right = Nach rechts drehen
video-controls-more = Mehr
media-viewer-loading-slow = Warte auf die Datei… (eine Cloud-Datei kann etwas dauern)
media-viewer-cannot-play = Dieser Clip kann nicht abgespielt werden
media-viewer-no-picture = Diese Datei hat kein Videobild
video-controls-volume-scroll = Lautstärke – zum Ändern scrollen
rotate-cannot = Drehen nicht möglich: { $reason }
rotate-reason-missing = die Datei ist nicht mehr da
rotate-flag-right = 90° rechts
rotate-flag-left = 90° links
rotate-flag-half = 180°
rotate-flag-none = keine
rotate-now = Drehung: { $flag }
rotate-turned = Gedreht um { $turn } · Drehung jetzt { $flag }
rotate-failed = Nicht gedreht: { $reason }
undo-name-taken = Nicht geändert: Eine andere Datei heißt schon { $name }
undo-back-on-clip = Zurück bei diesem Clip (die Verschiebung wurde rückgängig gemacht). Strg+Z macht seine letzte Änderung rückgängig.
redo-on-clip = Wieder bei diesem Clip (die Verschiebung wurde wiederholt).
rotate-reason-in-use = die Datei ist schreibgeschützt oder in Gebrauch
rotate-reason-format = dieses Format hat kein Rotations-Flag
rotate-reason-damaged = die Datei ist beschädigt
rotate-reason-no-video = die Datei hat keine Videospur
rotate-reason-matrix = das Video hat eine ungewöhnliche Anzeigematrix
batch-action-rotate = Videos drehen
batch-action-rotate-hint = Ändert das Rotations-Flag jeder ausgewählten MP4- und MOV-Datei; das Bild wird nicht neu kodiert, und Kommentare, In-/Out-Punkte und Marker bleiben. Premiere Pro sollte den Clip beim Import gedreht anzeigen. Andere Formate haben kein Rotations-Flag und schlagen fehl. Ein erneuter Lauf dreht die Dateien erneut; „Zurücksetzen“ entfernt jede Drehung, auch eine vom Handy aufgezeichnete, sodass ein Hochformat-Clip vom Handy danach auf der Seite liegt.
batch-action-rotate-right = 90° rechts (im Uhrzeigersinn)
batch-action-rotate-left = 90° links (gegen den Uhrzeigersinn)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Zurücksetzen: keine Drehung (0°)
media-viewer-video-subtitle-list = Untertitelliste
media-viewer-video-marker-list = Markerliste
media-viewer-video-markers-hint = Umschalt+F1 / Umschalt+F3 springen zwischen Markern; Umschalt+Ziehen rastet ein
media-viewer-video-tab-markers = Marker
media-viewer-video-fullscreen = Vollbild
media-viewer-video-close-list = Liste schließen
media-viewer-video-subtitle-file-unreadable = Diese Datei konnte nicht gelesen werden
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Fortgesetzt bei { $time } · Pos1: von vorn

## Markers list
markers-in-out = In-/Out-Punkte

markers-empty = Noch keine Marker
markers-add = Marker hinzufügen
markers-ai-hint = KI-Marker: wird ersetzt, wenn die KI diesen Clip erneut beschreibt
markers-keep-color = Farbe behalten
markers-done = Fertig
markers-delete = Marker löschen
markers-cannot-hold-hint = Premiere liest Marker aus MP4- und MOV-Dateien.
markers-color-green = Grün
markers-color-red = Rot
markers-color-orange = Orange
markers-color-yellow = Gelb
markers-color-white = Weiß
markers-color-blue = Blau
markers-color-cyan = Cyan
markers-color-lavender = Lavendel
markers-color-magenta = Magenta
markers-color-other = Andere Farbe
markers-read-only = schreibgeschützt
markers-name-placeholder = Name
markers-ai-describe = Mit KI beschreiben: den Marker benennen und ergänzen, was passiert
markers-ai-stop = Beschreiben beenden
markers-ai-describe-unnamed = { $count } unbenannte beschreiben
markers-ai-describe-unnamed-hint = Benennt und beschreibt jeden Marker ohne Namen mit KI, einige gleichzeitig. Eine Anfrage pro Marker; ein Name, den du inzwischen eingibst, bleibt erhalten.
markers-ai-price-about = etwa { $dollars }
markers-ai-describe-unnamed-cost = Eine Anfrage pro Marker: { $each } je Marker, { $total } für alle.
markers-ai-confirm = { $count ->
    [one] { $count } Marker beschreiben? Eine Anfrage: { $total }.
   *[other] { $count } Marker beschreiben? Eine Anfrage pro Marker: { $total } insgesamt.
}
markers-ai-confirm-unpriced = { $count ->
    [one] { $count } Marker beschreiben? Eine kostenpflichtige Anfrage.
   *[other] { $count } Marker beschreiben? Je eine kostenpflichtige Anfrage.
}
markers-ai-confirm-yes = Beschreiben
markers-ai-confirm-no = Abbrechen
markers-ai-waiting = Wartet auf seinen Zug…
markers-ai-run = Beschreibe { $describing } · wartend: { $waiting }
markers-ai-run-last = Beschreibe { $describing }
markers-ai-stop-all = Alle beenden
markers-ai-describing = Wird beschrieben…
markers-ai-done = Marker beschrieben
markers-ai-nothing-new = Der Marker hat diese Beschreibung schon
markers-no-marker-notice = Hier ist kein Marker
markers-ai-no-key = Kein { $provider }-API-Schlüssel: lege in den Einstellungen einen fest
markers-ai-failed = Nicht beschrieben: { $reason }
markers-ai-failed-unknown = etwas ist schiefgegangen
markers-read-only-notice = Dieser Marker ist schreibgeschützt
markers-ai-gone = Der Marker ist weg: Seine Beschreibung wurde nicht hinzugefügt
markers-ai-stopping-for-batch = Die Markerbeschreibungen werden zuerst beendet… Das dauert meist ein paar Sekunden.
folder-not-saved-marker-ai = Nicht gespeichert: Die Markerbeschreibung las den Clip noch. Versuche es gleich noch einmal.

## File workspace

file-workspace-search-placeholder = Tag suchen – oder einfach tippen
file-workspace-search-clear = Leeren
file-workspace-comment-placeholder = Kommentar...
file-workspace-comment-collapse = Zurück zu den Tags
file-workspace-comment-expand = Kommentar vergrößern

## Tag grid

tag-grid-star = Stern: oben halten
tag-grid-unstar = Stern entfernen
tag-grid-save = Zu den Tags des Ordners hinzufügen
tag-grid-delete = „{ $tag }“ aus den Tags des Ordners löschen
tag-grid-create = „{ $tag }“ erstellen
tag-grid-no-file = Öffne einen Clip, um ihn zu taggen
tag-grid-no-tags = Noch keine Tags
tag-grid-no-tags-hint = Tippe einen Namen und drücke Enter, um den ersten zu erstellen.
tag-grid-group-unsaved = Nicht in den Tags des Ordners
tag-grid-group-folder = Tags des Ordners
tag-grid-more = +{ $count } weitere

## Order strip

sync-panel-locked = Umordnen unten ordnet den Ordner um
sync-panel-unlocked = Umordnen unten ändert nur diesen Clip
sync-panel-unlock = Entsperren
sync-panel-lock = Sperren
sync-panel-differs = Die Reihenfolge weicht vom Ordner ab
sync-panel-use-for-folder = Für den Ordner übernehmen
sync-panel-sort-like-folder = Wie der Ordner sortieren

## File name card

file-name-panel-no-tags = Keine Tags an diesem Clip
file-name-panel-untag = Tag entfernen
file-name-panel-clear-in = In-Punkt löschen
file-name-panel-clear-out = Out-Punkt löschen
file-name-panel-suggested-in-out = KI
file-name-panel-apply-suggested-in-out = In und Out auf den Vorschlag der KI setzen (auf ganze Sekunden gerundet)
file-name-panel-suggested-tags = KI-Tags
file-name-panel-add-suggested-tag = Tag „{ $tag }“ hinzufügen ({ $percent } % sicher)
file-name-panel-add-suggested-tag-unsure = Tag „{ $tag }“ hinzufügen
file-name-panel-add-all-suggested-tags = Alle hinzufügen
file-name-panel-add-all-suggested-tags-tip = Jeden von der KI vorgeschlagenen Tag hinzufügen
file-name-panel-tag-ideas = Ideen: { $ideas }
file-name-panel-tag-ideas-tip = Tags, die der KI aufgefallen sind und die dieser Ordner nicht hat. Sie werden nicht hinzugefügt.
file-name-panel-saved = Alle Änderungen gespeichert
file-name-panel-saved-to-recovery = In der Wiederherstellung gespeichert, wird beim Verlassen des Clips angewendet

## Updates (Settings)

updates-check = Nach Updates suchen
updates-not-installed = Updates funktionieren in der installierten Version
updates-from-store = Updates kommen aus dem Microsoft Store
updates-from-releases = Neue Versionen: von der Releases-Seite herunterladen
updates-open-releases = Releases-Seite öffnen
updates-checking = Wird geprüft…
updates-downloading-named = { $version } wird heruntergeladen… { $percent } %
updates-downloading = Wird heruntergeladen… { $percent } %
updates-restarting = Neustart…
updates-check-failed = Die Suche nach Updates ist fehlgeschlagen: { $reason }
updates-update-failed = Das Update ist fehlgeschlagen: { $reason }
updates-close-other-window = Schließe zuerst das andere frename-Fenster
updates-reason-github = GitHub hat nicht geantwortet, versuche es später
updates-reason-no-connection = keine Verbindung
updates-reason-read-only = der Ordner ist schreibgeschützt
updates-reason-not-installed = Updates funktionieren in der installierten Version
updates-reason-download-stopped = der Download wurde unerwartet beendet
updates-version-available = Version { $version } ist verfügbar
updates-up-to-date = frename ist aktuell
updates-update-and-restart = Aktualisieren und neu starten
updates-wait-for-batch = Warte, bis die Stapelaktion fertig ist
updates-current-version = frename { $version }
updates-check-on-start = Beim Start von frename nach Updates suchen

recovery-restored = frename wurde unerwartet beendet. Deine nicht gespeicherte Arbeit an { $clip } ({ $what }) wurde wiederhergestellt.
recovery-kept = frename wurde unerwartet beendet. Deine nicht gespeicherte Arbeit an { $clip } ({ $what }) wurde nicht angewendet: { $why }. Eine lesbare Kopie zum Abtippen liegt in { $folder }.
recovery-unreadable = frename wurde unerwartet beendet, und eine Wiederherstellungsdatei konnte nicht gelesen werden. Sie liegt in { $folder }.
recovery-why-gone = der Clip ist weg oder wurde umbenannt
recovery-why-changed = der Clip hat sich nach deinen Änderungen geändert
recovery-why-not-written = eine Datei mit diesem Namen existiert, oder der Clip ist schreibgeschützt oder in Gebrauch
recovery-tags = { $count ->
    [one] { $count } Tag
   *[other] { $count } Tags
}
recovery-markers = { $count ->
    [one] { $count } Marker
   *[other] { $count } Marker
}
recovery-comment = Kommentar
recovery-in-out = In-/Out-Punkte
recovery-edits = Änderungen
folder-frame-saved = Bild gespeichert
folder-not-saved = Nicht gespeichert: Eine Datei mit diesem Namen existiert schon, oder sie ist schreibgeschützt oder in Gebrauch
markers-deleted-notice = Marker gelöscht
markers-not-saved-notice = Marker nicht gespeichert: Die Datei ist schreibgeschützt oder in Gebrauch
markers-comment-not-saved-notice = Marker nicht gespeichert: Die Kommentardatei ist schreibgeschützt oder in Gebrauch
markers-copy = Kopieren
markers-copy-tip = Die Marker als Text kopieren, je eine Zeile
markers-export = Exportieren…
markers-export-tip = Die Marker als Textdatei speichern
markers-exported = Marker in { $name } gespeichert
markers-not-exported = Marker nicht gespeichert: Die Datei konnte nicht geschrieben werden
