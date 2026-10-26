# Текст интерфејса frename на српском (ћирилица). Исти идентификатори, атрибути и $аргументи као у
# i18n/en/frename.ftl, истим редоследом (проверавају их тестови у src/i18n.rs).
# Латиничну верзију (i18n/sr-Latn) прави транслитерација ове датотеке, па се овде не пише ништа
# што се не може једнозначно пренети у латиницу.
#
# Glossary (docs/design/localization.md): every string uses these terms.
#   tag / tags                        тег / тегови (never «ознака»: that is Premiere's Label)
#   file                              датотека
#   folder                            фасцикла
#   video / clip                      снимак / клип
#   in / out points                   улазна / излазна тачка
#   comment                           коментар
#   subtitles                         титлови
#   marker (Premiere)                 маркер
#   range (a marker with a duration)  опсег
#   checked (files)                   означено
#   All / Invert (check boxes)        Све / Обрни
#   Cancel / Stopping…                Откажи / Заустављање…
#   log, "see the log"                дневник, «детаље погледајте у дневнику»
#   System (language list)            Као у систему (…)
#   batch action                      групна радња
#   settings                          подешавања
#   update (a newer frename)          ажурирање
#   AI description (Describe with AI) AI опис
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

settings-window-title = Подешавања
settings-page-interface = Интерфејс
settings-page-saving = Чување
settings-page-ai = Опис помоћу AI
settings-close = Затвори
settings-apply-note = Измене се примењују одмах. Tab пребацује између опција, Ctrl+Tab између страница.
settings-language = Језик
settings-language-system = Као у систему ({ $language })
settings-video = Видео
settings-video-autoplay = Аутоматски пусти снимке када се отворе
settings-subtitle-size = Величина титлова
settings-subtitle-size-small = Мала
settings-subtitle-size-medium = Средња
settings-subtitle-size-large = Велика
settings-subtitle-size-extra-large = Врло велика
settings-timecode = Приказ времена
settings-timecode-hint = Како се записује време испод снимка, у списковима маркера и титлова и на ознакама In / Out, као и време положаја репродукције које копира Ctrl+Shift+C. Premiere Pro и DaVinci Resolve приказују часове:минуте:секунде:кадрове према брзини кадрова клипа; 29,97 и 59,94 fps су drop-frame. Клип без стабилне брзине кадрова задржава подразумевани стил. Датотеке задржавају своје формате.
settings-timecode-default = Подразумевано (мм:сс)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Боје тегова
settings-tags-monochrome = Једнобојно
settings-tags-monochrome-hint = Тегови које сте додали приказују се тамнијом сивом, остали светлијом, без других боја.
settings-file-names = Називи датотека
settings-tags-space-after = Размак после сваког тега
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Датотеке задржавају називе док се не преименују или сачувају.
settings-tags-space-add = Додај размак у називе датотека…
settings-tags-space-remove = Уклони размак из назива датотека…
settings-comments = Коментари
settings-comments-in-video = Унутар видео датотеке
settings-comments-in-video-hint = XMP, колона Description у програму Premiere Pro
settings-comments-text-file = У текстуалној датотеци поред снимка
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Датотеке задржавају коментаре где јесу док се не преместе.
settings-comments-move-into-videos = Премести коментаре у снимке…
settings-comments-move-into-text-files = Премести коментаре у текстуалне датотеке…
settings-commented-tag = Тег за снимке са коментаром
settings-commented-tag-name = Тег
settings-commented-tag-hint = Означава се када напишете коментар уз снимак, нпр. { $tag }.IMG_0424.MOV, а скида када га обришете (AI описи се не рачунају). Иначе га мењате сами.
settings-commented-tag-off = Снимци са коментаром не добијају тег.
settings-in-out = Улазне и излазне тачке
settings-in-out-in-video = Унутар видео датотеке
settings-in-out-in-video-hint = Маркер подклипа у програму Premiere Pro
settings-in-out-comment = У коментару, у једном реду
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Датотеке задржавају улазне и излазне тачке где јесу док се не преместе.
settings-in-out-move-into-videos = Премести улазне и излазне тачке у снимке…
settings-in-out-move-into-comments = Премести улазне и излазне тачке у коментаре…
settings-markers = Маркери и опсези
settings-markers-in-video = Унутар видео датотеке
settings-markers-in-video-hint = XMP, види се на клипу у програму Premiere Pro
settings-markers-comment = У коментару, по један ред
settings-markers-comment-example = 0:41–0:47 — Лав
settings-markers-note = Датотеке задржавају маркере где јесу док се не преместе.
settings-markers-move-into-videos = Премести маркере у снимке…
settings-markers-copy-into-comment = Копирај маркере у коментаре…
settings-markers-hint = И тачке и опсези, као и тренуци које пронађе AI.
settings-updates = Ажурирања
settings-version = Верзија
settings-release-notes = Белешке о издању
settings-old-title = Подешавања из старијег frename-а
settings-old-hint = Пренесите подешавања frename-а који сте покретали из zip фасцикле.
settings-old-scheduled = Подешавања ће бити увезена када се frename поново покрене
settings-old-not-found = Нема frename.exe са frename.db у { $folder }
settings-old-failed = Увоз није успео: { $reason }
settings-old-import-button = Увези из старе frename фасцикле…
settings-ai = Опис помоћу AI
settings-ai-key-label = Anthropic API кључ
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Кључ добијате на console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Уклонити сачувани Anthropic кључ?
settings-ai-used-by = Користи га радња „Опис помоћу AI“ у групном режиму и на маркерима (✨, Ctrl+F2), када је изабран Claude модел.
settings-ai-openai-key-label = OpenAI API кључ
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Кључ добијате на platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Уклонити сачувани OpenAI кључ?
settings-ai-openai-used-by = Користи га радња „Опис помоћу AI“ у групном режиму и на маркерима (✨, Ctrl+F2), када је изабран GPT модел.
settings-ai-gemini-key-label = Gemini API кључ
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Кључ добијате на aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Уклонити сачувани Gemini кључ?
settings-ai-gemini-used-by = Користи га радња „Опис помоћу AI“ у групном режиму и на маркерима (✨, Ctrl+F2), када је изабран Gemini модел.
settings-ai-model-needs-key = { $model } — прво додајте кључ за { $provider }
settings-ai-model-label = Модел
settings-ai-language-label = Језик описа
settings-ai-hint = Haiku, GPT-4.1 mini и Gemini Flash-Lite су најјефтинији; Sonnet, Opus, GPT-4.1 и Gemini Flash уочавају више. Модел захтева кључ своје компаније.
settings-ai-moments-label = Тренуци
settings-ai-moments-important = Само оно што се истиче
settings-ai-moments-full = Обухвати цео клип
settings-ai-moments-important-hint = Предлаже и где поставити In и Out (AI ознака поред IN и OUT)
settings-ai-moments-full-hint = Тренутак за сваки део, и за тихе; без предлога за In и Out
settings-ai-tag-suggestions = Предложи и тегове из тегова фасцикле
settings-ai-tag-suggestions-hint = У истом захтеву AI каже који тегови фасцикле одговарају сваком клипу. Предложени тег се додаје тек када кликнете на њега.
settings-subtitles = Титлови
settings-subtitles-key-label = Soniox API кључ
settings-subtitles-key-placeholder = Налепите кључ
settings-subtitles-key-get = Кључ добијате на console.soniox.com. Звук се шаље услузи Soniox.
settings-subtitles-key-remove-confirm = Уклонити сачувани Soniox кључ?
settings-subtitles-languages-label = Језици
settings-subtitles-languages-locked = Сачувајте кључ да бисте бирали међу свим језицима које Soniox познаје.
settings-subtitles-languages-loading = Преузимање језика од услуге Soniox…
settings-subtitles-languages-none = Ништа није означено: језик се препознаје аутоматски.
settings-subtitles-languages-hint = Језици који се говоре у снимку, као наговештај.
settings-subtitles-cue-length-label = Дужина титла
settings-subtitles-cue-short = Кратка
settings-subtitles-cue-short-hint = Један ред, до 8 с
settings-subtitles-cue-sentence = Једна реченица
settings-subtitles-hint = Користи га радња „Направи титлове“ у групном режиму. Дужина титла важи за нове титлове.
settings-key-store-windows = Windows Credential Manager
settings-key-store-macos = macOS Keychain
settings-key-store-other = системски привезак кључева
settings-key-unavailable = Системски привезак кључева није могао да се отвори
settings-key-unavailable-hint = Можда је закључан или га нема (као што су GNOME Keyring или KWallet). Подешавања проверавају поново при сваком отварању.
settings-key-remove-confirm-hint = Мораћете поново да га налепите.
settings-key-remove-ask = Уклони…
settings-key-remove = Уклони кључ
settings-key-keep = Задржи
settings-key-replace = Замени…
settings-key-hide = Сакриј
settings-key-show = Прикажи
settings-key-save = Сачувај кључ
settings-key-get = Набави кључ
settings-key-billing = Додај средства
settings-key-cancel = Откажи
settings-key-saved-in = Сачувано у: { $store }, на овом рачунару
settings-key-save-into = „Сачувај кључ“ га чува у: { $store }, на овом рачунару.

## Batch mode

batch-checked-count = { $count ->
    [one] Означена је { $count } датотека
    [few] Означене су { $count } датотеке
   *[other] Означено је { $count } датотека
}
batch-no-key = Нема кључа
batch-group-move = Премештање између места
batch-group-fix = Исправљање назива и снимака
batch-group-paid = Плаћене услуге
batch-reason-none-checked = Ниједна датотека није означена
batch-reason-reading = Читање дужина клипова…
batch-check-all = Означи { $count ->
    [one] све ({ $count } датотека)
    [few] све ({ $count } датотеке)
   *[other] све ({ $count } датотека)
}
batch-job-running = ради
batch-job-finished = завршено
batch-job-stopped = заустављено
batch-progress-files = { $finished } од { $total ->
    [one] { $total } датотеке
    [few] { $total } датотеке
   *[other] { $total } датотека
}
batch-time-left = преостало још око { $time }
batch-time-estimating = процена времена…
batch-time-spent = до сада { $time }
batch-runs-in-background = Можете да наставите да радите док траје
batch-clip-in-work = У употреби: { $action }
batch-folder-held = Заузето: { $action }
batch-indicator-hint = Кликните за панел и отказивање
batch-action-prepare = Припреми клипове
batch-action-prepare-hint = За сваки означени клип, у једном пролазу: титлови (Soniox) ако их нема, па опис помоћу AI: резиме, његови тренуци као маркери, коментар и предлози тегова. Једна цена, једно ограничење и један напредак за све.
batch-prepare-step-off = искључено
batch-prepare-nothing = Нема шта да се уради за ове клипове
batch-prepare-report-cap = Потрошено { $spent } од ограничења { $cap }
batch-option-steps = Кораци
batch-prepare-step-subtitles = Прво направи титлове (Soniox)
batch-prepare-step-description = Опиши клипове помоћу AI
batch-prepare-cap = Заустави трошење на
batch-prepare-cap-none = Без ограничења (америчких долара)
batch-prepare-cap-invalid = Ограничење је износ у доларима, на пример 5
batch-prepare-left = Последње покретање је стало, преостало је још: { $count ->
    [one] { $count } клип
    [few] { $count } клипа
   *[other] { $count } клипова
}
batch-prepare-check-left = Означи их
batch-prepare-estimating = Процена шта ће сваки корак урадити…
batch-prepare-plan-subtitles = Титлови
batch-prepare-plan-description = Опис
batch-prepare-plan-none = нема шта да се ради
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } снимак
    [few] { $videos } снимка
   *[other] { $videos } снимака
} за транскрипцију, { $duration } звука
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } снимак
    [few] { $videos } снимка
   *[other] { $videos } снимака
} за опис
batch-prepare-resume-note = Сваки корак прескаче оно што већ постоји, па се заустављено покретање наставља поновним покретањем: ништа се не плаћа два пута.
batch-prepare-run = Припреми { $clips ->
    [one] { $clips } клип
    [few] { $clips } клипа
   *[other] { $clips } клипова
} · око { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Слични снимци
batch-prepare-groups-hint = Клипови који изгледају као исти кадар: дупликати, поновни извози, камера која се није померала.
batch-prepare-group-clips = { $count ->
    [one] { $count } клип
    [few] { $count } клипа
   *[other] { $count } клипова
}
batch-prepare-check-group = Означи ове клипове
batch-solo-queued = У реду чекања: { $action }
batch-suggest-in-job = Опис овог клипа већ је у послу који је у току, без предлога тегова. Покушајте поново када се заврши
batch-suggest-no-tags = У овој фасцикли још нема тегова за предлоге: прво додајте неке
batch-solo-gave-up = Припрема није успела
batch-solo-withdrawn = Повучено: { $action }
batch-solo-in-work = { $action } је у току. Откажите га са траке или са странице посла
batch-waiting-title = { $count ->
    [one] { $count } покретање чека
    [few] { $count } покретања чекају
   *[other] { $count } покретања чекају
}
batch-waiting-remove = Уклони
batch-indicator-waiting = чека: { $count }
file-workspace-describe-clip = Опиши овај клип помоћу AI
file-workspace-subtitles-clip = Направи титлове за овај клип
file-workspace-describe-clip-queued = Опис помоћу AI: чека на ред. Кликните да повучете
file-workspace-describe-clip-running = Опис помоћу AI: у току
file-workspace-suggest-tags-clip = Предложи тегове за овај клип помоћу AI (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Предлог тегова: чека на ред. Кликните да повучете
file-workspace-suggest-tags-clip-running = Предлог тегова: у току
file-workspace-subtitles-clip-queued = Прављење титлова: чека на ред. Кликните да повучете
file-workspace-subtitles-clip-running = Прављење титлова: у току
batch-run-again = Покрени поново за { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-result-done = Готово: { $total ->
    [one] { $total } датотека
    [few] { $total } датотеке
   *[other] { $total } датотека
}
batch-result-done-detail = Измењено: { $changed }, без измена: { $unchanged }
batch-result-problems = Готово уз проблеме: није урађено за { $failed } од { $total ->
    [one] { $total } датотеке
    [few] { $total } датотеке
   *[other] { $total } датотека
}
batch-figure-unchanged = без измена
batch-figure-not-done = није урађено
batch-figure-not-reached = није обрађено
batch-files-not-done = Датотеке за које није урађено
batch-table-file = Датотека
batch-table-why = Зашто

batch-run-move-comments = Премести { $count ->
    [one] { $count } коментар
    [few] { $count } коментара
   *[other] { $count } коментара
}
batch-run-move-in-out = Премести улазне/излазне тачке за { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-run-convert = Претвори { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-run-rotate = Ротирај { $count ->
    [one] { $count } снимак
    [few] { $count } снимка
   *[other] { $count } снимака
}
batch-run-tag = Означи тегом { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-run-fix-tags = Промени редослед тегова у { $count ->
    [one] { $count } називу
    [few] { $count } назива
   *[other] { $count } назива
}
batch-run-rename = Преименуј { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-run-reload = Поново учитај { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-run-export-markers = Извези маркере за { $count ->
    [one] { $count } датотеку
    [few] { $count } датотеке
   *[other] { $count } датотека
}
batch-service-soniox = Soniox
batch-change-renames = Преименује датотеке
batch-change-videos = Уписује у снимке
batch-change-text-files = Уписује текстуалне датотеке поред снимака
batch-change-comments = Уписује у коментаре
batch-change-subtitles = Уписује датотеке титлова поред снимака
batch-change-records = Мења само сопствене записе програма frename
batch-option-direction = Смер
batch-option-language = Језик
batch-option-tag-suggestions = Предлози тегова
batch-option-described = Описани снимци
batch-option-subtitled = Снимци са титловима
batch-option-subtitles-write = Датотеке за упис
batch-option-spacing = Размак
batch-option-tag = Тег
batch-option-turn = Окретање
batch-plan-videos = Снимци
batch-plan-length = Дужина
batch-plan-cost = Цена
batch-plan-time = Време
batch-reason-estimate = Чека се процена
batch-reason-subtitles-no-format = Означите бар једну датотеку за упис
batch-set-key = Подесите кључ…
batch-check-key = Проверите кључ…
batch-subtitles-languages-auto = Препознаје се у сваком снимку
batch-action-describe-ai-run-waiting = Опиши снимке
batch-action-fix-tags-order = Редослед: као на списку тегова, непознати тегови први.
batch-action-markers-comment-hint-short = Претвара редове коментара са временом у маркере или копира маркере у коментар.
batch-action-markers-to-comment-hint = Маркери остају; поновно покретање ништа не додаје два пута.
batch-action-rotate-hint-short = Окреће MP4 и MOV клипове према њиховој ознаци ротације, без поновног кодирања.
batch-action-tag-commented-off = Означавање снимака са коментаром је искључено
batch-action-tag-commented-choose = Изаберите тег…

batch-title = Групне радње
batch-done-label-changed = измењено
batch-done-label-subtitled = са титловима
batch-ai-at-least = најмање
batch-cancel = Откажи
batch-stopping = Заустављање…
batch-stop-over-budget = Заустављено: следећи клип би могао да премаши ограничење од { $cap } (потрошено { $spent }). Повећајте ограничење да би се урадило остало.
batch-stopped = Заустављено након { $finished } од { $total ->
    [one] { $total } датотеке
    [few] { $total } датотеке
   *[other] { $total } датотека
}.
batch-close = Затвори
batch-failed-subtitles = Без титлова:
batch-written-subtitles = Записане датотеке:
batch-table-written = Записано
batch-merged-markers = Спојени дупликати маркера:
batch-table-merged = Спојено
batch-markers-merged = { $count ->
    [one] Спојен је { $count } дупликат маркера
    [few] Спојена су { $count } дупликата маркера
   *[other] Спојено је { $count } дупликата маркера
}
batch-add-credit = Додај средства
batch-open-log = Отвори дневник

batch-action-move-comments = Премести коментаре
batch-action-move-comments-hint = Премешта коментар сваке означене датотеке на изабрано место, заједно са улазним и излазним тачкама које садржи. Тегови и улазне и излазне тачке сачуване у снимку остају где јесу.
batch-action-move-comments-into-videos = Из текстуалних датотека у снимке (XMP)
batch-action-move-comments-into-text-files = Из снимака (XMP) у текстуалне датотеке

batch-action-move-in-out = Улазне/излазне тачке: коментар ⇄ снимак (XMP)
batch-action-move-in-out-hint = Премешта улазне и излазне тачке сваке означене датотеке на изабрано место. Остатак коментара остаје где јесте.
batch-action-move-in-out-into-videos = Из коментара у снимке (Adobe XMP маркер)
batch-action-move-in-out-into-comments = Из снимака (XMP маркер) у коментаре

batch-action-markers-comment = Маркери ⇄ коментар
batch-action-markers-comment-to-markers = Редови коментара са временом у маркере
batch-action-markers-to-comment = Маркери у коментар (копија: маркери остају)
batch-action-markers-comment-hint = Ред као „03:24 — Take 3 — nice light“ је маркер на 3:24 под називом „Take 3“ са коментаром „nice light“; „0:41-0:47 — Lion“ је маркер од 0:41 до 0:47. Назив и коментар се деле на првом „ — “ или „ -- “, а не на обичном „ - “. Тренуци AI описа („0:00–0:14 Street.“) постају бели маркери и остају у опису. Поновно покретање у било ком смеру ништа не додаје два пута.

batch-action-tag-commented = Означи тегом снимке са коментаром
batch-action-tag-commented-hint = Додаје тег „{ $tag }“ сваком означеном снимку са вашим коментаром (AI описи се не рачунају) и уклања га са оних без коментара. Датотеке чији се тег промени се преименују.
batch-action-tag-commented-hint-off = Додаје тег за снимке са коментаром сваком означеном снимку са вашим коментаром и уклања га са оних без коментара. Тег је искључен у подешавањима.
batch-action-fix-tags = Исправи тегове по приоритету
batch-action-fix-tags-hint = Поређа тегове у називу сваке означене датотеке редом из панела тегова, па што је тег виши у њему, то је раније у називу. Тегови које фасцикла још не познаје иду први, као у панелу тегова. Датотеке чији се редослед промени се преименују.

batch-action-respace-tags = Примени размак између тегова
batch-action-respace-tags-hint-space = Преименује сваку означену датотеку тако да иза сваког тега буде размак, како је подешено у подешавањима: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Преименује сваку означену датотеку тако да иза тегова нема размака, како је подешено у подешавањима: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Размак: размак после сваког тега
batch-action-respace-tags-status-no-space = Размак: без размака после тегова
batch-action-export-markers = Извези маркере у текст
batch-action-export-markers-hint = Уписује маркере сваке означене датотеке у текстуалну датотеку поред ње, clip.markers.txt: један ред по маркеру, по реду времена, као „0:41–0:47 [red] — Lion crossing — walks left to right“. Боја различита од зелене је у угластим заградама; коментар следи иза назива. „Маркери ⇄ коментар“ враћа такве редове у маркере. Датотека без маркера не добија текстуалну датотеку, а текстуална датотека из ранијег покретања се замењује.
batch-export-markers-cannot-hold = Ова датотека не може да садржи маркере
batch-action-reload-files = Ресетуј кеш и поново учитај
batch-action-reload-files-hint = Поново чита коментар и улазне и излазне тачке сваке означене датотеке из саме датотеке и замењује оно што је фасцикла запамтила за њу. Користите то након што је датотеке променио други програм. Датотеке чије су запамћене вредности недостајале или биле застареле рачунају се као измењене.

batch-action-describe-ai = Опис помоћу AI
batch-action-describe-ai-run = Опиши { $videos } · око { $dollars }
batch-action-describe-ai-estimating = Процена… { $known } / { $total }
batch-action-describe-ai-none = Нема снимака за опис.
batch-action-describe-ai-hint = { batch-runs-in-background }. Откажи задржава већ описане снимке; поновно покретање их прескаче.
batch-action-describe-ai-no-subtitles = Без титлова (описује се само слика): { $videos }.
batch-action-describe-ai-redo = Поново обради снимке који већ имају AI опис
batch-action-describe-ai-hint-panel = Описује шта се дешава у сваком означеном снимку и када: резиме и сегменти са временским опсезима иду у AI опис његовог коментара; ваш текст се задржава. Кадрови и титлови се шаљу компанији { $provider }, произвођачу изабраног модела.
batch-ai-change = Промени
batch-ai-tag-suggestions-off = Искључено
batch-ai-tag-suggestions-no-tags = Укључено, али ова фасцикла још нема тегова
batch-ai-tag-suggestions-on = { $tags ->
    [one] Из { $tags } тега фасцикле
    [few] Из { $tags } тега фасцикле
   *[other] Из { $tags } тегова фасцикле
}
batch-ai-key-missing = Подесите API кључ за { $provider } у подешавањима
batch-ai-key-unavailable = Системски привезак кључева није могао да се отвори: можда је закључан или га нема (као што су GNOME Keyring или KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Као титлови
ai-language-english = Енглески
ai-language-russian = Руски
ai-language-ukrainian = Украјински
ai-language-german = Немачки
ai-language-spanish = Шпански
ai-language-french = Француски
batch-videos-count = { $n ->
    [one] { $n } снимак
    [few] { $n } снимка
   *[other] { $n } снимака
}
batch-minutes = { $n } мин
batch-ai-minutes-under = < 1 мин
batch-ai-duration-under-minute = мање од минута
batch-ai-duration-hours = { $h } ч
batch-ai-duration-hours-minutes = { $h } ч { $m } мин
batch-ai-dollars-under = мање од $0.01
batch-ai-skip-described = већ описано
batch-ai-skip-too-long = преко 30 мин
batch-ai-skip-unreadable = нечитљиво
batch-ai-skipped = Прескочено: { $parts }.
batch-ai-progress-frame = кадар { $done } од { $total }
batch-ai-progress-waiting = чека се { $provider }
batch-ai-progress-saving = чување
batch-ai-progress-retry-rate-limit = нови покушај за { $seconds } с (ограничење учесталости)
batch-ai-progress-retry-temporary = нови покушај за { $seconds } с (привремена грешка)
batch-ai-stop-no-key = Заустављено: нема API кључа за { $provider }. Подесите га у подешавањима.
batch-ai-fail-no-key = Нема API кључа
batch-ai-fail-unreadable = Снимак није могао да се прочита
batch-ai-fail-too-long = Клип је предугачак за AI (преко 30 мин)
batch-ai-fail-not-saved = Опис није могао да се сачува
batch-ai-stop-offline = Заустављено: нема везе са услугом { $provider }. Покрените поново да се опише остало.

batch-action-generate-subtitles = Направи титлове
batch-action-generate-subtitles-install-ffmpeg = да бисте читали .mkv, .m2ts, .avi …, инсталирајте ffmpeg са ffmpeg.org, додајте га у PATH и поново покрените frename
batch-action-generate-subtitles-replace = Замени постојеће титлове
batch-action-generate-subtitles-replace-hint = Поново транскрибује; кошта као што је приказано.
batch-action-generate-subtitles-srt = SRT титлови (clip.srt)
batch-action-generate-subtitles-premiere = Premiere Pro транскрипт (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = У програму Premiere Pro: панел Text → Transcript → Import Static Transcript.
batch-action-generate-subtitles-privacy = Звук ових снимака се шаље услузи Soniox и након тога се тамо брише.
batch-action-generate-subtitles-duration-hint = Траје неколико минута по сату звука. { batch-runs-in-background }. Затварање програма frename зауставља покретање; готови титлови се задржавају.
batch-action-generate-subtitles-hint = Транскрибује говор сваког означеног снимка помоћу услуге Soniox и чува резултат поред њега: титлове (clip.srt), где их frename приказује, и/или Premiere Pro транскрипт.
batch-subtitles-transcribe = Транскрибуј
batch-subtitles-transcribe-count = Транскрибуј { $videos }
batch-subtitles-build-free = Направи { $count } (бесплатно)
batch-subtitles-nothing = Нема шта да се транскрибује
batch-subtitles-estimating = Процена…
batch-subtitles-key-missing = Подесите Soniox API кључ у подешавањима
batch-subtitles-key-rejected = Soniox је одбио кључ
batch-subtitles-cost-unknown = цена је непозната
batch-subtitles-typical-price = { $usd } (уобичајена цена)
batch-subtitles-unknown-length = + { $n } непознате дужине
batch-subtitles-plan-line = { $videos } за транскрипцију, { $duration } звука{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } већ има титлове
    [few] { $n } већ имају титлове
   *[other] { $n } већ имају титлове
}
batch-subtitles-rebuilt-free = Поново направљено бесплатно из сачуваног транскрипта: { $n }
batch-subtitles-no-speech-before = Без говора прошли пут: { $n }
batch-subtitles-no-audio = { $n ->
    [one] { $n } нема звук
    [few] { $n } немају звук
   *[other] { $n } немају звук
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } дели назив титла са другим снимком
    [few] { $n } деле назив титла са другим снимком
   *[other] { $n } деле назив титла са другим снимком
}
batch-subtitles-unreadable = Не може се прочитати: { $n } ({ $how_to })
batch-subtitles-shared-name-reason = дели назив титла са другим снимком
batch-subtitles-unsupported-reason = формат звука није подржан
batch-subtitles-progress-transcribing = транскрипција…
batch-subtitles-progress-extracting = издвајање звука { $percent } %
batch-subtitles-progress-extracting-unknown = издвајање звука
batch-subtitles-progress-uploading = отпремање { $percent } %
batch-subtitles-progress-uploading-unknown = отпремање
batch-subtitles-progress-transcribing-left = транскрипција… преостало још око { $time }
batch-subtitles-progress-saving = чување
batch-subtitles-reason-already = већ има титлове
batch-subtitles-reason-no-audio = нема звука
batch-subtitles-reason-no-speech = нема говора
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox није доступан
batch-subtitles-stop-key-rejected = Заустављено: Soniox је одбио кључ. Проверите га у Подешавања → Титлови.
batch-subtitles-stop-balance-empty = Заустављено: стање на рачуну Soniox је празно. Допуните га на console.soniox.com.
batch-subtitles-stop-budget-used = Заустављено: месечни буџет услуге Soniox је потрошен. Повећајте га на console.soniox.com.
batch-subtitles-stop-other = Заустављено: Soniox је одбио да настави ({ $error_type }).
batch-subtitles-stop-unreachable = Заустављено: Soniox није доступан. Проверите интернет везу.
batch-subtitles-report-spend = Soniox: најмање { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } отпремање није могло да се обрише
    [few] { $n } отпремања нису могла да се обришу
   *[other] { $n } отпремања нису могла да се обришу
} на услузи Soniox (погледајте дневник)
batch-subtitles-languages-need-key = Сачувајте Soniox кључ да бисте бирали међу свим његовим језицима.
batch-subtitles-languages-rejected = Soniox је одбио кључ.
batch-subtitles-languages-failed = Језици од услуге Soniox нису могли да се преузму.
batch-subtitles-count = { $n ->
    [one] { $n } титл
    [few] { $n } титла
   *[other] { $n } титлова
}
batch-subtitles-duration-seconds = { $s } с
batch-subtitles-usd-under = мање од $0.01
batch-subtitles-usd-about = око ${ $amount }

## File list
folder-has-subtitles = Има титлове
folder-closed-by-job = Затворено док га групна радња мења
folder-filter-tip = Прикажи само…

folder-search-placeholder = Пронађи датотеку или коментар
folder-search-comments-loading = Претрага коментара… преостало: { $n }
folder-search-clear = Обриши
folder-all = Све
folder-invert = Обрни
folder-checked = означено: { $count }
folder-outcome-changed = Измењено
folder-outcome-unchanged = Нема шта да се мења
folder-outcome-failed = Није успело, погледајте дневник
folder-rename-error-empty = Назив је празан
folder-rename-error-bad-character = Није дозвољено: \ / : * ? " < > |
folder-rename-error-trailing = Не сме да се завршава тачком или размаком
folder-rename-error-exists = Датотека са овим називом већ постоји
folder-markers-not-saved = Маркери нису сачувани: датотека је само за читање или је у употреби (затворите је у програму Premiere, па отворите датотеку и напустите је поново)
drag-out-not-saved = Није превучено: датотека не може да се сачува (само за читање или отворена у програму Premiere)
folder-opening = Отварање фасцикле…
folder-empty-title = У овој фасцикли нема снимака
folder-empty-line = frename приказује MP4, MOV, MKV и друге видео датотеке.
folder-open-another = Отвори другу фасциклу…
folder-no-match = Ниједна датотека не одговара
folder-show-all = Прикажи све
folder-locked = Означавање је закључано док траје: { $action }
folder-checked-hidden = означено: { $count } · скривено: { $hidden }
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } означена датотека је скривена претрагом или филтером; радња се извршава и над њом
    [few] { $hidden } означене датотеке су скривене претрагом или филтером; радња се извршава и над њима
   *[other] { $hidden } означених датотека је скривено претрагом или филтером; радња се извршава и над њима
}
folder-outcome-working = У току
folder-outcome-not-reached = Није обрађено
folder-window-empty-title = Отворите фасциклу са клиповима
folder-window-empty-line = Или превуците фасциклу на прозор. Десни клик на дугме отвара једну датотеку.
folder-window-open = Отвори фасциклу…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Недавне фасцикле
recent-folders-title = Недавне фасцикле
recent-folders-none = Још ниједна фасцикла није отворена
recent-folders-not-found = Није пронађено
recent-folders-remove-tip = Уклони са списка
recent-folders-clear = Обриши списак
recent-folders-remove = Уклони
recent-folders-keep = Задржи
recent-folders-missing-line = Није пронађено. Уклонити?
recent-folders-just-now = Управо сада
recent-folders-minutes = пре { $count } мин
recent-folders-hours = пре { $count } ч
recent-folders-days = { $count ->
    [one] пре { $count } дан
    [few] пре { $count } дана
   *[other] пре { $count } дана
}
recent-folders-weeks = { $count ->
    [one] пре { $count } недељу
    [few] пре { $count } недеље
   *[other] пре { $count } недеља
}
recent-folders-months = { $count ->
    [one] пре { $count } месец
    [few] пре { $count } месеца
   *[other] пре { $count } месеци
}
recent-folders-years = { $count ->
    [one] пре { $count } годину
    [few] пре { $count } године
   *[other] пре { $count } година
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Прикажи у програму Explorer
file-menu-show-in-file-manager = Прикажи у управнику датотека
file-menu-copy-path = Копирај пуну путању
file-menu-copy-name = Копирај назив датотеке
file-menu-copied = Копирано
file-menu-not-copied = Није копирано: клипборд није доступан
file-menu-not-shown = Управник датотека није могао да се отвори

## Controls bar under the file list

folder-controls-filter-untagged = Без тегова
folder-controls-filter-subtitles = Титлови
folder-controls-filter-comments = Коментари
folder-controls-filter-markers = Маркери
folder-controls-scroll = Прикажи отворену датотеку на списку
folder-controls-open = Отвори фасциклу
folder-controls-batch = Групне радње над означеним датотекама
folder-controls-batch-back = Назад на отворену датотеку
folder-controls-update-available = Ажурирање је доступно: { $version }
folder-controls-whats-new = Шта је ново у верзији { $version }
folder-controls-previous = Претходна датотека
folder-controls-next = Следећа датотека
folder-controls-more = Још
folder-controls-open-file = Десни клик: отвори једну датотеку

## Video

video-controls-back = Назад 10 с
video-controls-play = Пусти
video-controls-pause = Паузирај
video-controls-forward = Напред 10 с
video-controls-frame-back = Један кадар уназад
video-controls-frame-forward = Један кадар унапред
video-controls-set-in = Постави улазну тачку
video-controls-set-out = Постави излазну тачку
video-controls-screenshot = Сачувај овај кадар
video-controls-add-marker = Додај маркер
video-controls-add-marker-hold = Држите за опсег; притисните поново да му дате назив
video-controls-cannot-hold-markers = Ова датотека не може да садржи маркере
video-controls-add-a-name = Додај назив
video-controls-rotate-left = Ротирај улево
video-controls-rotate-right = Ротирај удесно
video-controls-more = Још
media-viewer-loading-slow = Чека се датотека… (датотека у облаку може да потраје)
media-viewer-cannot-play = Овај клип не може да се пусти
media-viewer-no-picture = Ова датотека нема видео слику
video-timecode-copied = Временска ознака копирана: { $time }
video-timecode-unreadable = „{ $text }“ није време у овом клипу
video-controls-volume-scroll = Јачина звука — померајте точкић да је промените
rotate-cannot = Ротација није могућа: { $reason }
rotate-reason-missing = датотеке више нема
rotate-flag-right = 90° удесно
rotate-flag-left = 90° улево
rotate-flag-half = 180°
rotate-flag-none = нема
rotate-now = Ротација: { $flag }
rotate-turned = Окренуто { $turn } · ротација је сада { $flag }
rotate-failed = Није ротирано: { $reason }
undo-name-taken = Није промењено: друга датотека се већ зове { $name }
undo-back-on-clip = Поново на овом клипу (премештање је поништено). Ctrl+Z још једном поништава његову последњу измену.
redo-on-clip = Поново на овом клипу (премештање је враћено).
rotate-reason-in-use = датотека је само за читање или је у употреби
rotate-reason-format = овај формат нема ознаку ротације
rotate-reason-damaged = датотека је оштећена
rotate-reason-no-video = датотека нема видео траку
rotate-reason-matrix = видео има необичну матрицу приказа
batch-action-rotate = Ротирај снимке
batch-action-rotate-hint = Мења ознаку ротације сваке означене MP4 и MOV датотеке; слика се не кодира поново, а коментари, улазне и излазне тачке и маркери остају. Очекује се да Premiere Pro приликом увоза прикаже клип окренут. Други формати немају ознаку ротације и не успевају. Поновно покретање поново окреће датотеке; „Ресетуј“ уклања сваку ротацију, па и ону коју је снимио телефон, тако да портретни клип са телефона после тога лежи на боку.
batch-action-rotate-right = 90° удесно (у смеру казаљке на сату)
batch-action-rotate-left = 90° улево (супротно од казаљке на сату)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Ресетуј: без ротације (0°)
media-viewer-video-subtitle-list = Списак титлова
media-viewer-video-marker-list = Списак маркера
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 скачу између маркера; Shift+превлачење поравнава
media-viewer-video-tab-markers = Маркери
media-viewer-video-fullscreen = Цео екран
media-viewer-video-close-list = Затвори списак
media-viewer-video-subtitle-file-unreadable = Ова датотека није могла да се прочита
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Настављено од { $time } · Home: од почетка

## Markers list
markers-in-out = Улазне и излазне тачке

markers-empty = Још нема маркера
markers-add = Додај маркер
markers-ai-hint = AI маркер: замењује се када AI поново опише овај клип
markers-keep-color = Задржи боју
markers-done = Готово
markers-delete = Обриши маркер
markers-cannot-hold-hint = Premiere чита маркере из MP4 и MOV датотека.
markers-color-green = Зелена
markers-color-red = Црвена
markers-color-orange = Наранџаста
markers-color-yellow = Жута
markers-color-white = Бела
markers-color-blue = Плава
markers-color-cyan = Цијан
markers-color-lavender = Лаванда
markers-color-magenta = Магента
markers-color-other = Друга боја
markers-read-only = само за читање
markers-name-placeholder = Назив
markers-ai-describe = Опис помоћу AI: дај маркеру назив и додај шта се дешава
markers-ai-stop = Заустави опис
markers-ai-describe-unnamed = Опиши без назива: { $count }
markers-ai-describe-unnamed-hint = Даје назив и описује помоћу AI сваки маркер који нема назив, по неколико одједном. По један захтев за сваки; назив који у међувремену укуцате се задржава.
markers-ai-price-about = око { $dollars }
markers-ai-describe-unnamed-cost = По један захтев за сваки: { $each } по маркеру, { $total } за све.
markers-ai-confirm = { $count ->
    [one] Описати { $count } маркер? По један захтев за сваки: укупно { $total }.
    [few] Описати { $count } маркера? По један захтев за сваки: укупно { $total }.
   *[other] Описати { $count } маркера? По један захтев за сваки: укупно { $total }.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Описати { $count } маркер? По један плаћен захтев за сваки.
    [few] Описати { $count } маркера? По један плаћен захтев за сваки.
   *[other] Описати { $count } маркера? По један плаћен захтев за сваки.
}
markers-ai-confirm-yes = Опиши
markers-ai-confirm-no = Откажи
markers-ai-waiting = Чека на ред…
markers-ai-run = Описује се: { $describing } · чека: { $waiting }
markers-ai-run-last = Описује се: { $describing }
markers-ai-stop-all = Заустави све
markers-ai-describing = Описивање…
markers-ai-done = Маркер је описан
markers-ai-nothing-new = Маркер већ има овај опис
markers-no-marker-notice = Овде нема маркера
markers-ai-no-key = Нема API кључа за { $provider }: подесите га у подешавањима
markers-ai-failed = Није описано: { $reason }
markers-ai-failed-unknown = нешто је пошло наопако
markers-read-only-notice = Тај маркер је само за читање
markers-ai-gone = Маркера више нема: његов опис није додат
markers-ai-stopping-for-batch = Прво се заустављају описи маркера… Обично траје неколико секунди.
folder-not-saved-marker-ai = Није сачувано: опис маркера је још читао клип. Покушајте поново за тренутак.

## File workspace

file-workspace-search-placeholder = Пронађите тег — или само куцајте
file-workspace-search-clear = Обриши
file-workspace-comment-placeholder = Коментар...
file-workspace-comment-collapse = Назад на тегове
file-workspace-comment-expand = Прошири коментар

## Tag grid

tag-grid-star = Звездица: држи га при врху
tag-grid-unstar = Уклони звездицу
tag-grid-save = Додај у тегове фасцикле
tag-grid-delete = Обриши „{ $tag }“ из тегова фасцикле
tag-grid-create = Направи „{ $tag }“
tag-grid-no-file = Отворите клип да бисте га означили тегом
tag-grid-no-tags = Још нема тегова
tag-grid-no-tags-hint = Унесите назив и притисните Enter да направите први.
tag-grid-group-unsaved = Нису у теговима фасцикле
tag-grid-group-folder = Тегови фасцикле
tag-grid-more = још +{ $count }

## Order strip

sync-panel-locked = Промена редоследа испод мења фасциклу
sync-panel-unlocked = Промена редоследа испод мења само овај клип
sync-panel-unlock = Откључај
sync-panel-lock = Закључај
sync-panel-differs = Редослед се разликује од фасцикле
sync-panel-use-for-folder = Примени на фасциклу
sync-panel-sort-like-folder = Поређај као у фасцикли

## File name card

file-name-panel-no-tags = Овај клип нема тегова
file-name-panel-untag = Скини тег
file-name-panel-clear-in = Обриши улазну тачку
file-name-panel-clear-out = Обриши излазну тачку
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = Постави In и Out на оно што AI предлаже (заокружено на целе секунде)
file-name-panel-suggested-tags = AI тегови
file-name-panel-add-suggested-tag = Додај тег „{ $tag }“ (сигурност { $percent }%)
file-name-panel-add-suggested-tag-unsure = Додај тег „{ $tag }“
file-name-panel-add-all-suggested-tags = Додај све
file-name-panel-add-all-suggested-tags-tip = Додај сваки тег који AI предлаже
file-name-panel-tag-ideas = Идеје: { $ideas }
file-name-panel-tag-ideas-tip = Тегови које је AI приметио, а које ова фасцикла нема. Они нису додати.
file-name-panel-saved = Све измене су сачуване
file-name-panel-saved-to-recovery = Сачувано за опоравак, примењује се када напустите клип

## Updates (Settings)

updates-check = Провери ажурирања
updates-not-installed = Ажурирања раде у инсталираној верзији
updates-from-store = Ажурирања долазе из Microsoft Store-а
updates-from-releases = Нове верзије: преузмите их са странице издања
updates-open-releases = Отвори страницу издања
updates-checking = Провера…
updates-downloading-named = Преузимање { $version }… { $percent }%
updates-downloading = Преузимање… { $percent }%
updates-restarting = Поновно покретање…
updates-check-failed = Провера ажурирања није успела: { $reason }
updates-update-failed = Ажурирање није успело: { $reason }
updates-close-other-window = Прво затворите други frename прозор
updates-reason-github = GitHub није одговорио, покушајте касније
updates-reason-no-connection = нема везе
updates-reason-read-only = фасцикла је само за читање
updates-reason-not-installed = ажурирања раде у инсталираној верзији
updates-reason-download-stopped = преузимање се неочекивано зауставило
updates-version-available = Доступна је верзија { $version }
updates-up-to-date = frename је ажуран
updates-update-and-restart = Ажурирај и поново покрени
updates-wait-for-batch = Сачекајте да се групна радња заврши
updates-current-version = frename { $version }
updates-check-on-start = Провери ажурирања када се frename покрене
updates-notes-show = Белешке о издањима
updates-notes-hide = Сакриј белешке о издањима
updates-notes-all-releases = Сва издања на GitHub-у
updates-notes-history = Све верзије, најновије прво
updates-notes-update = Шта доноси верзија { $version }
updates-notes-whats-new = Шта је ново у верзији { $version }
updates-notes-loading = Учитавају се белешке о издању…
updates-notes-unavailable = Белешке о најновијој верзији нису могле да се учитају.
updates-notes-more = Прикажи још

recovery-restored = frename се неочекивано затворио. Враћен је ваш несачувани рад на: { $clip } ({ $what }).
recovery-kept = frename се неочекивано затворио. Ваш несачувани рад на: { $clip } ({ $what }) није примењен: { $why }. Читљива копија за поновно куцање налази се у: { $folder }.
recovery-unreadable = frename се неочекивано затворио, а датотека за опоравак није могла да се прочита. Чува се у: { $folder }.
recovery-why-gone = клипа више нема или је преименован
recovery-why-changed = клип је измењен након ваших измена
recovery-why-not-written = датотека са тим називом већ постоји, или је клип само за читање или у употреби
recovery-tags = { $count ->
    [one] { $count } тег
    [few] { $count } тега
   *[other] { $count } тегова
}
recovery-markers = { $count ->
    [one] { $count } маркер
    [few] { $count } маркера
   *[other] { $count } маркера
}
recovery-comment = коментар
recovery-in-out = улазне и излазне тачке
recovery-edits = измене
folder-frame-saved = Кадар је сачуван
folder-not-saved = Није сачувано: датотека са тим називом већ постоји, или је само за читање или у употреби
markers-deleted-notice = Маркер је обрисан
markers-not-saved-notice = Маркери нису сачувани: датотека је само за читање или је у употреби
markers-comment-not-saved-notice = Маркери нису сачувани: датотека коментара је само за читање или је у употреби
markers-copy = Копирај
markers-copy-tip = Копирај маркере као текст, по један ред
markers-export = Извези…
markers-export-tip = Сачувај маркере као текстуалну датотеку
markers-exported = Маркери су сачувани у: { $name }
markers-not-exported = Маркери нису сачувани: датотека није могла да се запише

batch-option-notes = Белешке за вештачку интелигенцију
batch-ai-notes-info = Ваше белешке о снимцима у овој фасцикли. Шаљу се уз сваки захтев ка вештачкој интелигенцији овде: описи, предлози тегова и описи маркера. Чувају се у датотеци .frename фасцикле.
batch-ai-notes-placeholder = нпр. Човек који се најчешће појављује је Евгеније. Означавај рекламне билборде. Предложи тег чај где год видиш чај.
batch-ai-notes-hint = Шаљу се добављачу { $provider } заједно са кадровима. Имена долазе само из ових белешки; frename не препознаје лица. Већ описани снимци се прескачу: означите Понови да бисте их описали поново са новим белешкама.
batch-ai-notes-too-long = Белешке имају { $chars } знакова, ограничење је { $max }. Скратите их да бисте покренули.
batch-ai-notes-too-long-reason = Белешке за вештачку интелигенцију су предугачке
batch-option-notes-once = Додатни ред за ово покретање
batch-ai-notes-once-placeholder = нпр. У овом клипу: жена у црвеном је Ана
file-workspace-ai-notes-on = Белешке су укључене
file-workspace-ai-notes-tip = Белешке за вештачку интелигенцију иду уз сваки захтев у овој фасцикли. Мењају се у Опиши помоћу вештачке интелигенције (групне радње): { $notes }
batch-ai-notes-no-subtitles = Изгледа да се белешке односе на оно што се говори, а { $videos } немају титлове: прво их направите (Прављење титлова) да би вештачка интелигенција могла да чита речи.
