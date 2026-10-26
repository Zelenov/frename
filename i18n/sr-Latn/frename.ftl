# Tekst interfejsa frename na srpskom (latinica). Isti identifikatori, atributi i $argumenti kao u
# i18n/en/frename.ftl, istim redosledom (proveravaju ih testovi u src/i18n.rs).
# Ovo je ćirilična datoteka (i18n/sr-Cyrl/frename.ftl) slovo po slovo prebačena u latinicu: tekst se
# ispravlja tamo, a ova datoteka se pravi ponovo (test u src/i18n.rs proverava da se slažu).
#
# Glossary (docs/design/localization.md): every string uses these terms.
#   tag / tags                        teg / tegovi (never «oznaka»: that is Premiere's Label)
#   file                              datoteka
#   folder                            fascikla
#   video / clip                      snimak / klip
#   in / out points                   ulazna / izlazna tačka
#   comment                           komentar
#   subtitles                         titlovi
#   marker (Premiere)                 marker
#   range (a marker with a duration)  opseg
#   checked (files)                   označeno
#   All / Invert (check boxes)        Sve / Obrni
#   Cancel / Stopping…                Otkaži / Zaustavljanje…
#   log, "see the log"                dnevnik, «detalje pogledajte u dnevniku»
#   System (language list)            Kao u sistemu (…)
#   batch action                      grupna radnja
#   settings                          podešavanja
#   update (a newer frename)          ažuriranje
#   AI description (Describe with AI) AI opis
#   a tag name inside a sentence      in «…» quotes: «Commented»

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

settings-window-title = Podešavanja
settings-page-interface = Interfejs
settings-page-saving = Čuvanje
settings-page-ai = Opis pomoću AI
settings-close = Zatvori
settings-apply-note = Izmene se primenjuju odmah. Tab prebacuje između opcija, Ctrl+Tab između stranica.
settings-language = Jezik
settings-language-system = Kao u sistemu ({ $language })
settings-video = Video
settings-video-autoplay = Automatski pusti snimke kada se otvore
settings-subtitle-size = Veličina titlova
settings-subtitle-size-small = Mala
settings-subtitle-size-medium = Srednja
settings-subtitle-size-large = Velika
settings-subtitle-size-extra-large = Vrlo velika
settings-timecode = Prikaz vremena
settings-timecode-hint = Kako se zapisuje vreme ispod snimka, u spiskovima markera i titlova i na oznakama In / Out, kao i vreme položaja reprodukcije koje kopira Ctrl+Shift+C. Premiere Pro i DaVinci Resolve prikazuju časove:minute:sekunde:kadrove prema brzini kadrova klipa; 29,97 i 59,94 fps su drop-frame. Klip bez stabilne brzine kadrova zadržava podrazumevani stil. Datoteke zadržavaju svoje formate.
settings-timecode-default = Podrazumevano (mm:ss)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Boje tegova
settings-tags-monochrome = Jednobojno
settings-tags-monochrome-hint = Tegovi koje ste dodali prikazuju se tamnijom sivom, ostali svetlijom, bez drugih boja.
settings-file-names = Nazivi datoteka
settings-tags-space-after = Razmak posle svakog tega
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Datoteke zadržavaju nazive dok se ne preimenuju ili sačuvaju.
settings-tags-space-add = Dodaj razmak u nazive datoteka…
settings-tags-space-remove = Ukloni razmak iz naziva datoteka…
settings-comments = Komentari
settings-comments-in-video = Unutar video datoteke
settings-comments-in-video-hint = XMP, kolona Description u programu Premiere Pro
settings-comments-text-file = U tekstualnoj datoteci pored snimka
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Datoteke zadržavaju komentare gde jesu dok se ne premeste.
settings-comments-move-into-videos = Premesti komentare u snimke…
settings-comments-move-into-text-files = Premesti komentare u tekstualne datoteke…
settings-commented-tag = Teg za snimke sa komentarom
settings-commented-tag-name = Teg
settings-commented-tag-hint = Označava se kada napišete komentar uz snimak, npr. { $tag }.IMG_0424.MOV, a skida kada ga obrišete (AI opisi se ne računaju). Inače ga menjate sami.
settings-commented-tag-off = Snimci sa komentarom ne dobijaju teg.
settings-in-out = Ulazne i izlazne tačke
settings-in-out-in-video = Unutar video datoteke
settings-in-out-in-video-hint = Marker podklipa u programu Premiere Pro
settings-in-out-comment = U komentaru, u jednom redu
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Datoteke zadržavaju ulazne i izlazne tačke gde jesu dok se ne premeste.
settings-in-out-move-into-videos = Premesti ulazne i izlazne tačke u snimke…
settings-in-out-move-into-comments = Premesti ulazne i izlazne tačke u komentare…
settings-markers = Markeri i opsezi
settings-markers-in-video = Unutar video datoteke
settings-markers-in-video-hint = XMP, vidi se na klipu u programu Premiere Pro
settings-markers-comment = U komentaru, po jedan red
settings-markers-comment-example = 0:41–0:47 — Lav
settings-markers-note = Datoteke zadržavaju markere gde jesu dok se ne premeste.
settings-markers-move-into-videos = Premesti markere u snimke…
settings-markers-copy-into-comment = Kopiraj markere u komentare…
settings-markers-hint = I tačke i opsezi, kao i trenuci koje pronađe AI.
settings-updates = Ažuriranja
settings-version = Verzija
settings-release-notes = Beleške o izdanju
settings-old-title = Podešavanja iz starijeg frename-a
settings-old-hint = Prenesite podešavanja frename-a koji ste pokretali iz zip fascikle.
settings-old-scheduled = Podešavanja će biti uvezena kada se frename ponovo pokrene
settings-old-not-found = Nema frename.exe sa frename.db u { $folder }
settings-old-failed = Uvoz nije uspeo: { $reason }
settings-old-import-button = Uvezi iz stare frename fascikle…
settings-ai = Opis pomoću AI
settings-ai-key-label = Anthropic API ključ
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Ključ dobijate na console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Ukloniti sačuvani Anthropic ključ?
settings-ai-used-by = Koristi ga radnja „Opis pomoću AI“ u grupnom režimu i na markerima (✨, Ctrl+F2), kada je izabran Claude model.
settings-ai-openai-key-label = OpenAI API ključ
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Ključ dobijate na platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Ukloniti sačuvani OpenAI ključ?
settings-ai-openai-used-by = Koristi ga radnja „Opis pomoću AI“ u grupnom režimu i na markerima (✨, Ctrl+F2), kada je izabran GPT model.
settings-ai-gemini-key-label = Gemini API ključ
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Ključ dobijate na aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Ukloniti sačuvani Gemini ključ?
settings-ai-gemini-used-by = Koristi ga radnja „Opis pomoću AI“ u grupnom režimu i na markerima (✨, Ctrl+F2), kada je izabran Gemini model.
settings-ai-model-needs-key = { $model } — prvo dodajte ključ za { $provider }
settings-ai-model-label = Model
settings-ai-language-label = Jezik opisa
settings-ai-hint = Haiku, GPT-4.1 mini i Gemini Flash-Lite su najjeftiniji; Sonnet, Opus, GPT-4.1 i Gemini Flash uočavaju više. Model zahteva ključ svoje kompanije.
settings-ai-moments-label = Trenuci
settings-ai-moments-important = Samo ono što se ističe
settings-ai-moments-full = Obuhvati ceo klip
settings-ai-moments-important-hint = Predlaže i gde postaviti In i Out (AI oznaka pored IN i OUT)
settings-ai-moments-full-hint = Trenutak za svaki deo, i za tihe; bez predloga za In i Out
settings-ai-tag-suggestions = Predloži i tegove iz tegova fascikle
settings-ai-tag-suggestions-hint = U istom zahtevu AI kaže koji tegovi fascikle odgovaraju svakom klipu. Predloženi teg se dodaje tek kada kliknete na njega.
settings-subtitles = Titlovi
settings-subtitles-key-label = Soniox API ključ
settings-subtitles-key-placeholder = Nalepite ključ
settings-subtitles-key-get = Ključ dobijate na console.soniox.com. Zvuk se šalje usluzi Soniox.
settings-subtitles-key-remove-confirm = Ukloniti sačuvani Soniox ključ?
settings-subtitles-languages-label = Jezici
settings-subtitles-languages-locked = Sačuvajte ključ da biste birali među svim jezicima koje Soniox poznaje.
settings-subtitles-languages-loading = Preuzimanje jezika od usluge Soniox…
settings-subtitles-languages-none = Ništa nije označeno: jezik se prepoznaje automatski.
settings-subtitles-languages-hint = Jezici koji se govore u snimku, kao nagoveštaj.
settings-subtitles-cue-length-label = Dužina titla
settings-subtitles-cue-short = Kratka
settings-subtitles-cue-short-hint = Jedan red, do 8 s
settings-subtitles-cue-sentence = Jedna rečenica
settings-subtitles-hint = Koristi ga radnja „Napravi titlove“ u grupnom režimu. Dužina titla važi za nove titlove.
settings-key-store-windows = Windows Credential Manager
settings-key-store-macos = macOS Keychain
settings-key-store-other = sistemski privezak ključeva
settings-key-unavailable = Sistemski privezak ključeva nije mogao da se otvori
settings-key-unavailable-hint = Možda je zaključan ili ga nema (kao što su GNOME Keyring ili KWallet). Podešavanja proveravaju ponovo pri svakom otvaranju.
settings-key-remove-confirm-hint = Moraćete ponovo da ga nalepite.
settings-key-remove-ask = Ukloni…
settings-key-remove = Ukloni ključ
settings-key-keep = Zadrži
settings-key-replace = Zameni…
settings-key-hide = Sakrij
settings-key-show = Prikaži
settings-key-save = Sačuvaj ključ
settings-key-get = Nabavi ključ
settings-key-billing = Dodaj sredstva
settings-key-cancel = Otkaži
settings-key-saved-in = Sačuvano u: { $store }, na ovom računaru
settings-key-save-into = „Sačuvaj ključ“ ga čuva u: { $store }, na ovom računaru.

## Batch mode

batch-checked-count = { $count ->
    [one] Označena je { $count } datoteka
    [few] Označene su { $count } datoteke
   *[other] Označeno je { $count } datoteka
}
batch-no-key = Nema ključa
batch-group-move = Premeštanje između mesta
batch-group-fix = Ispravljanje naziva i snimaka
batch-group-paid = Plaćene usluge
batch-reason-none-checked = Nijedna datoteka nije označena
batch-reason-reading = Čitanje dužina klipova…
batch-check-all = Označi { $count ->
    [one] sve ({ $count } datoteka)
    [few] sve ({ $count } datoteke)
   *[other] sve ({ $count } datoteka)
}
batch-job-running = radi
batch-job-finished = završeno
batch-job-stopped = zaustavljeno
batch-progress-files = { $finished } od { $total ->
    [one] { $total } datoteke
    [few] { $total } datoteke
   *[other] { $total } datoteka
}
batch-time-left = preostalo još oko { $time }
batch-time-estimating = procena vremena…
batch-time-spent = do sada { $time }
batch-runs-in-background = Možete da nastavite da radite dok traje
batch-clip-in-work = U upotrebi: { $action }
batch-folder-held = Zauzeto: { $action }
batch-indicator-hint = Kliknite za panel i otkazivanje
batch-action-prepare = Pripremi klipove
batch-action-prepare-hint = Za svaki označeni klip, u jednom prolazu: titlovi (Soniox) ako ih nema, pa opis pomoću AI: rezime, njegovi trenuci kao markeri, komentar i predlozi tegova. Jedna cena, jedno ograničenje i jedan napredak za sve.
batch-prepare-step-off = isključeno
batch-prepare-nothing = Nema šta da se uradi za ove klipove
batch-prepare-report-cap = Potrošeno { $spent } od ograničenja { $cap }
batch-option-steps = Koraci
batch-prepare-step-subtitles = Prvo napravi titlove (Soniox)
batch-prepare-step-description = Opiši klipove pomoću AI
batch-prepare-cap = Zaustavi trošenje na
batch-prepare-cap-none = Bez ograničenja (američkih dolara)
batch-prepare-cap-invalid = Ograničenje je iznos u dolarima, na primer 5
batch-prepare-left = Poslednje pokretanje je stalo, preostalo je još: { $count ->
    [one] { $count } klip
    [few] { $count } klipa
   *[other] { $count } klipova
}
batch-prepare-check-left = Označi ih
batch-prepare-estimating = Procena šta će svaki korak uraditi…
batch-prepare-plan-subtitles = Titlovi
batch-prepare-plan-description = Opis
batch-prepare-plan-none = nema šta da se radi
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } snimak
    [few] { $videos } snimka
   *[other] { $videos } snimaka
} za transkripciju, { $duration } zvuka
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } snimak
    [few] { $videos } snimka
   *[other] { $videos } snimaka
} za opis
batch-prepare-resume-note = Svaki korak preskače ono što već postoji, pa se zaustavljeno pokretanje nastavlja ponovnim pokretanjem: ništa se ne plaća dva puta.
batch-prepare-run = Pripremi { $clips ->
    [one] { $clips } klip
    [few] { $clips } klipa
   *[other] { $clips } klipova
} · oko { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Slični snimci
batch-prepare-groups-hint = Klipovi koji izgledaju kao isti kadar: duplikati, ponovni izvozi, kamera koja se nije pomerala.
batch-prepare-group-clips = { $count ->
    [one] { $count } klip
    [few] { $count } klipa
   *[other] { $count } klipova
}
batch-prepare-check-group = Označi ove klipove
batch-solo-queued = U redu čekanja: { $action }
batch-suggest-in-job = Opis ovog klipa već je u poslu koji je u toku, bez predloga tegova. Pokušajte ponovo kada se završi
batch-suggest-no-tags = U ovoj fascikli još nema tegova za predloge: prvo dodajte neke
batch-solo-gave-up = Priprema nije uspela
batch-solo-withdrawn = Povučeno: { $action }
batch-solo-in-work = { $action } je u toku. Otkažite ga sa trake ili sa stranice posla
batch-waiting-title = { $count ->
    [one] { $count } pokretanje čeka
    [few] { $count } pokretanja čekaju
   *[other] { $count } pokretanja čekaju
}
batch-waiting-remove = Ukloni
batch-indicator-waiting = čeka: { $count }
file-workspace-describe-clip = Opiši ovaj klip pomoću AI
file-workspace-subtitles-clip = Napravi titlove za ovaj klip
file-workspace-describe-clip-queued = Opis pomoću AI: čeka na red. Kliknite da povučete
file-workspace-describe-clip-running = Opis pomoću AI: u toku
file-workspace-suggest-tags-clip = Predloži tegove za ovaj klip pomoću AI (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Predlog tegova: čeka na red. Kliknite da povučete
file-workspace-suggest-tags-clip-running = Predlog tegova: u toku
file-workspace-subtitles-clip-queued = Pravljenje titlova: čeka na red. Kliknite da povučete
file-workspace-subtitles-clip-running = Pravljenje titlova: u toku
batch-run-again = Pokreni ponovo za { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-result-done = Gotovo: { $total ->
    [one] { $total } datoteka
    [few] { $total } datoteke
   *[other] { $total } datoteka
}
batch-result-done-detail = Izmenjeno: { $changed }, bez izmena: { $unchanged }
batch-result-problems = Gotovo uz probleme: nije urađeno za { $failed } od { $total ->
    [one] { $total } datoteke
    [few] { $total } datoteke
   *[other] { $total } datoteka
}
batch-figure-unchanged = bez izmena
batch-figure-not-done = nije urađeno
batch-figure-not-reached = nije obrađeno
batch-files-not-done = Datoteke za koje nije urađeno
batch-table-file = Datoteka
batch-table-why = Zašto

batch-run-move-comments = Premesti { $count ->
    [one] { $count } komentar
    [few] { $count } komentara
   *[other] { $count } komentara
}
batch-run-move-in-out = Premesti ulazne/izlazne tačke za { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-run-convert = Pretvori { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-run-rotate = Rotiraj { $count ->
    [one] { $count } snimak
    [few] { $count } snimka
   *[other] { $count } snimaka
}
batch-run-tag = Označi tegom { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-run-fix-tags = Promeni redosled tegova u { $count ->
    [one] { $count } nazivu
    [few] { $count } naziva
   *[other] { $count } naziva
}
batch-run-rename = Preimenuj { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-run-reload = Ponovo učitaj { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-run-export-markers = Izvezi markere za { $count ->
    [one] { $count } datoteku
    [few] { $count } datoteke
   *[other] { $count } datoteka
}
batch-service-soniox = Soniox
batch-change-renames = Preimenuje datoteke
batch-change-videos = Upisuje u snimke
batch-change-text-files = Upisuje tekstualne datoteke pored snimaka
batch-change-comments = Upisuje u komentare
batch-change-subtitles = Upisuje datoteke titlova pored snimaka
batch-change-records = Menja samo sopstvene zapise programa frename
batch-option-direction = Smer
batch-option-language = Jezik
batch-option-tag-suggestions = Predlozi tegova
batch-option-described = Opisani snimci
batch-option-subtitled = Snimci sa titlovima
batch-option-subtitles-write = Datoteke za upis
batch-option-spacing = Razmak
batch-option-tag = Teg
batch-option-turn = Okretanje
batch-plan-videos = Snimci
batch-plan-length = Dužina
batch-plan-cost = Cena
batch-plan-time = Vreme
batch-reason-estimate = Čeka se procena
batch-reason-subtitles-no-format = Označite bar jednu datoteku za upis
batch-set-key = Podesite ključ…
batch-check-key = Proverite ključ…
batch-subtitles-languages-auto = Prepoznaje se u svakom snimku
batch-action-describe-ai-run-waiting = Opiši snimke
batch-action-fix-tags-order = Redosled: kao na spisku tegova, nepoznati tegovi prvi.
batch-action-markers-comment-hint-short = Pretvara redove komentara sa vremenom u markere ili kopira markere u komentar.
batch-action-markers-to-comment-hint = Markeri ostaju; ponovno pokretanje ništa ne dodaje dva puta.
batch-action-rotate-hint-short = Okreće MP4 i MOV klipove prema njihovoj oznaci rotacije, bez ponovnog kodiranja.
batch-action-tag-commented-off = Označavanje snimaka sa komentarom je isključeno
batch-action-tag-commented-choose = Izaberite teg…

batch-title = Grupne radnje
batch-done-label-changed = izmenjeno
batch-done-label-subtitled = sa titlovima
batch-ai-at-least = najmanje
batch-cancel = Otkaži
batch-stopping = Zaustavljanje…
batch-stop-over-budget = Zaustavljeno: sledeći klip bi mogao da premaši ograničenje od { $cap } (potrošeno { $spent }). Povećajte ograničenje da bi se uradilo ostalo.
batch-stopped = Zaustavljeno nakon { $finished } od { $total ->
    [one] { $total } datoteke
    [few] { $total } datoteke
   *[other] { $total } datoteka
}.
batch-close = Zatvori
batch-failed-subtitles = Bez titlova:
batch-written-subtitles = Zapisane datoteke:
batch-table-written = Zapisano
batch-merged-markers = Spojeni duplikati markera:
batch-table-merged = Spojeno
batch-markers-merged = { $count ->
    [one] Spojen je { $count } duplikat markera
    [few] Spojena su { $count } duplikata markera
   *[other] Spojeno je { $count } duplikata markera
}
batch-add-credit = Dodaj sredstva
batch-open-log = Otvori dnevnik

batch-action-move-comments = Premesti komentare
batch-action-move-comments-hint = Premešta komentar svake označene datoteke na izabrano mesto, zajedno sa ulaznim i izlaznim tačkama koje sadrži. Tegovi i ulazne i izlazne tačke sačuvane u snimku ostaju gde jesu.
batch-action-move-comments-into-videos = Iz tekstualnih datoteka u snimke (XMP)
batch-action-move-comments-into-text-files = Iz snimaka (XMP) u tekstualne datoteke

batch-action-move-in-out = Ulazne/izlazne tačke: komentar ⇄ snimak (XMP)
batch-action-move-in-out-hint = Premešta ulazne i izlazne tačke svake označene datoteke na izabrano mesto. Ostatak komentara ostaje gde jeste.
batch-action-move-in-out-into-videos = Iz komentara u snimke (Adobe XMP marker)
batch-action-move-in-out-into-comments = Iz snimaka (XMP marker) u komentare

batch-action-markers-comment = Markeri ⇄ komentar
batch-action-markers-comment-to-markers = Redovi komentara sa vremenom u markere
batch-action-markers-to-comment = Markeri u komentar (kopija: markeri ostaju)
batch-action-markers-comment-hint = Red kao „03:24 — Take 3 — nice light“ je marker na 3:24 pod nazivom „Take 3“ sa komentarom „nice light“; „0:41-0:47 — Lion“ je marker od 0:41 do 0:47. Naziv i komentar se dele na prvom „ — “ ili „ -- “, a ne na običnom „ - “. Trenuci AI opisa („0:00–0:14 Street.“) postaju beli markeri i ostaju u opisu. Ponovno pokretanje u bilo kom smeru ništa ne dodaje dva puta.

batch-action-tag-commented = Označi tegom snimke sa komentarom
batch-action-tag-commented-hint = Dodaje teg „{ $tag }“ svakom označenom snimku sa vašim komentarom (AI opisi se ne računaju) i uklanja ga sa onih bez komentara. Datoteke čiji se teg promeni se preimenuju.
batch-action-tag-commented-hint-off = Dodaje teg za snimke sa komentarom svakom označenom snimku sa vašim komentarom i uklanja ga sa onih bez komentara. Teg je isključen u podešavanjima.
batch-action-fix-tags = Ispravi tegove po prioritetu
batch-action-fix-tags-hint = Poređa tegove u nazivu svake označene datoteke redom iz panela tegova, pa što je teg viši u njemu, to je ranije u nazivu. Tegovi koje fascikla još ne poznaje idu prvi, kao u panelu tegova. Datoteke čiji se redosled promeni se preimenuju.

batch-action-respace-tags = Primeni razmak između tegova
batch-action-respace-tags-hint-space = Preimenuje svaku označenu datoteku tako da iza svakog tega bude razmak, kako je podešeno u podešavanjima: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Preimenuje svaku označenu datoteku tako da iza tegova nema razmaka, kako je podešeno u podešavanjima: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Razmak: razmak posle svakog tega
batch-action-respace-tags-status-no-space = Razmak: bez razmaka posle tegova
batch-action-export-markers = Izvezi markere u tekst
batch-action-export-markers-hint = Upisuje markere svake označene datoteke u tekstualnu datoteku pored nje, clip.markers.txt: jedan red po markeru, po redu vremena, kao „0:41–0:47 [red] — Lion crossing — walks left to right“. Boja različita od zelene je u uglastim zagradama; komentar sledi iza naziva. „Markeri ⇄ komentar“ vraća takve redove u markere. Datoteka bez markera ne dobija tekstualnu datoteku, a tekstualna datoteka iz ranijeg pokretanja se zamenjuje.
batch-export-markers-cannot-hold = Ova datoteka ne može da sadrži markere
batch-action-reload-files = Resetuj keš i ponovo učitaj
batch-action-reload-files-hint = Ponovo čita komentar i ulazne i izlazne tačke svake označene datoteke iz same datoteke i zamenjuje ono što je fascikla zapamtila za nju. Koristite to nakon što je datoteke promenio drugi program. Datoteke čije su zapamćene vrednosti nedostajale ili bile zastarele računaju se kao izmenjene.

batch-action-describe-ai = Opis pomoću AI
batch-action-describe-ai-run = Opiši { $videos } · oko { $dollars }
batch-action-describe-ai-estimating = Procena… { $known } / { $total }
batch-action-describe-ai-none = Nema snimaka za opis.
batch-action-describe-ai-hint = { batch-runs-in-background }. Otkaži zadržava već opisane snimke; ponovno pokretanje ih preskače.
batch-action-describe-ai-no-subtitles = Bez titlova (opisuje se samo slika): { $videos }.
batch-action-describe-ai-redo = Ponovo obradi snimke koji već imaju AI opis
batch-action-describe-ai-hint-panel = Opisuje šta se dešava u svakom označenom snimku i kada: rezime i segmenti sa vremenskim opsezima idu u AI opis njegovog komentara; vaš tekst se zadržava. Kadrovi i titlovi se šalju kompaniji { $provider }, proizvođaču izabranog modela.
batch-ai-change = Promeni
batch-ai-tag-suggestions-off = Isključeno
batch-ai-tag-suggestions-no-tags = Uključeno, ali ova fascikla još nema tegova
batch-ai-tag-suggestions-on = { $tags ->
    [one] Iz { $tags } tega fascikle
    [few] Iz { $tags } tega fascikle
   *[other] Iz { $tags } tegova fascikle
}
batch-ai-key-missing = Podesite API ključ za { $provider } u podešavanjima
batch-ai-key-unavailable = Sistemski privezak ključeva nije mogao da se otvori: možda je zaključan ili ga nema (kao što su GNOME Keyring ili KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Kao titlovi
ai-language-english = Engleski
ai-language-russian = Ruski
ai-language-ukrainian = Ukrajinski
ai-language-german = Nemački
ai-language-spanish = Španski
ai-language-french = Francuski
batch-videos-count = { $n ->
    [one] { $n } snimak
    [few] { $n } snimka
   *[other] { $n } snimaka
}
batch-minutes = { $n } min
batch-ai-minutes-under = < 1 min
batch-ai-duration-under-minute = manje od minuta
batch-ai-duration-hours = { $h } č
batch-ai-duration-hours-minutes = { $h } č { $m } min
batch-ai-dollars-under = manje od $0.01
batch-ai-skip-described = već opisano
batch-ai-skip-too-long = preko 30 min
batch-ai-skip-unreadable = nečitljivo
batch-ai-skipped = Preskočeno: { $parts }.
batch-ai-progress-frame = kadar { $done } od { $total }
batch-ai-progress-waiting = čeka se { $provider }
batch-ai-progress-saving = čuvanje
batch-ai-progress-retry-rate-limit = novi pokušaj za { $seconds } s (ograničenje učestalosti)
batch-ai-progress-retry-temporary = novi pokušaj za { $seconds } s (privremena greška)
batch-ai-stop-no-key = Zaustavljeno: nema API ključa za { $provider }. Podesite ga u podešavanjima.
batch-ai-fail-no-key = Nema API ključa
batch-ai-fail-unreadable = Snimak nije mogao da se pročita
batch-ai-fail-too-long = Klip je predugačak za AI (preko 30 min)
batch-ai-fail-not-saved = Opis nije mogao da se sačuva
batch-ai-stop-offline = Zaustavljeno: nema veze sa uslugom { $provider }. Pokrenite ponovo da se opiše ostalo.

batch-action-generate-subtitles = Napravi titlove
batch-action-generate-subtitles-install-ffmpeg = da biste čitali .mkv, .m2ts, .avi …, instalirajte ffmpeg sa ffmpeg.org, dodajte ga u PATH i ponovo pokrenite frename
batch-action-generate-subtitles-replace = Zameni postojeće titlove
batch-action-generate-subtitles-replace-hint = Ponovo transkribuje; košta kao što je prikazano.
batch-action-generate-subtitles-srt = SRT titlovi (clip.srt)
batch-action-generate-subtitles-premiere = Premiere Pro transkript (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = U programu Premiere Pro: panel Text → Transcript → Import Static Transcript.
batch-action-generate-subtitles-privacy = Zvuk ovih snimaka se šalje usluzi Soniox i nakon toga se tamo briše.
batch-action-generate-subtitles-duration-hint = Traje nekoliko minuta po satu zvuka. { batch-runs-in-background }. Zatvaranje programa frename zaustavlja pokretanje; gotovi titlovi se zadržavaju.
batch-action-generate-subtitles-hint = Transkribuje govor svakog označenog snimka pomoću usluge Soniox i čuva rezultat pored njega: titlove (clip.srt), gde ih frename prikazuje, i/ili Premiere Pro transkript.
batch-subtitles-transcribe = Transkribuj
batch-subtitles-transcribe-count = Transkribuj { $videos }
batch-subtitles-build-free = Napravi { $count } (besplatno)
batch-subtitles-nothing = Nema šta da se transkribuje
batch-subtitles-estimating = Procena…
batch-subtitles-key-missing = Podesite Soniox API ključ u podešavanjima
batch-subtitles-key-rejected = Soniox je odbio ključ
batch-subtitles-cost-unknown = cena je nepoznata
batch-subtitles-typical-price = { $usd } (uobičajena cena)
batch-subtitles-unknown-length = + { $n } nepoznate dužine
batch-subtitles-plan-line = { $videos } za transkripciju, { $duration } zvuka{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } već ima titlove
    [few] { $n } već imaju titlove
   *[other] { $n } već imaju titlove
}
batch-subtitles-rebuilt-free = Ponovo napravljeno besplatno iz sačuvanog transkripta: { $n }
batch-subtitles-no-speech-before = Bez govora prošli put: { $n }
batch-subtitles-no-audio = { $n ->
    [one] { $n } nema zvuk
    [few] { $n } nemaju zvuk
   *[other] { $n } nemaju zvuk
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } deli naziv titla sa drugim snimkom
    [few] { $n } dele naziv titla sa drugim snimkom
   *[other] { $n } dele naziv titla sa drugim snimkom
}
batch-subtitles-unreadable = Ne može se pročitati: { $n } ({ $how_to })
batch-subtitles-shared-name-reason = deli naziv titla sa drugim snimkom
batch-subtitles-unsupported-reason = format zvuka nije podržan
batch-subtitles-progress-transcribing = transkripcija…
batch-subtitles-progress-extracting = izdvajanje zvuka { $percent } %
batch-subtitles-progress-extracting-unknown = izdvajanje zvuka
batch-subtitles-progress-uploading = otpremanje { $percent } %
batch-subtitles-progress-uploading-unknown = otpremanje
batch-subtitles-progress-transcribing-left = transkripcija… preostalo još oko { $time }
batch-subtitles-progress-saving = čuvanje
batch-subtitles-reason-already = već ima titlove
batch-subtitles-reason-no-audio = nema zvuka
batch-subtitles-reason-no-speech = nema govora
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox nije dostupan
batch-subtitles-stop-key-rejected = Zaustavljeno: Soniox je odbio ključ. Proverite ga u Podešavanja → Titlovi.
batch-subtitles-stop-balance-empty = Zaustavljeno: stanje na računu Soniox je prazno. Dopunite ga na console.soniox.com.
batch-subtitles-stop-budget-used = Zaustavljeno: mesečni budžet usluge Soniox je potrošen. Povećajte ga na console.soniox.com.
batch-subtitles-stop-other = Zaustavljeno: Soniox je odbio da nastavi ({ $error_type }).
batch-subtitles-stop-unreachable = Zaustavljeno: Soniox nije dostupan. Proverite internet vezu.
batch-subtitles-report-spend = Soniox: najmanje { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } otpremanje nije moglo da se obriše
    [few] { $n } otpremanja nisu mogla da se obrišu
   *[other] { $n } otpremanja nisu mogla da se obrišu
} na usluzi Soniox (pogledajte dnevnik)
batch-subtitles-languages-need-key = Sačuvajte Soniox ključ da biste birali među svim njegovim jezicima.
batch-subtitles-languages-rejected = Soniox je odbio ključ.
batch-subtitles-languages-failed = Jezici od usluge Soniox nisu mogli da se preuzmu.
batch-subtitles-count = { $n ->
    [one] { $n } titl
    [few] { $n } titla
   *[other] { $n } titlova
}
batch-subtitles-duration-seconds = { $s } s
batch-subtitles-usd-under = manje od $0.01
batch-subtitles-usd-about = oko ${ $amount }

## File list
folder-has-subtitles = Ima titlove
folder-closed-by-job = Zatvoreno dok ga grupna radnja menja
folder-filter-tip = Prikaži samo…

folder-search-placeholder = Pronađi datoteku ili komentar
folder-search-comments-loading = Pretraga komentara… preostalo: { $n }
folder-search-clear = Obriši
folder-all = Sve
folder-invert = Obrni
folder-checked = označeno: { $count }
folder-outcome-changed = Izmenjeno
folder-outcome-unchanged = Nema šta da se menja
folder-outcome-failed = Nije uspelo, pogledajte dnevnik
folder-rename-error-empty = Naziv je prazan
folder-rename-error-bad-character = Nije dozvoljeno: \ / : * ? " < > |
folder-rename-error-trailing = Ne sme da se završava tačkom ili razmakom
folder-rename-error-exists = Datoteka sa ovim nazivom već postoji
folder-markers-not-saved = Markeri nisu sačuvani: datoteka je samo za čitanje ili je u upotrebi (zatvorite je u programu Premiere, pa otvorite datoteku i napustite je ponovo)
drag-out-not-saved = Nije prevučeno: datoteka ne može da se sačuva (samo za čitanje ili otvorena u programu Premiere)
folder-opening = Otvaranje fascikle…
folder-empty-title = U ovoj fascikli nema snimaka
folder-empty-line = frename prikazuje MP4, MOV, MKV i druge video datoteke.
folder-open-another = Otvori drugu fasciklu…
folder-no-match = Nijedna datoteka ne odgovara
folder-show-all = Prikaži sve
folder-locked = Označavanje je zaključano dok traje: { $action }
folder-checked-hidden = označeno: { $count } · skriveno: { $hidden }
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } označena datoteka je skrivena pretragom ili filterom; radnja se izvršava i nad njom
    [few] { $hidden } označene datoteke su skrivene pretragom ili filterom; radnja se izvršava i nad njima
   *[other] { $hidden } označenih datoteka je skriveno pretragom ili filterom; radnja se izvršava i nad njima
}
folder-outcome-working = U toku
folder-outcome-not-reached = Nije obrađeno
folder-window-empty-title = Otvorite fasciklu sa klipovima
folder-window-empty-line = Ili prevucite fasciklu na prozor. Desni klik na dugme otvara jednu datoteku.
folder-window-open = Otvori fasciklu…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Nedavne fascikle
recent-folders-title = Nedavne fascikle
recent-folders-none = Još nijedna fascikla nije otvorena
recent-folders-not-found = Nije pronađeno
recent-folders-remove-tip = Ukloni sa spiska
recent-folders-clear = Obriši spisak
recent-folders-remove = Ukloni
recent-folders-keep = Zadrži
recent-folders-missing-line = Nije pronađeno. Ukloniti?
recent-folders-just-now = Upravo sada
recent-folders-minutes = pre { $count } min
recent-folders-hours = pre { $count } č
recent-folders-days = { $count ->
    [one] pre { $count } dan
    [few] pre { $count } dana
   *[other] pre { $count } dana
}
recent-folders-weeks = { $count ->
    [one] pre { $count } nedelju
    [few] pre { $count } nedelje
   *[other] pre { $count } nedelja
}
recent-folders-months = { $count ->
    [one] pre { $count } mesec
    [few] pre { $count } meseca
   *[other] pre { $count } meseci
}
recent-folders-years = { $count ->
    [one] pre { $count } godinu
    [few] pre { $count } godine
   *[other] pre { $count } godina
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Prikaži u programu Explorer
file-menu-show-in-file-manager = Prikaži u upravniku datoteka
file-menu-copy-path = Kopiraj punu putanju
file-menu-copy-name = Kopiraj naziv datoteke
file-menu-copied = Kopirano
file-menu-not-copied = Nije kopirano: klipbord nije dostupan
file-menu-not-shown = Upravnik datoteka nije mogao da se otvori

## Controls bar under the file list

folder-controls-filter-untagged = Bez tegova
folder-controls-filter-subtitles = Titlovi
folder-controls-filter-comments = Komentari
folder-controls-filter-markers = Markeri
folder-controls-scroll = Prikaži otvorenu datoteku na spisku
folder-controls-open = Otvori fasciklu
folder-controls-batch = Grupne radnje nad označenim datotekama
folder-controls-batch-back = Nazad na otvorenu datoteku
app-bar-single = Jedna datoteka
app-bar-batch = Grupa
folder-controls-update-available = Ažuriranje je dostupno: { $version }
folder-controls-whats-new = Šta je novo u verziji { $version }
folder-controls-previous = Prethodna datoteka
folder-controls-next = Sledeća datoteka
folder-controls-open-file = Desni klik: otvori jednu datoteku

## Video

video-controls-back = Nazad 10 s
video-controls-play = Pusti
video-controls-pause = Pauziraj
video-controls-forward = Napred 10 s
video-controls-frame-back = Jedan kadar unazad
video-controls-frame-forward = Jedan kadar unapred
video-controls-set-in = Postavi ulaznu tačku
video-controls-set-out = Postavi izlaznu tačku
video-controls-screenshot = Sačuvaj ovaj kadar
video-controls-add-marker = Dodaj marker
video-controls-add-marker-hold = Držite za opseg; pritisnite ponovo da mu date naziv
video-controls-cannot-hold-markers = Ova datoteka ne može da sadrži markere
video-controls-add-a-name = Dodaj naziv
video-controls-rotate-left = Rotiraj ulevo
video-controls-rotate-right = Rotiraj udesno
video-controls-more = Još
media-viewer-loading-slow = Čeka se datoteka… (datoteka u oblaku može da potraje)
media-viewer-cannot-play = Ovaj klip ne može da se pusti
media-viewer-no-picture = Ova datoteka nema video sliku
video-timecode-copied = Vremenska oznaka kopirana: { $time }
video-timecode-unreadable = „{ $text }“ nije vreme u ovom klipu
video-controls-volume-scroll = Jačina zvuka — pomerajte točkić da je promenite
rotate-cannot = Rotacija nije moguća: { $reason }
rotate-reason-missing = datoteke više nema
rotate-flag-right = 90° udesno
rotate-flag-left = 90° ulevo
rotate-flag-half = 180°
rotate-flag-none = nema
rotate-now = Rotacija: { $flag }
rotate-turned = Okrenuto { $turn } · rotacija je sada { $flag }
rotate-failed = Nije rotirano: { $reason }
undo-name-taken = Nije promenjeno: druga datoteka se već zove { $name }
undo-back-on-clip = Ponovo na ovom klipu (premeštanje je poništeno). Ctrl+Z još jednom poništava njegovu poslednju izmenu.
redo-on-clip = Ponovo na ovom klipu (premeštanje je vraćeno).
rotate-reason-in-use = datoteka je samo za čitanje ili je u upotrebi
rotate-reason-format = ovaj format nema oznaku rotacije
rotate-reason-damaged = datoteka je oštećena
rotate-reason-no-video = datoteka nema video traku
rotate-reason-matrix = video ima neobičnu matricu prikaza
batch-action-rotate = Rotiraj snimke
batch-action-rotate-hint = Menja oznaku rotacije svake označene MP4 i MOV datoteke; slika se ne kodira ponovo, a komentari, ulazne i izlazne tačke i markeri ostaju. Očekuje se da Premiere Pro prilikom uvoza prikaže klip okrenut. Drugi formati nemaju oznaku rotacije i ne uspevaju. Ponovno pokretanje ponovo okreće datoteke; „Resetuj“ uklanja svaku rotaciju, pa i onu koju je snimio telefon, tako da portretni klip sa telefona posle toga leži na boku.
batch-action-rotate-right = 90° udesno (u smeru kazaljke na satu)
batch-action-rotate-left = 90° ulevo (suprotno od kazaljke na satu)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Resetuj: bez rotacije (0°)
media-viewer-video-subtitle-list = Spisak titlova
media-viewer-video-marker-list = Spisak markera
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 skaču između markera; Shift+prevlačenje poravnava
media-viewer-video-tab-markers = Markeri
media-viewer-video-fullscreen = Ceo ekran
media-viewer-video-close-list = Zatvori spisak
media-viewer-video-subtitle-file-unreadable = Ova datoteka nije mogla da se pročita
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Nastavljeno od { $time } · Home: od početka

## Markers list
markers-in-out = Ulazne i izlazne tačke

markers-empty = Još nema markera
markers-add = Dodaj marker
markers-ai-hint = AI marker: zamenjuje se kada AI ponovo opiše ovaj klip
markers-keep-color = Zadrži boju
markers-done = Gotovo
markers-delete = Obriši marker
markers-cannot-hold-hint = Premiere čita markere iz MP4 i MOV datoteka.
markers-color-green = Zelena
markers-color-red = Crvena
markers-color-orange = Narandžasta
markers-color-yellow = Žuta
markers-color-white = Bela
markers-color-blue = Plava
markers-color-cyan = Cijan
markers-color-lavender = Lavanda
markers-color-magenta = Magenta
markers-color-other = Druga boja
markers-read-only = samo za čitanje
markers-name-placeholder = Naziv
markers-ai-describe = Opis pomoću AI: daj markeru naziv i dodaj šta se dešava
markers-ai-stop = Zaustavi opis
markers-ai-describe-unnamed = Opiši bez naziva: { $count }
markers-ai-describe-unnamed-hint = Daje naziv i opisuje pomoću AI svaki marker koji nema naziv, po nekoliko odjednom. Po jedan zahtev za svaki; naziv koji u međuvremenu ukucate se zadržava.
markers-ai-price-about = oko { $dollars }
markers-ai-describe-unnamed-cost = Po jedan zahtev za svaki: { $each } po markeru, { $total } za sve.
markers-ai-confirm = { $count ->
    [one] Opisati { $count } marker? Po jedan zahtev za svaki: ukupno { $total }.
    [few] Opisati { $count } markera? Po jedan zahtev za svaki: ukupno { $total }.
   *[other] Opisati { $count } markera? Po jedan zahtev za svaki: ukupno { $total }.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Opisati { $count } marker? Po jedan plaćen zahtev za svaki.
    [few] Opisati { $count } markera? Po jedan plaćen zahtev za svaki.
   *[other] Opisati { $count } markera? Po jedan plaćen zahtev za svaki.
}
markers-ai-confirm-yes = Opiši
markers-ai-confirm-no = Otkaži
markers-ai-waiting = Čeka na red…
markers-ai-run = Opisuje se: { $describing } · čeka: { $waiting }
markers-ai-run-last = Opisuje se: { $describing }
markers-ai-stop-all = Zaustavi sve
markers-ai-describing = Opisivanje…
markers-ai-done = Marker je opisan
markers-ai-nothing-new = Marker već ima ovaj opis
markers-no-marker-notice = Ovde nema markera
markers-ai-no-key = Nema API ključa za { $provider }: podesite ga u podešavanjima
markers-ai-failed = Nije opisano: { $reason }
markers-ai-failed-unknown = nešto je pošlo naopako
markers-read-only-notice = Taj marker je samo za čitanje
markers-ai-gone = Markera više nema: njegov opis nije dodat
markers-ai-stopping-for-batch = Prvo se zaustavljaju opisi markera… Obično traje nekoliko sekundi.
folder-not-saved-marker-ai = Nije sačuvano: opis markera je još čitao klip. Pokušajte ponovo za trenutak.

## File workspace

file-workspace-search-placeholder = Pronađite teg — ili samo kucajte
file-workspace-search-clear = Obriši
file-workspace-comment-placeholder = Komentar...
file-workspace-comment-collapse = Nazad na tegove
file-workspace-comment-expand = Proširi komentar

## Tag grid

tag-grid-star = Zvezdica: drži ga pri vrhu
tag-grid-unstar = Ukloni zvezdicu
tag-grid-save = Dodaj u tegove fascikle
tag-grid-delete = Obriši „{ $tag }“ iz tegova fascikle
tag-grid-create = Napravi „{ $tag }“
tag-grid-no-file = Otvorite klip da biste ga označili tegom
tag-grid-no-tags = Još nema tegova
tag-grid-no-tags-hint = Unesite naziv i pritisnite Enter da napravite prvi.
tag-grid-group-unsaved = Nisu u tegovima fascikle
tag-grid-group-folder = Tegovi fascikle
tag-grid-more = još +{ $count }

## Order strip

sync-panel-locked = Promena redosleda ispod menja fasciklu
sync-panel-unlocked = Promena redosleda ispod menja samo ovaj klip
sync-panel-unlock = Otključaj
sync-panel-lock = Zaključaj
sync-panel-differs = Redosled se razlikuje od fascikle
sync-panel-use-for-folder = Primeni na fasciklu
sync-panel-sort-like-folder = Poređaj kao u fascikli

## File name card

file-name-panel-no-tags = Ovaj klip nema tegova
file-name-panel-untag = Skini teg
file-name-panel-clear-in = Obriši ulaznu tačku
file-name-panel-clear-out = Obriši izlaznu tačku
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = Postavi In i Out na ono što AI predlaže (zaokruženo na cele sekunde)
file-name-panel-suggested-tags = AI tegovi
file-name-panel-add-suggested-tag = Dodaj teg „{ $tag }“ (sigurnost { $percent }%)
file-name-panel-add-suggested-tag-unsure = Dodaj teg „{ $tag }“
file-name-panel-add-all-suggested-tags = Dodaj sve
file-name-panel-add-all-suggested-tags-tip = Dodaj svaki teg koji AI predlaže
file-name-panel-tag-ideas = Ideje: { $ideas }
file-name-panel-tag-ideas-tip = Tegovi koje je AI primetio, a koje ova fascikla nema. Oni nisu dodati.
file-name-panel-saved = Sve izmene su sačuvane
file-name-panel-saved-to-recovery = Sačuvano za oporavak, primenjuje se kada napustite klip

## Updates (Settings)

updates-check = Proveri ažuriranja
updates-not-installed = Ažuriranja rade u instaliranoj verziji
updates-from-store = Ažuriranja dolaze iz Microsoft Store-a
updates-from-releases = Nove verzije: preuzmite ih sa stranice izdanja
updates-open-releases = Otvori stranicu izdanja
updates-checking = Provera…
updates-downloading-named = Preuzimanje { $version }… { $percent }%
updates-downloading = Preuzimanje… { $percent }%
updates-restarting = Ponovno pokretanje…
updates-check-failed = Provera ažuriranja nije uspela: { $reason }
updates-update-failed = Ažuriranje nije uspelo: { $reason }
updates-close-other-window = Prvo zatvorite drugi frename prozor
updates-reason-github = GitHub nije odgovorio, pokušajte kasnije
updates-reason-no-connection = nema veze
updates-reason-read-only = fascikla je samo za čitanje
updates-reason-not-installed = ažuriranja rade u instaliranoj verziji
updates-reason-download-stopped = preuzimanje se neočekivano zaustavilo
updates-version-available = Dostupna je verzija { $version }
updates-up-to-date = frename je ažuran
updates-update-and-restart = Ažuriraj i ponovo pokreni
updates-wait-for-batch = Sačekajte da se grupna radnja završi
updates-current-version = frename { $version }
updates-check-on-start = Proveri ažuriranja kada se frename pokrene
updates-notes-show = Beleške o izdanjima
updates-notes-hide = Sakrij beleške o izdanjima
updates-notes-all-releases = Sva izdanja na GitHub-u
updates-notes-history = Sve verzije, najnovije prvo
updates-notes-update = Šta donosi verzija { $version }
updates-notes-whats-new = Šta je novo u verziji { $version }
updates-notes-loading = Učitavaju se beleške o izdanju…
updates-notes-unavailable = Beleške o najnovijoj verziji nisu mogle da se učitaju.
updates-notes-more = Prikaži još

recovery-restored = frename se neočekivano zatvorio. Vraćen je vaš nesačuvani rad na: { $clip } ({ $what }).
recovery-kept = frename se neočekivano zatvorio. Vaš nesačuvani rad na: { $clip } ({ $what }) nije primenjen: { $why }. Čitljiva kopija za ponovno kucanje nalazi se u: { $folder }.
recovery-unreadable = frename se neočekivano zatvorio, a datoteka za oporavak nije mogla da se pročita. Čuva se u: { $folder }.
recovery-why-gone = klipa više nema ili je preimenovan
recovery-why-changed = klip je izmenjen nakon vaših izmena
recovery-why-not-written = datoteka sa tim nazivom već postoji, ili je klip samo za čitanje ili u upotrebi
recovery-tags = { $count ->
    [one] { $count } teg
    [few] { $count } tega
   *[other] { $count } tegova
}
recovery-markers = { $count ->
    [one] { $count } marker
    [few] { $count } markera
   *[other] { $count } markera
}
recovery-comment = komentar
recovery-in-out = ulazne i izlazne tačke
recovery-edits = izmene
folder-frame-saved = Kadar je sačuvan
folder-not-saved = Nije sačuvano: datoteka sa tim nazivom već postoji, ili je samo za čitanje ili u upotrebi
markers-deleted-notice = Marker je obrisan
markers-not-saved-notice = Markeri nisu sačuvani: datoteka je samo za čitanje ili je u upotrebi
markers-comment-not-saved-notice = Markeri nisu sačuvani: datoteka komentara je samo za čitanje ili je u upotrebi
markers-copy = Kopiraj
markers-copy-tip = Kopiraj markere kao tekst, po jedan red
markers-export = Izvezi…
markers-export-tip = Sačuvaj markere kao tekstualnu datoteku
markers-exported = Markeri su sačuvani u: { $name }
markers-not-exported = Markeri nisu sačuvani: datoteka nije mogla da se zapiše

batch-option-notes = Beleške za veštačku inteligenciju
batch-ai-notes-info = Vaše beleške o snimcima u ovoj fascikli. Šalju se uz svaki zahtev ka veštačkoj inteligenciji ovde: opisi, predlozi tegova i opisi markera. Čuvaju se u datoteci .frename fascikle.
batch-ai-notes-placeholder = npr. Čovek koji se najčešće pojavljuje je Evgenije. Označavaj reklamne bilborde. Predloži teg čaj gde god vidiš čaj.
batch-ai-notes-hint = Šalju se dobavljaču { $provider } zajedno sa kadrovima. Imena dolaze samo iz ovih beleški; frename ne prepoznaje lica. Već opisani snimci se preskaču: označite Ponovi da biste ih opisali ponovo sa novim beleškama.
batch-ai-notes-too-long = Beleške imaju { $chars } znakova, ograničenje je { $max }. Skratite ih da biste pokrenuli.
batch-ai-notes-too-long-reason = Beleške za veštačku inteligenciju su predugačke
batch-option-notes-once = Dodatni red za ovo pokretanje
batch-ai-notes-once-placeholder = npr. U ovom klipu: žena u crvenom je Ana
file-workspace-ai-notes-on = Beleške su uključene
file-workspace-ai-notes-tip = Beleške za veštačku inteligenciju idu uz svaki zahtev u ovoj fascikli. Menjaju se u Opiši pomoću veštačke inteligencije (grupne radnje): { $notes }
batch-ai-notes-no-subtitles = Izgleda da se beleške odnose na ono što se govori, a { $videos } nemaju titlove: prvo ih napravite (Pravljenje titlova) da bi veštačka inteligencija mogla da čita reči.
