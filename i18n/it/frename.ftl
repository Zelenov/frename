# frename UI text in Italian. Same ids, attributes and $arguments as i18n/en/frename.ftl, in the
# same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms; the reader is addressed
# impersonally (infinitives or noun phrases for actions and labels, «si» constructions in
# sentences), never with «tu», and with «Lei» only where a sentence cannot do without a subject.
# Terms follow the Italian Premiere Pro.
# Typography: no space before «:», «;», «!», «?» or «%» (50%); quotes are «…»; decimal comma
# (0,01 $, 29,97 fps); a plain apostrophe (l'IA).
# Keys: Ctrl stays Ctrl, Shift is Maiusc, Enter is Invio, Home stays Home (as on Italian keyboards).
#   tag / tags                        tag / tag, masculine, invariable (never «etichetta»: that is
#                                     Premiere's Label)
#   in / out points                   punti di attacco/stacco (Premiere: Contrassegna attacco/stacco)
#   comment                           commento
#   subtitles                         sottotitoli
#   screenshot / frame                fotogramma
#   marker (Premiere)                 marcatore
#   range (a marker with a duration)  intervallo
#   subclip (Premiere)                sottoclip
#   Description column (Premiere)     colonna «Descrizione»
#   checked (files)                   selezionato
#   All / Invert (check boxes)        Tutti / Invertire
#   video / clip / file               video (m.) / clip (f., as in Premiere) / file (m.), all invariable
#   tag panel / tag list              pannello dei tag / elenco dei tag (the folder's list of tags)
#   file name                         nome del file
#   a tag name inside a sentence      in «…» quotes: «Commented»
#   untagged (filter)                 Senza tag
#   changed / unchanged / failed      modificato / niente da cambiare / errore
#   Cancel / Stopping…                Annullare / Arresto in corso…
#   log, "see the log"                registro, «vedere il registro»
#   System (language list)            Sistema (…)
#   batch action / batch mode         azione batch / modalità batch
#   request (to the AI)               richiesta
#   moments (AI)                      momenti
#   file list / folder                elenco dei file / cartella (feminine)
#   settings                          Impostazioni
#   update (a newer frename)          aggiornamento
#   AI description (Describe with AI) descrizione con IA (Descrivere con IA)
#   subtitles generation (Soniox)     trascrizione / sottotitoli
#   cue (a subtitle line)             sottotitolo
#   keyring                           portachiavi (Portachiavi di macOS, Gestione credenziali di Windows)
#   timecode / frame rate / drop-frame  timecode / frequenza fotogrammi / drop frame
#   playhead                          indicatore di riproduzione
#
# Plurals: Italian has the categories one / other here, other as the default. «one» is only 1
# (0 marcatori, 1 marcatore, 2 marcatori). video, clip, file and tag do not change in the plural,
# but participles and adjectives agree in each variant (file selezionato / file selezionati, clip
# descritta); where a figure stands alone, invariable phrases (senza esito, da elaborare) avoid
# guessing the gender. Store names ($store) carry no article, since «in» + article contracts.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский
language-name-de = Deutsch
language-name-fr = Français
language-name-es = Español
language-name-it = Italiano
language-name-pt-BR = Português (Brasil)
language-name-uk = Українська
language-name-sr-Cyrl = Српски (ћирилица)
language-name-sr-Latn = Srpski (latinica)
language-name-ja = 日本語
language-name-ko = 한국어
language-name-zh-Hans = 简体中文
language-name-zh-Hant = 繁體中文

## Settings window

settings-window-title = Impostazioni
settings-page-interface = Interfaccia
settings-page-saving = Salvataggio
settings-page-ai = Descrizione con IA
settings-close = Chiudere
settings-apply-note = Le modifiche si applicano subito. Tab passa da un'opzione all'altra, Ctrl+Tab da una pagina all'altra.
settings-language = Lingua
settings-language-system = Sistema ({ $language })
settings-video = Video
settings-video-autoplay = Riprodurre automaticamente i video all'apertura
settings-subtitle-size = Dimensione dei sottotitoli
settings-subtitle-size-small = Piccola
settings-subtitle-size-medium = Media
settings-subtitle-size-large = Grande
settings-subtitle-size-extra-large = Molto grande
settings-timecode = Formato del tempo
settings-timecode-hint = Come si scrivono i tempi sotto il video, negli elenchi dei marcatori e dei sottotitoli e sugli indicatori In / Out, e il tempo dell'indicatore di riproduzione che copia Ctrl+Maiusc+C. Premiere Pro e DaVinci Resolve mostrano ore:minuti:secondi:fotogrammi alla frequenza fotogrammi della clip; 29,97 e 59,94 fps sono drop frame. Una clip senza una frequenza fotogrammi costante mantiene il formato predefinito. I file mantengono i propri formati.
settings-timecode-default = Predefinito (mm:ss)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Colori dei tag
settings-tags-monochrome = Monocromatico
settings-tags-monochrome-hint = I tag aggiunti appaiono in un grigio più scuro, gli altri in uno più chiaro, senza altri colori.
settings-file-names = Nomi dei file
settings-tags-space-after = Spazio dopo ogni tag
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = I file mantengono il proprio nome finché non vengono rinominati o salvati.
settings-tags-space-add = Aggiungere lo spazio ai nomi dei file…
settings-tags-space-remove = Togliere lo spazio dai nomi dei file…
settings-comments = Commenti
settings-comments-in-video = Dentro il file video
settings-comments-in-video-hint = XMP, la colonna «Descrizione» di Premiere Pro
settings-comments-text-file = In un file di testo accanto al video
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = I file mantengono i commenti dove sono finché non vengono spostati.
settings-comments-move-into-videos = Spostare i commenti nei video…
settings-comments-move-into-text-files = Spostare i commenti in file di testo…
settings-commented-tag = Aggiungere un tag ai video con un commento
settings-commented-tag-name = Tag
settings-commented-tag-hint = Si aggiunge quando si scrive un commento su un video, ad es. { $tag }.IMG_0424.MOV, e si toglie quando lo si cancella (le descrizioni con IA non contano). Per il resto si può cambiare liberamente.
settings-commented-tag-off = I video con un commento non ricevono alcun tag.
settings-in-out = Punti di attacco/stacco
settings-in-out-in-video = Dentro il file video
settings-in-out-in-video-hint = Un marcatore di sottoclip in Premiere Pro
settings-in-out-comment = Nel commento, su una riga
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = I file mantengono i punti di attacco/stacco dove sono finché non vengono spostati.
settings-in-out-move-into-videos = Spostare i punti di attacco/stacco nei video…
settings-in-out-move-into-comments = Spostare i punti di attacco/stacco nei commenti…
settings-markers = Marcatori e intervalli
settings-markers-in-video = Dentro il file video
settings-markers-in-video-hint = XMP, visibili sulla clip in Premiere Pro
settings-markers-comment = Nel commento, una riga per marcatore
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = I file mantengono i marcatori dove sono finché non vengono spostati.
settings-markers-move-into-videos = Spostare i marcatori nei video…
settings-markers-copy-into-comment = Copiare i marcatori nei commenti…
settings-markers-hint = Sia punti sia intervalli, anche i momenti trovati dall'IA.
settings-updates = Aggiornamenti
settings-version = Versione
settings-release-notes = Note di rilascio
settings-old-title = Impostazioni di un frename precedente
settings-old-hint = Importare le impostazioni di un frename avviato da una cartella zip.
settings-old-scheduled = Le impostazioni verranno importate al riavvio di frename
settings-old-not-found = Nessun frename.exe con un frename.db in { $folder }
settings-old-failed = Importazione non riuscita: { $reason }
settings-old-import-button = Importare da una vecchia cartella di frename…
settings-ai = Descrizione con IA
settings-ai-key-label = Chiave API di Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Ottenere una chiave su console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Rimuovere la chiave di Anthropic salvata?
settings-ai-used-by = Usata da «Descrivere con IA» in modalità batch e sui marcatori (✨, Ctrl+F2), quando si sceglie un modello Claude.
settings-ai-openai-key-label = Chiave API di OpenAI
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Ottenere una chiave su platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Rimuovere la chiave di OpenAI salvata?
settings-ai-openai-used-by = Usata da «Descrivere con IA» in modalità batch e sui marcatori (✨, Ctrl+F2), quando si sceglie un modello GPT.
settings-ai-gemini-key-label = Chiave API di Gemini
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Ottenere una chiave su aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Rimuovere la chiave di Gemini salvata?
settings-ai-gemini-used-by = Usata da «Descrivere con IA» in modalità batch e sui marcatori (✨, Ctrl+F2), quando si sceglie un modello Gemini.
settings-ai-model-needs-key = { $model } — aggiungere prima la chiave di { $provider }
settings-ai-model-label = Modello
settings-ai-language-label = Lingua della descrizione
settings-ai-hint = Haiku, GPT-4.1 mini e Gemini Flash-Lite sono i più economici; Sonnet, Opus, GPT-4.1 e Gemini Flash colgono più dettagli. Ogni modello richiede la chiave della propria azienda.
settings-ai-moments-label = Momenti
settings-ai-moments-important = Solo ciò che spicca
settings-ai-moments-full = Coprire tutta la clip
settings-ai-moments-important-hint = Suggerisce anche dove mettere l'attacco e lo stacco (la pillola IA accanto a IN e OUT)
settings-ai-moments-full-hint = Un momento per ogni parte, anche quelle tranquille; nessun suggerimento di attacco e stacco
settings-ai-tag-suggestions = Suggerire anche tag tra quelli della cartella
settings-ai-tag-suggestions-hint = Nella stessa richiesta, l'IA indica quali tag della cartella si adattano a ogni clip. Un tag suggerito si aggiunge solo facendo clic su di esso.
settings-subtitles = Sottotitoli
settings-subtitles-key-label = Chiave API di Soniox
settings-subtitles-key-placeholder = Incollare la chiave
settings-subtitles-key-get = Ottenere una chiave su console.soniox.com. L'audio viene inviato a Soniox.
settings-subtitles-key-remove-confirm = Rimuovere la chiave di Soniox salvata?
settings-subtitles-languages-label = Lingue
settings-subtitles-languages-locked = Salvare la chiave per scegliere tra tutte le lingue che Soniox conosce.
settings-subtitles-languages-loading = Recupero delle lingue da Soniox…
settings-subtitles-languages-none = Nessuna selezionata: rilevamento automatico.
settings-subtitles-languages-hint = Le lingue parlate nel girato, come suggerimenti.
settings-subtitles-cue-length-label = Lunghezza dei sottotitoli
settings-subtitles-cue-short = Breve
settings-subtitles-cue-short-hint = Una riga, fino a 8 s
settings-subtitles-cue-sentence = Una frase
settings-subtitles-hint = Valgono per «Generare sottotitoli» in modalità batch. La lunghezza si applica ai nuovi sottotitoli.
settings-key-store-windows = Gestione credenziali di Windows
settings-key-store-macos = Portachiavi di macOS
settings-key-store-other = portachiavi di sistema
settings-key-unavailable = Impossibile aprire il portachiavi di sistema
settings-key-unavailable-hint = Potrebbe essere bloccato o non esistere (come GNOME Keyring o KWallet). Le impostazioni lo verificano di nuovo a ogni apertura.
settings-key-remove-confirm-hint = Sarà necessario incollarla di nuovo.
settings-key-remove-ask = Rimuovere…
settings-key-remove = Rimuovere la chiave
settings-key-keep = Mantenere
settings-key-replace = Sostituire…
settings-key-hide = Nascondere
settings-key-show = Mostrare
settings-key-save = Salvare la chiave
settings-key-get = Ottenere una chiave
settings-key-billing = Aggiungere credito
settings-key-cancel = Annullare
settings-key-saved-in = Salvata in { $store } su questo computer
settings-key-save-into = «Salvare la chiave» la conserva in { $store } su questo computer.

## Batch mode

batch-checked-count = { $count ->
    [one] { $count } file selezionato
   *[other] { $count } file selezionati
}
batch-no-key = Nessuna chiave
batch-group-move = Spostare tra posizioni
batch-group-fix = Correggere nomi e video
batch-group-paid = Servizi a pagamento
batch-reason-none-checked = Nessun file selezionato
batch-reason-reading = Lettura della durata delle clip…
batch-check-all = Selezionare { $count ->
    [one] { $count } file
   *[other] tutti i { $count } file
}
batch-job-running = in corso
batch-job-finished = terminato
batch-job-stopped = interrotto
batch-progress-files = { $finished } di { $total ->
    [one] { $total } file
   *[other] { $total } file
}
batch-time-left = circa { $time } rimanenti
batch-time-estimating = stima del tempo…
batch-time-spent = { $time } finora
batch-runs-in-background = Si può continuare a lavorare durante l'esecuzione
batch-clip-in-work = In uso: { $action }
batch-folder-held = Occupata: { $action }
batch-indicator-hint = Fare clic per il pannello e Annullare
batch-action-prepare = Preparare le clip
batch-action-prepare-hint = Per ogni clip selezionata, in un solo passaggio: i sottotitoli (Soniox) se non li ha, poi la descrizione con IA: un riepilogo, i suoi momenti come marcatori, il commento e i tag suggeriti. Un solo prezzo, un solo limite e un solo avanzamento per tutto.
batch-prepare-step-off = disattivato
batch-prepare-nothing = Niente da fare per queste clip
batch-prepare-report-cap = Spesi { $spent } su un limite di { $cap }
batch-option-steps = Passaggi
batch-prepare-step-subtitles = Creare prima i sottotitoli (Soniox)
batch-prepare-step-description = Descrivere le clip con IA
batch-prepare-cap = Smettere di spendere a
batch-prepare-cap-none = Nessun limite (dollari USA)
batch-prepare-cap-invalid = Il limite è un importo in dollari, come 5
batch-prepare-left = L'ultima esecuzione si è fermata con { $count ->
    [one] { $count } clip rimanente
   *[other] { $count } clip rimanenti
}
batch-prepare-check-left = Selezionarle
batch-prepare-estimating = Calcolo di cosa farà ogni passaggio…
batch-prepare-plan-subtitles = Sottotitoli
batch-prepare-plan-description = Descrizione
batch-prepare-plan-none = niente da fare
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } video
   *[other] { $videos } video
} da trascrivere, { $duration } di audio
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } video
   *[other] { $videos } video
} da descrivere
batch-prepare-resume-note = Ogni passaggio salta ciò che esiste già, quindi un'esecuzione interrotta si riprende eseguendola di nuovo: niente si paga due volte.
batch-prepare-run = Preparare { $clips ->
    [one] { $clips } clip
   *[other] { $clips } clip
} · circa { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Riprese simili
batch-prepare-groups-hint = Clip che sembrano la stessa ripresa: duplicati, riesportazioni, una videocamera che non si è mossa.
batch-prepare-group-clips = { $count } clip
batch-prepare-check-group = Selezionare queste clip
batch-solo-queued = In coda: { $action }
batch-suggest-in-job = Una descrizione di questa clip è già nell'azione in corso, senza tag suggeriti. Richiederla di nuovo quando termina
batch-suggest-no-tags = Questa cartella non ha ancora tag da suggerire: aggiungerne prima alcuni
batch-solo-gave-up = Preparazione non riuscita
batch-solo-withdrawn = Fuori dalla coda: { $action }
batch-solo-in-work = { $action } è in corso. Si può annullare dalla barra o dalla pagina dell'azione
batch-waiting-title = { $count ->
    [one] { $count } esecuzione in attesa
   *[other] { $count } esecuzioni in attesa
}
batch-waiting-remove = Rimuovere
batch-indicator-waiting = { $count } in attesa
file-workspace-describe-clip = Descrivere questa clip con IA
file-workspace-subtitles-clip = Generare i sottotitoli per questa clip
file-workspace-describe-clip-queued = Descrivere con IA: in attesa del proprio turno. Fare clic per ritirare la richiesta
file-workspace-describe-clip-running = Descrivere con IA: in corso
file-workspace-suggest-tags-clip = Suggerire tag per questa clip con IA (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Suggerire tag: in attesa del proprio turno. Fare clic per ritirare la richiesta
file-workspace-suggest-tags-clip-running = Suggerire tag: in corso
file-workspace-subtitles-clip-queued = Generare i sottotitoli: in attesa del proprio turno. Fare clic per ritirare la richiesta
file-workspace-subtitles-clip-running = Generare i sottotitoli: in corso
batch-run-again = Eseguire di nuovo su { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-result-done = Fatto: { $total ->
    [one] { $total } file
   *[other] { $total } file
}
batch-result-done-detail = { $changed } con modifiche, { $unchanged } senza nulla da cambiare
batch-result-problems = Fatto con problemi: { $failed } su { $total ->
    [one] { $total } file
   *[other] { $total } file
} senza esito
batch-figure-unchanged = senza modifiche
batch-figure-not-done = senza esito
batch-figure-not-reached = da elaborare
batch-files-not-done = File senza esito
batch-table-file = File
batch-table-why = Motivo

batch-run-move-comments = Spostare { $count ->
    [one] { $count } commento
   *[other] { $count } commenti
}
batch-run-move-in-out = Spostare attacco/stacco di { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-run-convert = Convertire { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-run-rotate = Ruotare { $count ->
    [one] { $count } video
   *[other] { $count } video
}
batch-run-tag = Aggiungere il tag a { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-run-fix-tags = Riordinare i tag in { $count ->
    [one] { $count } nome
   *[other] { $count } nomi
}
batch-run-rename = Rinominare { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-run-reload = Ricaricare { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-run-export-markers = Esportare i marcatori di { $count ->
    [one] { $count } file
   *[other] { $count } file
}
batch-service-soniox = Soniox
batch-change-renames = Rinomina i file
batch-change-videos = Scrive nei video
batch-change-text-files = Scrive file di testo accanto ai video
batch-change-comments = Scrive nei commenti
batch-change-subtitles = Scrive file di sottotitoli accanto ai video
batch-change-records = Cambia solo i dati interni di frename
batch-option-direction = Direzione
batch-option-language = Lingua
batch-option-tag-suggestions = Tag suggeriti
batch-option-described = Video descritti
batch-option-subtitled = Video sottotitolati
batch-option-subtitles-write = File da scrivere
batch-option-spacing = Spaziatura
batch-option-tag = Tag
batch-option-turn = Rotazione
batch-plan-videos = Video
batch-plan-length = Durata
batch-plan-cost = Costo
batch-plan-time = Tempo
batch-reason-estimate = In attesa della stima
batch-reason-subtitles-no-format = Selezionare almeno un file da scrivere
batch-set-key = Impostare la chiave…
batch-check-key = Verificare la chiave…
batch-subtitles-languages-auto = Rilevata in ogni video
batch-action-describe-ai-run-waiting = Descrivere i video
batch-action-fix-tags-order = Ordine: come nell'elenco dei tag, prima i tag sconosciuti.
batch-action-markers-comment-hint-short = Trasforma le righe del commento con un tempo in marcatori, o copia i marcatori nel commento.
batch-action-markers-to-comment-hint = I marcatori restano; eseguirla di nuovo non aggiunge nulla due volte.
batch-action-rotate-hint-short = Ruota le clip MP4 e MOV secondo il loro indicatore di rotazione, senza ricodifica.
batch-action-tag-commented-off = Il tag per i video con un commento è disattivato
batch-action-tag-commented-choose = Scegliere il tag…

batch-title = Azioni batch
batch-done-label-changed = modificati
batch-done-label-subtitled = sottotitolati
batch-ai-at-least = almeno
batch-cancel = Annullare
batch-stopping = Arresto in corso…
batch-stop-over-budget = Interrotto: la clip successiva potrebbe far superare il limite di { $cap } ({ $spent } spesi). Alzare il limite per fare il resto.
batch-stopped = Interrotto dopo { $finished } di { $total ->
    [one] { $total } file
   *[other] { $total } file
}.
batch-close = Chiudere
batch-failed-subtitles = Non sottotitolati:
batch-written-subtitles = File scritti:
batch-table-written = Scritti
batch-merged-markers = Marcatori duplicati uniti:
batch-table-merged = Uniti
batch-markers-merged = { $count ->
    [one] { $count } marcatore duplicato unito
   *[other] { $count } marcatori duplicati uniti
}
batch-add-credit = Aggiungere credito
batch-open-log = Aprire il registro

batch-action-move-comments = Spostare i commenti
batch-action-move-comments-hint = Sposta il commento di ogni file selezionato nella posizione scelta, con i punti di attacco/stacco che contiene. I tag e i punti di attacco/stacco salvati nel video restano dove sono.
batch-action-move-comments-into-videos = Dai file di testo ai video (XMP)
batch-action-move-comments-into-text-files = Dai video (XMP) ai file di testo

batch-action-move-in-out = Punti di attacco/stacco: commento ⇄ video (XMP)
batch-action-move-in-out-hint = Sposta i punti di attacco/stacco di ogni file selezionato nella posizione scelta. Il resto del commento resta dove si trova.
batch-action-move-in-out-into-videos = Dai commenti ai video (marcatore XMP di Adobe)
batch-action-move-in-out-into-comments = Dai video (marcatore XMP) ai commenti

batch-action-markers-comment = Marcatori ⇄ commento
batch-action-markers-comment-to-markers = Righe del commento con un tempo in marcatori
batch-action-markers-to-comment = Marcatori nel commento (una copia: i marcatori restano)
batch-action-markers-comment-hint = Una riga come «03:24 — Take 3 — nice light» è un marcatore a 3:24 chiamato «Take 3» con il commento «nice light»; «0:41-0:47 — Lion» è un marcatore da 0:41 a 0:47. Il nome e il commento si separano al primo « — » o « -- », non a un semplice « - ». I momenti di una descrizione con IA («0:00–0:14 Street.») diventano marcatori bianchi e restano nella descrizione. Eseguirla di nuovo in una delle due direzioni non aggiunge nulla due volte.

batch-action-tag-commented = Aggiungere un tag ai video con un commento
batch-action-tag-commented-hint = Aggiunge il tag «{ $tag }» a ogni video selezionato con un commento proprio (le descrizioni con IA non contano) e lo toglie da quelli senza. I file il cui tag cambia vengono rinominati.
batch-action-tag-commented-hint-off = Aggiunge il tag per i video con un commento a ogni video selezionato con un commento proprio e lo toglie da quelli senza. Il tag è disattivato nelle impostazioni.
batch-action-fix-tags = Correggere i tag per priorità
batch-action-fix-tags-hint = Dispone i tag nel nome di ogni file selezionato nell'ordine del pannello dei tag: più in alto si trova un tag lì, prima compare nel nome. I tag che la cartella non conosce ancora vengono per primi, come nel pannello dei tag. I file il cui ordine cambia vengono rinominati.

batch-action-respace-tags = Applicare la spaziatura dei tag
batch-action-respace-tags-hint-space = Rinomina ogni file selezionato per mettere uno spazio dopo ogni tag, come indicato nelle impostazioni: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Rinomina ogni file selezionato in modo che non ci sia spazio dopo i suoi tag, come indicato nelle impostazioni: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Spaziatura: uno spazio dopo ogni tag
batch-action-respace-tags-status-no-space = Spaziatura: nessuno spazio dopo i tag
batch-action-export-markers = Esportare i marcatori in testo
batch-action-export-markers-hint = Scrive i marcatori di ogni file selezionato in un file di testo accanto a esso, clip.markers.txt: una riga per marcatore in ordine di tempo, come «0:41–0:47 [red] — Lion crossing — walks left to right». Un colore diverso dal verde è tra parentesi quadre; il commento segue il nome. «Marcatori ⇄ commento» rilegge queste righe come marcatori. Un file senza marcatori non riceve alcun file di testo, e il file di testo di un'esecuzione precedente viene sostituito.
batch-export-markers-cannot-hold = Questo file non può contenere marcatori
batch-action-reload-files = Reimpostare la cache e ricaricare
batch-action-reload-files-hint = Rilegge dal file stesso il commento e i punti di attacco/stacco di ogni file selezionato e sostituisce ciò che la cartella ricordava. Da usare dopo aver modificato i file in un altro programma. I file i cui valori ricordati mancavano o non erano aggiornati contano come modificati.

batch-action-describe-ai = Descrivere con IA
batch-action-describe-ai-run = Descrivere { $videos } · circa { $dollars }
batch-action-describe-ai-estimating = Stima in corso… { $known } / { $total }
batch-action-describe-ai-none = Nessun video da descrivere.
batch-action-describe-ai-hint = { batch-runs-in-background }. Annullare mantiene i video già descritti; eseguirla di nuovo li salta.
batch-action-describe-ai-no-subtitles = Senza sottotitoli (si descrive solo l'immagine): { $videos }.
batch-action-describe-ai-redo = Rifare i video che hanno già una descrizione con IA
batch-action-describe-ai-hint-panel = Descrive cosa succede in ogni video selezionato, e quando: un riepilogo e segmenti con intervalli di tempo finiscono nella descrizione con IA del suo commento; il testo scritto a mano resta. Fotogrammi e sottotitoli vengono inviati a { $provider }, l'azienda del modello scelto.
batch-ai-change = Cambiare
batch-ai-tag-suggestions-off = Disattivati
batch-ai-tag-suggestions-no-tags = Attivati, ma questa cartella non ha ancora tag
batch-ai-tag-suggestions-on = { $tags ->
    [one] Dall'unico tag della cartella
   *[other] Dai { $tags } tag della cartella
}
batch-ai-key-missing = Impostare una chiave API di { $provider } nelle Impostazioni
batch-ai-key-unavailable = Impossibile aprire il portachiavi di sistema: potrebbe essere bloccato o non esistere (come GNOME Keyring o KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Uguale ai sottotitoli
ai-language-english = Inglese
ai-language-russian = Russo
ai-language-ukrainian = Ucraino
ai-language-german = Tedesco
ai-language-spanish = Spagnolo
ai-language-french = Francese
batch-videos-count = { $n ->
    [one] { $n } video
   *[other] { $n } video
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = meno di un minuto
batch-ai-duration-hours = { $h } h
batch-ai-duration-hours-minutes = { $h } h { $m } min
batch-ai-dollars-under = meno di 0,01 $
batch-ai-skip-described = già descritti
batch-ai-skip-too-long = oltre 30 min
batch-ai-skip-unreadable = illeggibili
batch-ai-skipped = Saltati: { $parts }.
batch-ai-progress-frame = fotogramma { $done } di { $total }
batch-ai-progress-waiting = in attesa di { $provider }
batch-ai-progress-saving = salvataggio
batch-ai-progress-retry-rate-limit = nuovo tentativo tra { $seconds } s (limite di richieste)
batch-ai-progress-retry-temporary = nuovo tentativo tra { $seconds } s (errore temporaneo)
batch-ai-stop-no-key = Interrotto: nessuna chiave API di { $provider }. Impostarne una nelle Impostazioni.
batch-ai-fail-no-key = Nessuna chiave API
batch-ai-fail-unreadable = Impossibile leggere il video
batch-ai-fail-too-long = La clip è troppo lunga per l'IA (oltre 30 min)
batch-ai-fail-not-saved = Impossibile salvare la descrizione
batch-ai-stop-offline = Interrotto: nessuna connessione con { $provider }. Eseguirla di nuovo per descrivere il resto.

batch-action-generate-subtitles = Generare sottotitoli
batch-action-generate-subtitles-install-ffmpeg = per leggere .mkv, .m2ts, .avi …, installare ffmpeg da ffmpeg.org, aggiungerlo al PATH e riavviare frename
batch-action-generate-subtitles-replace = Sostituire i sottotitoli esistenti
batch-action-generate-subtitles-replace-hint = Trascrive di nuovo; costa quanto indicato.
batch-action-generate-subtitles-srt = Sottotitoli SRT (clip.srt)
batch-action-generate-subtitles-premiere = Trascrizione di Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = In Premiere Pro: pannello Testo → Trascrizione → Importa trascrizione statica.
batch-action-generate-subtitles-privacy = L'audio di questi video viene inviato a Soniox e poi eliminato lì.
batch-action-generate-subtitles-duration-hint = Richiede qualche minuto per ogni ora di audio. { batch-runs-in-background }. Chiudere frename interrompe l'esecuzione; i sottotitoli completati restano.
batch-action-generate-subtitles-hint = Trascrive il parlato di ogni video selezionato con Soniox e salva il risultato accanto a esso: sottotitoli (clip.srt), dove frename li mostra, e/o una trascrizione di Premiere Pro.
batch-subtitles-transcribe = Trascrivere
batch-subtitles-transcribe-count = Trascrivere { $videos }
batch-subtitles-build-free = Creare { $count } (gratis)
batch-subtitles-nothing = Niente da trascrivere
batch-subtitles-estimating = Stima in corso…
batch-subtitles-key-missing = Impostare una chiave API di Soniox nelle Impostazioni
batch-subtitles-key-rejected = Soniox ha rifiutato la chiave
batch-subtitles-cost-unknown = costo sconosciuto
batch-subtitles-typical-price = { $usd } (prezzo tipico)
batch-subtitles-unknown-length = + { $n } di durata sconosciuta
batch-subtitles-plan-line = { $videos } da trascrivere, { $duration } di audio{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } ha già i sottotitoli
   *[other] { $n } hanno già i sottotitoli
}
batch-subtitles-rebuilt-free = { $n ->
    [one] { $n } ricreato
   *[other] { $n } ricreati
} gratis da una trascrizione salvata
batch-subtitles-no-speech-before = { $n } senza parlato l'ultima volta
batch-subtitles-no-audio = { $n ->
    [one] { $n } non ha audio
   *[other] { $n } non hanno audio
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } condivide il nome dei sottotitoli con un altro video
   *[other] { $n } condividono il nome dei sottotitoli con un altro video
}
batch-subtitles-unreadable = { $n ->
    [one] { $n } non è leggibile
   *[other] { $n } non sono leggibili
} ({ $how_to })
batch-subtitles-shared-name-reason = condivide il nome dei sottotitoli con un altro video
batch-subtitles-unsupported-reason = formato audio non supportato
batch-subtitles-progress-transcribing = trascrizione…
batch-subtitles-progress-extracting = estrazione dell'audio { $percent }%
batch-subtitles-progress-extracting-unknown = estrazione dell'audio
batch-subtitles-progress-uploading = caricamento { $percent }%
batch-subtitles-progress-uploading-unknown = caricamento
batch-subtitles-progress-transcribing-left = trascrizione… circa { $time } rimanenti
batch-subtitles-progress-saving = salvataggio
batch-subtitles-reason-already = aveva già i sottotitoli
batch-subtitles-reason-no-audio = nessun audio
batch-subtitles-reason-no-speech = nessun parlato
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = impossibile raggiungere Soniox
batch-subtitles-stop-key-rejected = Interrotto: Soniox ha rifiutato la chiave. Verificarla in Impostazioni → Sottotitoli.
batch-subtitles-stop-balance-empty = Interrotto: il credito Soniox è esaurito. Ricaricarlo su console.soniox.com.
batch-subtitles-stop-budget-used = Interrotto: il budget mensile di Soniox è esaurito. Aumentarlo su console.soniox.com.
batch-subtitles-stop-other = Interrotto: Soniox si è rifiutato di continuare ({ $error_type }).
batch-subtitles-stop-unreachable = Interrotto: impossibile raggiungere Soniox. Verificare la connessione a Internet.
batch-subtitles-report-spend = Soniox: almeno { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } file caricato non si è potuto eliminare
   *[other] { $n } file caricati non si sono potuti eliminare
} su Soniox (vedere il registro)
batch-subtitles-languages-need-key = Salvare una chiave di Soniox per scegliere tra tutte le sue lingue.
batch-subtitles-languages-rejected = Soniox ha rifiutato la chiave.
batch-subtitles-languages-failed = Impossibile ottenere le lingue da Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } sottotitolo
   *[other] { $n } sottotitoli
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = meno di 0,01 $
batch-subtitles-usd-about = circa { $amount } $

## File list
folder-has-subtitles = Ha i sottotitoli
folder-closed-by-job = Chiusa mentre un'azione batch la modifica
folder-filter-tip = Mostrare solo…

folder-search-placeholder = Cercare un file o un commento
folder-search-comments-loading = Ricerca nei commenti… ne mancano { $n }
folder-search-clear = Cancellare
folder-all = Tutti
folder-invert = Invertire
folder-checked = { $count ->
    [one] { $count } selezionato
   *[other] { $count } selezionati
}
folder-outcome-changed = Modificato
folder-outcome-unchanged = Niente da cambiare
folder-outcome-failed = Errore, vedere il registro
folder-rename-error-empty = Il nome è vuoto
folder-rename-error-bad-character = Non consentito: \ / : * ? " < > |
folder-rename-error-trailing = Non può terminare con un punto o uno spazio
folder-rename-error-exists = Esiste già un file con questo nome
folder-markers-not-saved = Marcatori non salvati: il file è di sola lettura o in uso (chiuderlo in Premiere, poi aprire il file e uscirne di nuovo)
drag-out-not-saved = Non trascinato: impossibile salvare il file (sola lettura, o aperto in Premiere)
folder-opening = Apertura della cartella…
folder-empty-title = Nessun video in questa cartella
folder-empty-line = frename mostra file MP4, MOV, MKV e altri file video.
folder-open-another = Aprire un'altra cartella…
folder-no-match = Nessun file corrisponde
folder-show-all = Mostrare tutto
folder-locked = Le caselle sono bloccate mentre è in corso { $action }
folder-checked-hidden = { $count ->
    [one] { $count } selezionato
   *[other] { $count } selezionati
} · { $hidden ->
    [one] { $hidden } nascosto
   *[other] { $hidden } nascosti
}
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } file selezionato è nascosto dalla ricerca o da un filtro; l'azione si esegue anche su di esso
   *[other] { $hidden } file selezionati sono nascosti dalla ricerca o da un filtro; l'azione si esegue anche su di essi
}
folder-outcome-working = In corso
folder-outcome-not-reached = Non raggiunto
folder-window-empty-title = Aprire una cartella di clip
folder-window-empty-line = Oppure trascinare una cartella sulla finestra. Un clic destro sul pulsante apre un singolo file.
folder-window-open = Aprire una cartella…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Cartelle recenti
recent-folders-title = Cartelle recenti
recent-folders-none = Nessuna cartella aperta finora
recent-folders-not-found = Non trovata
recent-folders-remove-tip = Rimuovere dall'elenco
recent-folders-clear = Svuotare l'elenco
recent-folders-remove = Rimuovere
recent-folders-keep = Mantenere
recent-folders-missing-line = Non trovata. Rimuoverla?
recent-folders-just-now = Adesso
recent-folders-minutes = { $count } min fa
recent-folders-hours = { $count } h fa
recent-folders-days = { $count ->
    [one] { $count } giorno fa
   *[other] { $count } giorni fa
}
recent-folders-weeks = { $count ->
    [one] { $count } settimana fa
   *[other] { $count } settimane fa
}
recent-folders-months = { $count ->
    [one] { $count } mese fa
   *[other] { $count } mesi fa
}
recent-folders-years = { $count ->
    [one] { $count } anno fa
   *[other] { $count } anni fa
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Mostrare in Esplora file
file-menu-show-in-file-manager = Mostrare nel file manager
file-menu-copy-path = Copiare il percorso completo
file-menu-copy-name = Copiare il nome del file
file-menu-copied = Copiato
file-menu-not-copied = Non copiato: gli appunti non sono disponibili
file-menu-not-shown = Impossibile aprire il file manager

## Controls bar under the file list

folder-controls-filter-untagged = Senza tag
folder-controls-filter-subtitles = Sottotitoli
folder-controls-filter-comments = Commenti
folder-controls-filter-markers = Marcatori
folder-controls-scroll = Mostrare il file aperto nell'elenco
folder-controls-open = Aprire una cartella
folder-controls-batch = Azioni batch sui file selezionati
folder-controls-batch-back = Tornare al file aperto
folder-controls-update-available = Aggiornamento disponibile: { $version }
folder-controls-whats-new = Novità della versione { $version }
folder-controls-previous = File precedente
folder-controls-next = File successivo
folder-controls-more = Altro
folder-controls-open-file = Clic destro: aprire un singolo file

## Video

video-controls-back = Indietro di 10 s
video-controls-play = Riprodurre
video-controls-pause = Pausa
video-controls-forward = Avanti di 10 s
video-controls-frame-back = Un fotogramma indietro
video-controls-frame-forward = Un fotogramma avanti
video-controls-set-in = Impostare il punto di attacco
video-controls-set-out = Impostare il punto di stacco
video-controls-screenshot = Salvare questo fotogramma
video-controls-add-marker = Aggiungere un marcatore
video-controls-add-marker-hold = Tenere premuto per un intervallo; premere di nuovo per dargli un nome
video-controls-cannot-hold-markers = Questo file non può contenere marcatori
video-controls-add-a-name = Aggiungere un nome
video-controls-rotate-left = Ruotare a sinistra
video-controls-rotate-right = Ruotare a destra
video-controls-more = Altro
media-viewer-loading-slow = In attesa del file… (un file nel cloud può richiedere un po' di tempo)
media-viewer-cannot-play = Impossibile riprodurre questa clip
media-viewer-no-picture = Questo file non ha immagine video
video-timecode-copied = Timecode copiato: { $time }
video-timecode-unreadable = «{ $text }» non è un tempo di questa clip
video-controls-volume-scroll = Volume — scorrere per cambiarlo
rotate-cannot = Impossibile ruotare: { $reason }
rotate-reason-missing = il file non c'è più
rotate-flag-right = 90° a destra
rotate-flag-left = 90° a sinistra
rotate-flag-half = 180°
rotate-flag-none = nessuna
rotate-now = Rotazione: { $flag }
rotate-turned = Ruotato di { $turn } · rotazione attuale: { $flag }
rotate-failed = Non ruotato: { $reason }
undo-name-taken = Nessuna modifica: un altro file si chiama già { $name }
undo-back-on-clip = Di nuovo su questa clip (spostamento annullato). Ctrl+Z di nuovo annulla la sua ultima modifica.
redo-on-clip = Di nuovo su questa clip (spostamento ripristinato).
rotate-reason-in-use = il file è di sola lettura o in uso
rotate-reason-format = questo formato non ha un indicatore di rotazione
rotate-reason-damaged = il file è danneggiato
rotate-reason-no-video = il file non ha una traccia video
rotate-reason-matrix = il video ha una matrice di visualizzazione insolita
batch-action-rotate = Ruotare i video
batch-action-rotate-hint = Cambia l'indicatore di rotazione di ogni file MP4 e MOV selezionato; l'immagine non viene ricodificata, e commenti, punti di attacco/stacco e marcatori restano. Premiere Pro dovrebbe mostrare la clip ruotata quando la importa. Gli altri formati non hanno un indicatore di rotazione e danno errore. Eseguirla di nuovo ruota di nuovo i file; «Reimpostare» elimina qualsiasi rotazione, anche quella registrata da un telefono, quindi la clip verticale di un telefono viene poi riprodotta di lato.
batch-action-rotate-right = 90° a destra (senso orario)
batch-action-rotate-left = 90° a sinistra (senso antiorario)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Reimpostare: nessuna rotazione (0°)
media-viewer-video-subtitle-list = Elenco dei sottotitoli
media-viewer-video-marker-list = Elenco dei marcatori
media-viewer-video-markers-hint = Maiusc+F1 / Maiusc+F3 saltano tra i marcatori; Maiusc+trascinamento aggancia
media-viewer-video-tab-markers = Marcatori
media-viewer-video-fullscreen = Schermo intero
media-viewer-video-close-list = Chiudere l'elenco
media-viewer-video-subtitle-file-unreadable = Impossibile leggere questo file
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Ripreso da { $time } · Home: ricominciare dall'inizio

## Markers list
markers-in-out = Punti di attacco/stacco

markers-empty = Ancora nessun marcatore
markers-add = Aggiungere un marcatore
markers-ai-hint = Marcatore IA: sostituito quando l'IA descrive di nuovo questa clip
markers-keep-color = Mantenere il colore
markers-done = Fatto
markers-delete = Eliminare il marcatore
markers-cannot-hold-hint = Premiere legge i marcatori dai file MP4 e MOV.
markers-color-green = Verde
markers-color-red = Rosso
markers-color-orange = Arancione
markers-color-yellow = Giallo
markers-color-white = Bianco
markers-color-blue = Blu
markers-color-cyan = Ciano
markers-color-lavender = Lavanda
markers-color-magenta = Magenta
markers-color-other = Altro colore
markers-read-only = sola lettura
markers-name-placeholder = Nome
markers-ai-describe = Descrivere con IA: dare un nome al marcatore e aggiungere cosa succede
markers-ai-stop = Smettere di descrivere
markers-ai-describe-unnamed = Descrivere { $count } senza nome
markers-ai-describe-unnamed-hint = Dare un nome e descrivere con IA ogni marcatore senza nome, pochi alla volta. Una richiesta per marcatore; un nome digitato nel frattempo viene mantenuto.
markers-ai-price-about = circa { $dollars }
markers-ai-describe-unnamed-cost = Una richiesta per marcatore: { $each } ciascuno, { $total } in tutto.
markers-ai-confirm = { $count ->
    [one] Descrivere { $count } marcatore? Una richiesta: { $total }.
   *[other] Descrivere { $count } marcatori? Una richiesta per marcatore: { $total } in tutto.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Descrivere { $count } marcatore? Una richiesta a pagamento.
   *[other] Descrivere { $count } marcatori? Una richiesta a pagamento per marcatore.
}
markers-ai-confirm-yes = Descrivere
markers-ai-confirm-no = Annullare
markers-ai-waiting = In attesa del proprio turno…
markers-ai-run = Descrizione in corso: { $describing } · { $waiting } in attesa
markers-ai-run-last = Descrizione in corso: { $describing }
markers-ai-stop-all = Fermare tutto
markers-ai-describing = Descrizione in corso…
markers-ai-done = Marcatore descritto
markers-ai-nothing-new = Il marcatore ha già questa descrizione
markers-no-marker-notice = Nessun marcatore qui
markers-ai-no-key = Nessuna chiave API di { $provider }: impostarne una nelle Impostazioni
markers-ai-failed = Non descritto: { $reason }
markers-ai-failed-unknown = qualcosa è andato storto
markers-read-only-notice = Quel marcatore è di sola lettura
markers-ai-gone = Il marcatore non c'è più: la sua descrizione non è stata aggiunta
markers-ai-stopping-for-batch = Arresto delle descrizioni dei marcatori in corso… Di solito bastano pochi secondi.
folder-not-saved-marker-ai = Non salvato: la descrizione del marcatore stava ancora leggendo la clip. Riprovare tra un momento.

## File workspace

file-workspace-search-placeholder = Cercare un tag — o semplicemente scrivere
file-workspace-search-clear = Cancellare
file-workspace-comment-placeholder = Commento...
file-workspace-comment-collapse = Tornare ai tag
file-workspace-comment-expand = Espandere il commento

## Tag grid

tag-grid-star = Aggiungere una stella: tenerlo in alto
tag-grid-unstar = Togliere la stella
tag-grid-save = Aggiungere ai tag della cartella
tag-grid-delete = Eliminare «{ $tag }» dai tag della cartella
tag-grid-create = Creare «{ $tag }»
tag-grid-no-file = Aprire una clip per aggiungerle dei tag
tag-grid-no-tags = Ancora nessun tag
tag-grid-no-tags-hint = Digitare un nome e premere Invio per creare il primo.
tag-grid-group-unsaved = Non tra i tag della cartella
tag-grid-group-folder = Tag della cartella
tag-grid-more = +{ $count } altri

## Order strip

sync-panel-locked = Riordinare qui sotto riordina la cartella
sync-panel-unlocked = Riordinare qui sotto cambia solo questa clip
sync-panel-unlock = Sbloccare
sync-panel-lock = Bloccare
sync-panel-differs = L'ordine è diverso da quello della cartella
sync-panel-use-for-folder = Usare per la cartella
sync-panel-sort-like-folder = Ordinare come la cartella

## File name card

file-name-panel-no-tags = Nessun tag su questa clip
file-name-panel-untag = Togliere il tag
file-name-panel-clear-in = Cancellare il punto di attacco
file-name-panel-clear-out = Cancellare il punto di stacco
file-name-panel-suggested-in-out = IA
file-name-panel-apply-suggested-in-out = Impostare attacco e stacco come suggerisce l'IA (arrotondati ai secondi interi)
file-name-panel-suggested-tags = Tag IA
file-name-panel-add-suggested-tag = Aggiungere il tag «{ $tag }» (certezza: { $percent }%)
file-name-panel-add-suggested-tag-unsure = Aggiungere il tag «{ $tag }»
file-name-panel-add-all-suggested-tags = Aggiungere tutti
file-name-panel-add-all-suggested-tags-tip = Aggiungere tutti i tag suggeriti dall'IA
file-name-panel-tag-ideas = Idee: { $ideas }
file-name-panel-tag-ideas-tip = Tag notati dall'IA che questa cartella non ha. Non vengono aggiunti.
file-name-panel-saved = Tutte le modifiche salvate
file-name-panel-saved-to-recovery = Salvato nel ripristino, applicato quando si lascia la clip

## Updates (Settings)

updates-check = Cercare aggiornamenti
updates-not-installed = Gli aggiornamenti funzionano nella versione installata
updates-from-store = Gli aggiornamenti arrivano da Microsoft Store
updates-from-releases = Nuove versioni: scaricarle dalla pagina delle release
updates-open-releases = Aprire la pagina delle release
updates-checking = Verifica in corso…
updates-downloading-named = Download di { $version }… { $percent }%
updates-downloading = Download… { $percent }%
updates-restarting = Riavvio…
updates-check-failed = Impossibile cercare aggiornamenti: { $reason }
updates-update-failed = Impossibile aggiornare: { $reason }
updates-close-other-window = Chiudere prima l'altra finestra di frename
updates-reason-github = GitHub non ha risposto, riprovare più tardi
updates-reason-no-connection = nessuna connessione
updates-reason-read-only = la cartella è di sola lettura
updates-reason-not-installed = gli aggiornamenti funzionano nella versione installata
updates-reason-download-stopped = il download si è interrotto inaspettatamente
updates-version-available = È disponibile la versione { $version }
updates-up-to-date = frename è aggiornato
updates-update-and-restart = Aggiornare e riavviare
updates-wait-for-batch = Attendere la fine dell'azione batch
updates-current-version = frename { $version }
updates-check-on-start = Cercare aggiornamenti all'avvio di frename
updates-notes-show = Note di rilascio
updates-notes-hide = Nascondi le note di rilascio
updates-notes-all-releases = Tutte le versioni su GitHub
updates-notes-history = Tutte le versioni, dalla più recente
updates-notes-update = Cosa porta la versione { $version }
updates-notes-whats-new = Novità della versione { $version }
updates-notes-loading = Caricamento delle note di rilascio…
updates-notes-unavailable = Non è stato possibile caricare le note dell’ultima versione.
updates-notes-more = Mostra altro

recovery-restored = frename si è chiuso inaspettatamente. Il lavoro non salvato su { $clip } ({ $what }) è stato ripristinato.
recovery-kept = frename si è chiuso inaspettatamente. Il lavoro non salvato su { $clip } ({ $what }) non è stato applicato: { $why }. Una copia leggibile da cui riscriverlo si trova in { $folder }.
recovery-unreadable = frename si è chiuso inaspettatamente e non è stato possibile leggere un file di ripristino. È conservato in { $folder }.
recovery-why-gone = la clip non c'è più o è stata rinominata
recovery-why-changed = la clip è cambiata dopo le modifiche
recovery-why-not-written = esiste già un file con quel nome, oppure la clip è di sola lettura o in uso
recovery-tags = { $count ->
    [one] { $count } tag
   *[other] { $count } tag
}
recovery-markers = { $count ->
    [one] { $count } marcatore
   *[other] { $count } marcatori
}
recovery-comment = commento
recovery-in-out = punti di attacco/stacco
recovery-edits = modifiche
folder-frame-saved = Fotogramma salvato
folder-not-saved = Non salvato: esiste già un file con quel nome, oppure è di sola lettura o in uso
markers-deleted-notice = Marcatore eliminato
markers-not-saved-notice = Marcatori non salvati: il file è di sola lettura o in uso
markers-comment-not-saved-notice = Marcatori non salvati: il file del commento è di sola lettura o in uso
markers-copy = Copiare
markers-copy-tip = Copiare i marcatori come testo, una riga per marcatore
markers-export = Esportare…
markers-export-tip = Salvare i marcatori in un file di testo
markers-exported = Marcatori salvati in { $name }
markers-not-exported = Marcatori non salvati: impossibile scrivere il file

batch-option-notes = Note per l’IA
batch-ai-notes-info = Le tue note sul girato di questa cartella, inviate con ogni richiesta all’IA qui: descrizioni, suggerimenti di tag e descrizioni dei marcatori. Salvate nel file .frename della cartella.
batch-ai-notes-placeholder = es. L’uomo che compare più spesso è Eugenio. Segna i cartelloni pubblicitari. Suggerisci il tag tè dove vedi del tè.
batch-ai-notes-hint = Inviate a { $provider } insieme ai fotogrammi. I nomi vengono solo da queste note; frename non riconosce i volti. I video già descritti vengono saltati: spunta Rifai per descriverli di nuovo con le nuove note.
batch-ai-notes-too-long = Le note sono di { $chars } caratteri; il limite è { $max }. Accorciale per avviare.
batch-ai-notes-too-long-reason = Le note per l’IA sono troppo lunghe
batch-option-notes-once = Riga in più per questa esecuzione
batch-ai-notes-once-placeholder = es. In questa clip: la donna in rosso è Ana
file-workspace-ai-notes-on = Note attive
file-workspace-ai-notes-tip = Le note per l’IA accompagnano ogni richiesta all’IA in questa cartella. Modificale in Descrivi con l’IA (azioni in blocco): { $notes }
batch-ai-notes-no-subtitles = Le tue note sembrano riguardare ciò che viene detto, e { $videos } non hanno sottotitoli: generali prima (Genera sottotitoli), così l’IA può leggere le parole.
