# Текст інтерфейсу frename українською. Ті самі ідентифікатори, атрибути та $аргументи, що й в i18n/en.

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

## Settings window

settings-window-title = Налаштування
settings-page-interface = Інтерфейс
settings-page-saving = Збереження
settings-page-ai = Опис за допомогою ШІ
settings-close = Закрити
settings-apply-note = Зміни застосовуються одразу. Tab — перехід між параметрами, Ctrl+Tab — між сторінками.
settings-language = Мова
settings-language-system = Системна ({ $language })
settings-video = Відео
settings-video-autoplay = Автоматично відтворювати відео після відкриття
settings-subtitle-size = Розмір субтитрів
settings-subtitle-size-small = Малий
settings-subtitle-size-medium = Середній
settings-subtitle-size-large = Великий
settings-subtitle-size-extra-large = Дуже великий
settings-timecode = Відображення часу
settings-timecode-hint = Як записується час під відео, у списках позначок і субтитрів, на плашках In / Out, а також час позиції відтворення, який копіює Ctrl+Shift+C. Premiere Pro і DaVinci Resolve показують години:хвилини:секунди:кадри з частотою кадрів кліпу; 29,97 і 59,94 к/с — із пропуском кадрів (drop-frame). Кліп без сталої частоти кадрів лишається зі стандартним виглядом. Файли зберігають час у власних форматах.
settings-timecode-default = Стандартний (хв:с)
settings-timecode-premiere = Premiere Pro (HH:MM:SS:FF)
settings-timecode-resolve = DaVinci Resolve (HH:MM:SS:FF)
settings-tags = Кольори тегів
settings-tags-monochrome = Монохромні
settings-tags-monochrome-hint = Додані вами теги показано темнішим сірим, решту — світлішим, без інших кольорів.
settings-file-names = Імена файлів
settings-tags-space-after = Пробіл після кожного тега
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Файли зберігають імена, доки їх не перейменують або не збережуть.
settings-tags-space-add = Додати пробіл в імена файлів…
settings-tags-space-remove = Прибрати пробіл з імен файлів…
settings-comments = Коментарі
settings-comments-in-video = У відеофайлі
settings-comments-in-video-hint = XMP, стовпець «Опис» у Premiere Pro
settings-comments-text-file = У текстовому файлі поруч із відео
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Файли зберігають коментарі там, де вони є, доки їх не перенесуть.
settings-comments-move-into-videos = Перенести коментарі у відео…
settings-comments-move-into-text-files = Перенести коментарі в текстові файли…
settings-commented-tag = Тег для відео з коментарем
settings-commented-tag-name = Тег
settings-commented-tag-hint = Ставиться, коли ви пишете коментар до відео, наприклад { $tag }.IMG_0424.MOV, і знімається, коли ви його стираєте (описи ШІ не враховуються). Далі ви можете змінювати його самі.
settings-commented-tag-off = Відео з коментарем не отримують тега.
settings-in-out = Точки входу й виходу
settings-in-out-in-video = У відеофайлі
settings-in-out-in-video-hint = Маркер підкліпу в Premiere Pro
settings-in-out-comment = У коментарі, одним рядком
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Файли зберігають точки входу й виходу там, де вони є, доки їх не перенесуть.
settings-in-out-move-into-videos = Перенести точки входу й виходу у відео…
settings-in-out-move-into-comments = Перенести точки входу й виходу в коментарі…
settings-markers = Позначки та діапазони
settings-markers-in-video = У відеофайлі
settings-markers-in-video-hint = XMP, показуються на кліпі в Premiere Pro
settings-markers-comment = У коментарі, по одній на рядок
settings-markers-comment-example = 0:41–0:47 — Лев
settings-markers-note = Файли зберігають позначки там, де вони є, доки їх не перенесуть.
settings-markers-move-into-videos = Перенести позначки у відео…
settings-markers-copy-into-comment = Скопіювати позначки в коментарі…
settings-markers-hint = І окремі точки, і діапазони, зокрема моменти, які знаходить ШІ.
settings-updates = Оновлення
settings-version = Версія
settings-old-title = Налаштування зі старої версії frename
settings-old-hint = Перенесіть налаштування frename, який ви запускали з розпакованої zip-папки.
settings-old-scheduled = Налаштування буде імпортовано після перезапуску frename
settings-old-not-found = У { $folder } немає frename.exe з frename.db
settings-old-failed = Не вдалося імпортувати: { $reason }
settings-old-import-button = Імпортувати зі старої папки frename…
settings-ai = Опис за допомогою ШІ
settings-ai-key-label = Ключ API Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Ключ можна отримати на console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Видалити збережений ключ Anthropic?
settings-ai-used-by = Використовується для опису за допомогою ШІ в пакетному режимі та на позначках (✨, Ctrl+F2), коли вибрано модель Claude.
settings-ai-openai-key-label = Ключ API OpenAI
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = Ключ можна отримати на platform.openai.com/api-keys.
settings-ai-openai-key-remove-confirm = Видалити збережений ключ OpenAI?
settings-ai-openai-used-by = Використовується для опису за допомогою ШІ в пакетному режимі та на позначках (✨, Ctrl+F2), коли вибрано модель GPT.
settings-ai-gemini-key-label = Ключ API Gemini
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = Ключ можна отримати на aistudio.google.com/apikey.
settings-ai-gemini-key-remove-confirm = Видалити збережений ключ Gemini?
settings-ai-gemini-used-by = Використовується для опису за допомогою ШІ в пакетному режимі та на позначках (✨, Ctrl+F2), коли вибрано модель Gemini.
settings-ai-model-needs-key = { $model } — спершу додайте ключ { $provider }
settings-ai-model-label = Модель
settings-ai-language-label = Мова опису
settings-ai-hint = Найдешевші — Haiku, GPT-4.1 mini і Gemini Flash-Lite; Sonnet, Opus, GPT-4.1 і Gemini Flash помічають більше. Моделі потрібен ключ її власної компанії.
settings-ai-moments-label = Моменти
settings-ai-moments-important = Лише помітне
settings-ai-moments-full = Охопити весь кліп
settings-ai-moments-important-hint = Також підказує, де поставити In і Out (позначка ШІ біля IN і OUT)
settings-ai-moments-full-hint = Момент для кожної частини, навіть тихої; без підказки для In і Out
settings-ai-tag-suggestions = Також пропонувати теги з тегів папки
settings-ai-tag-suggestions-hint = У тому самому запиті ШІ каже, які з тегів папки пасують до кожного кліпу. Запропонований тег додається, лише коли ви на нього клацнете.
settings-subtitles = Субтитри
settings-subtitles-key-label = Ключ API Soniox
settings-subtitles-key-placeholder = Вставте ключ
settings-subtitles-key-get = Ключ можна отримати на console.soniox.com. Аудіо надсилається до Soniox.
settings-subtitles-key-remove-confirm = Видалити збережений ключ Soniox?
settings-subtitles-languages-label = Мови
settings-subtitles-languages-locked = Збережіть ключ, щоб обирати з усіх мов, які знає Soniox.
settings-subtitles-languages-loading = Отримуємо мови від Soniox…
settings-subtitles-languages-none = Нічого не вибрано: визначається автоматично.
settings-subtitles-languages-hint = Мови, якими говорять у відзнятому матеріалі, як підказка.
settings-subtitles-cue-length-label = Довжина репліки
settings-subtitles-cue-short = Коротка
settings-subtitles-cue-short-hint = Один рядок, до 8 с
settings-subtitles-cue-sentence = Одне речення
settings-subtitles-hint = Використовується дією «Створити субтитри» в пакетному режимі. Довжина репліки діє для нових субтитрів.
settings-key-store-windows = у Диспетчері облікових даних Windows
settings-key-store-macos = у в’язці ключів macOS
settings-key-store-other = у системній в’язці ключів
settings-key-unavailable = Не вдалося відкрити системну в’язку ключів
settings-key-unavailable-hint = Вона може бути заблокована або відсутня (наприклад, GNOME Keyring чи KWallet). Налаштування перевіряють це щоразу, коли відкриваються.
settings-key-remove-confirm-hint = Вам доведеться вставити його знову.
settings-key-remove-ask = Видалити…
settings-key-remove = Видалити ключ
settings-key-keep = Залишити
settings-key-replace = Замінити…
settings-key-hide = Сховати
settings-key-show = Показати
settings-key-save = Зберегти ключ
settings-key-get = Отримати ключ
settings-key-billing = Поповнити рахунок
settings-key-cancel = Скасувати
settings-key-saved-in = Збережено { $store } на цьому комп’ютері
settings-key-save-into = «Зберегти ключ» зберігає його { $store } на цьому комп’ютері.

## Batch mode

batch-checked-count = { $count ->
    [one] Позначено { $count } файл
    [few] Позначено { $count } файли
    [many] Позначено { $count } файлів
   *[other] Позначено { $count } файлів
}
batch-no-key = Немає ключа
batch-group-move = Перенесення між місцями
batch-group-fix = Виправлення імен і відео
batch-group-paid = Платні сервіси
batch-reason-none-checked = Немає позначених файлів
batch-reason-reading = Читаємо тривалість кліпів…
batch-check-all = Позначити все: { $count ->
    [one] { $count } файл
    [few] { $count } файли
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-job-running = виконується
batch-job-finished = завершено
batch-job-stopped = зупинено
batch-progress-files = { $finished } із { $total ->
    [one] { $total } файлу
    [few] { $total } файлів
    [many] { $total } файлів
   *[other] { $total } файлів
}
batch-time-left = залишилося близько { $time }
batch-time-estimating = оцінюємо час…
batch-time-spent = минуло { $time }
batch-runs-in-background = Ви можете працювати далі, поки це виконується
batch-clip-in-work = Зайнято: { $action }
batch-folder-held = Зайнято: { $action }
batch-indicator-hint = Клацніть, щоб відкрити панель і скасування
batch-action-prepare = Підготувати кліпи
batch-action-prepare-hint = Для кожного позначеного кліпу за один прохід: субтитри (Soniox), якщо їх немає, потім опис за допомогою ШІ: короткий зміст, моменти як позначки, коментар і пропозиції тегів. Одна ціна, один ліміт і один індикатор перебігу на все.
batch-prepare-step-off = вимкнено
batch-prepare-nothing = Для цих кліпів нічого робити
batch-prepare-report-cap = Витрачено { $spent } із ліміту { $cap }
batch-option-steps = Кроки
batch-prepare-step-subtitles = Спершу створити субтитри (Soniox)
batch-prepare-step-description = Описати кліпи за допомогою ШІ
batch-prepare-cap = Зупинитися, коли витрати сягнуть
batch-prepare-cap-none = Без ліміту (в доларах США)
batch-prepare-cap-invalid = Ліміт — це сума в доларах, наприклад 5
batch-prepare-left = Останній запуск зупинився, ще не оброблено: { $count ->
    [one] { $count } кліп
    [few] { $count } кліпи
    [many] { $count } кліпів
   *[other] { $count } кліпів
}
batch-prepare-check-left = Позначити їх
batch-prepare-estimating = З’ясовуємо, що зробить кожен крок…
batch-prepare-plan-subtitles = Субтитри
batch-prepare-plan-description = Опис
batch-prepare-plan-none = нічого робити
batch-prepare-plan-subtitles-line = { $videos ->
    [one] { $videos } відео
    [few] { $videos } відео
    [many] { $videos } відео
   *[other] { $videos } відео
} для розпізнавання, { $duration } аудіо
batch-prepare-plan-description-line = { $videos ->
    [one] { $videos } відео
    [few] { $videos } відео
    [many] { $videos } відео
   *[other] { $videos } відео
} для опису
batch-prepare-resume-note = Кожен крок пропускає те, що вже є, тож зупинений запуск продовжується повторним запуском: ні за що не платите двічі.
batch-prepare-run = Підготувати { $clips ->
    [one] { $clips } кліп
    [few] { $clips } кліпи
    [many] { $clips } кліпів
   *[other] { $clips } кліпів
} · близько { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = Схожий матеріал
batch-prepare-groups-hint = Кліпи, схожі на один і той самий кадр: дублікати, повторні експорти, камера, що не рухалася.
batch-prepare-group-clips = Кліпів: { $count }
batch-prepare-check-group = Позначити ці кліпи
batch-solo-queued = У черзі: { $action }
batch-suggest-in-job = Опис цього кліпу вже є в запущеному завданні, без пропозицій тегів. Спробуйте ще раз, коли воно завершиться
batch-suggest-no-tags = У цій папці ще немає тегів для пропозицій: спершу додайте кілька
batch-solo-gave-up = Не вдалося підготуватися
batch-solo-withdrawn = Скасовано: { $action }
batch-solo-in-work = { $action } виконується. Скасуйте це зі стрічки або зі сторінки завдання
batch-waiting-title = { $count ->
    [one] { $count } запуск очікує
    [few] { $count } запуски очікують
    [many] { $count } запусків очікують
   *[other] { $count } запусків очікують
}
batch-waiting-remove = Прибрати
batch-indicator-waiting = Очікує: { $count }
file-workspace-describe-clip = Описати цей кліп за допомогою ШІ
file-workspace-subtitles-clip = Створити субтитри для цього кліпу
file-workspace-describe-clip-queued = Опис за допомогою ШІ: очікує в черзі. Клацніть, щоб скасувати
file-workspace-describe-clip-running = Опис за допомогою ШІ: виконується
file-workspace-suggest-tags-clip = Запропонувати теги для цього кліпу за допомогою ШІ (Ctrl+F6)
file-workspace-suggest-tags-clip-queued = Пропозиція тегів: очікує в черзі. Клацніть, щоб скасувати
file-workspace-suggest-tags-clip-running = Пропозиція тегів: виконується
file-workspace-subtitles-clip-queued = Створення субтитрів: очікує в черзі. Клацніть, щоб скасувати
file-workspace-subtitles-clip-running = Створення субтитрів: виконується
batch-run-again = Запустити знову для { $count ->
    [one] { $count } файлу
    [few] { $count } файлів
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-result-done = Готово: { $total ->
    [one] { $total } файл
    [few] { $total } файли
    [many] { $total } файлів
   *[other] { $total } файлів
}
batch-result-done-detail = змінено: { $changed }, не потребували змін: { $unchanged }
batch-result-problems = Завершено з помилками: не виконано { $failed } із { $total ->
    [one] { $total } файлу
    [few] { $total } файлів
    [many] { $total } файлів
   *[other] { $total } файлів
}
batch-figure-unchanged = без змін
batch-figure-not-done = не виконано
batch-figure-not-reached = не дійшли
batch-files-not-done = Не виконано для файлів
batch-table-file = Файл
batch-table-why = Причина

batch-run-move-comments = Перенести { $count ->
    [one] { $count } коментар
    [few] { $count } коментарі
    [many] { $count } коментарів
   *[other] { $count } коментарів
}
batch-run-move-in-out = Перенести In/Out для { $count ->
    [one] { $count } файлу
    [few] { $count } файлів
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-run-convert = Конвертувати { $count ->
    [one] { $count } файл
    [few] { $count } файли
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-run-rotate = Повернути { $count ->
    [one] { $count } відео
    [few] { $count } відео
    [many] { $count } відео
   *[other] { $count } відео
}
batch-run-tag = Позначити тегом { $count ->
    [one] { $count } файл
    [few] { $count } файли
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-run-fix-tags = Упорядкувати теги в { $count ->
    [one] { $count } імені
    [few] { $count } іменах
    [many] { $count } іменах
   *[other] { $count } іменах
}
batch-run-rename = Перейменувати { $count ->
    [one] { $count } файл
    [few] { $count } файли
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-run-reload = Перечитати { $count ->
    [one] { $count } файл
    [few] { $count } файли
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-run-export-markers = Експортувати позначки з { $count ->
    [one] { $count } файлу
    [few] { $count } файлів
    [many] { $count } файлів
   *[other] { $count } файлів
}
batch-service-soniox = Soniox
batch-change-renames = Перейменовує файли
batch-change-videos = Записує у відео
batch-change-text-files = Записує текстові файли поруч із відео
batch-change-comments = Записує в коментарі
batch-change-subtitles = Записує файли субтитрів поруч із відео
batch-change-records = Змінює лише власні записи frename
batch-option-direction = Напрям
batch-option-language = Мова
batch-option-tag-suggestions = Пропозиції тегів
batch-option-described = Описані відео
batch-option-subtitled = Відео із субтитрами
batch-option-subtitles-write = Файли для запису
batch-option-spacing = Пробіли
batch-option-tag = Тег
batch-option-turn = Поворот
batch-plan-videos = Відео
batch-plan-length = Тривалість
batch-plan-cost = Вартість
batch-plan-time = Час
batch-reason-estimate = Чекаємо на оцінку
batch-reason-subtitles-no-format = Позначте принаймні один файл для запису
batch-set-key = Указати ключ…
batch-check-key = Перевірити ключ…
batch-subtitles-languages-auto = Визначається в кожному відео
batch-action-describe-ai-run-waiting = Описати відео
batch-action-fix-tags-order = Порядок: як у списку тегів, невідомі теги першими.
batch-action-markers-comment-hint-short = Перетворює рядки коментаря з часом на позначки або копіює позначки в коментар.
batch-action-markers-to-comment-hint = Позначки лишаються; повторний запуск нічого не дублює.
batch-action-rotate-hint-short = Повертає кліпи MP4 і MOV за прапорцем повороту, без перекодування.
batch-action-tag-commented-off = Тегування відео з коментарем вимкнено
batch-action-tag-commented-choose = Вибрати тег…

batch-title = Пакетні дії
batch-done-label-changed = змінено
batch-done-label-subtitled = із субтитрами
batch-ai-at-least = щонайменше
batch-cancel = Скасувати
batch-stopping = Зупиняємо…
batch-stop-over-budget = Зупинено: наступний кліп міг би вивести запуск за ліміт { $cap } (витрачено { $spent }). Збільште ліміт, щоб виконати решту.
batch-stopped = Зупинено після { $finished } із { $total ->
    [one] { $total } файлу
    [few] { $total } файлів
    [many] { $total } файлів
   *[other] { $total } файлів
}.
batch-close = Закрити
batch-failed-subtitles = Без субтитрів:
batch-written-subtitles = Записані файли:
batch-table-written = Записано
batch-merged-markers = Об’єднані дублікати позначок:
batch-table-merged = Об’єднано
batch-markers-merged = { $count ->
    [one] Об’єднано { $count } дублікат позначки
    [few] Об’єднано { $count } дублікати позначок
    [many] Об’єднано { $count } дублікатів позначок
   *[other] Об’єднано { $count } дублікатів позначок
}
batch-add-credit = Поповнити рахунок
batch-open-log = Відкрити журнал

batch-action-move-comments = Перенести коментарі
batch-action-move-comments-hint = Переносить коментар кожного позначеного файлу в обране місце разом із точками входу й виходу, які в ньому є. Теги та точки входу й виходу, збережені у відео, лишаються на місці.
batch-action-move-comments-into-videos = З текстових файлів у відео (XMP)
batch-action-move-comments-into-text-files = З відео (XMP) у текстові файли

batch-action-move-in-out = Точки входу й виходу: коментар ⇄ відео (XMP)
batch-action-move-in-out-hint = Переносить точки входу й виходу кожного позначеного файлу в обране місце. Решта коментаря лишається на місці.
batch-action-move-in-out-into-videos = З коментарів у відео (маркер Adobe XMP)
batch-action-move-in-out-into-comments = З відео (маркер XMP) у коментарі

batch-action-markers-comment = Позначки ⇄ коментар
batch-action-markers-comment-to-markers = Рядки коментаря з часом — у позначки
batch-action-markers-to-comment = Позначки — у коментар (копія: позначки лишаються)
batch-action-markers-comment-hint = Рядок на кшталт «03:24 — Дубль 3 — гарне світло» — це позначка на 3:24 з назвою «Дубль 3» і коментарем «гарне світло»; «0:41-0:47 — Лев» — позначка від 0:41 до 0:47. Назва й коментар розділяються за першим « — » або « -- », а не за звичайним « - ». Моменти з опису ШІ («0:00–0:14 Вулиця.») стають білими позначками й лишаються в описі. Повторний запуск у будь-якому напрямі нічого не дублює.

batch-action-tag-commented = Позначити тегом відео з коментарем
batch-action-tag-commented-hint = Додає тег «{ $tag }» кожному позначеному відео з вашим коментарем (описи ШІ не враховуються) і знімає його з тих, де коментаря немає. Файли, у яких змінюється тег, перейменовуються.
batch-action-tag-commented-hint-off = Додає тег для відео з коментарем кожному позначеному відео з вашим коментарем і знімає його з тих, де коментаря немає. Тег вимкнено в налаштуваннях.
batch-action-fix-tags = Виправити теги за пріоритетом
batch-action-fix-tags-hint = Розташовує теги в імені кожного позначеного файлу в порядку панелі тегів: чим вище тег на панелі, тим раніше він в імені. Теги, яких папка ще не знає, йдуть першими, як на панелі тегів. Файли, у яких змінюється порядок, перейменовуються.

batch-action-respace-tags = Застосувати пробіли після тегів
batch-action-respace-tags-hint-space = Перейменовує кожен позначений файл так, щоб після кожного тега був пробіл, як задано в налаштуваннях: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Перейменовує кожен позначений файл так, щоб після тегів не було пробілу, як задано в налаштуваннях: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Пробіли: пробіл після кожного тега
batch-action-respace-tags-status-no-space = Пробіли: без пробілу після тегів
batch-action-export-markers = Експортувати позначки в текст
batch-action-export-markers-hint = Записує позначки кожного позначеного файлу в текстовий файл поруч із ним, clip.markers.txt: один рядок на позначку в порядку часу, наприклад «0:41–0:47 [red] — Лев іде — іде зліва направо». Колір, відмінний від зеленого, у квадратних дужках; коментар іде після назви. Дія «Позначки ⇄ коментар» читає такі рядки назад як позначки. Файл без позначок не отримує текстового файлу, а текстовий файл від попереднього запуску замінюється.
batch-export-markers-cannot-hold = Цей файл не може містити позначки
batch-action-reload-files = Скинути кеш і перечитати
batch-action-reload-files-hint = Знову читає коментар і точки входу й виходу кожного позначеного файлу з самого файлу й замінює те, що папка пам’ятала про нього. Скористайтеся цим, якщо файли змінила інша програма. Файли, значень яких бракувало або які застаріли, вважаються зміненими.

batch-action-describe-ai = Описати за допомогою ШІ
batch-action-describe-ai-run = Описати { $videos } · близько { $dollars }
batch-action-describe-ai-estimating = Оцінюємо… { $known } / { $total }
batch-action-describe-ai-none = Немає відео для опису.
batch-action-describe-ai-hint = { batch-runs-in-background }. Скасування зберігає вже описані відео; повторний запуск їх пропускає.
batch-action-describe-ai-no-subtitles = Без субтитрів (описується лише зображення): { $videos }.
batch-action-describe-ai-redo = Повторити для відео, які вже мають опис ШІ
batch-action-describe-ai-hint-panel = Описує, що і коли відбувається в кожному позначеному відео: короткий зміст і сегменти з часом потрапляють у коментар як опис ШІ; ваш власний текст зберігається. Кадри й субтитри надсилаються до { $provider } — компанії обраної моделі.
batch-ai-change = Змінити
batch-ai-tag-suggestions-off = Вимкнено
batch-ai-tag-suggestions-no-tags = Увімкнено, але в цій папці ще немає тегів
batch-ai-tag-suggestions-on = { $tags ->
    [one] З { $tags } тега папки
    [few] З { $tags } тегів папки
    [many] З { $tags } тегів папки
   *[other] З { $tags } тегів папки
}
batch-ai-key-missing = Укажіть ключ API { $provider } у налаштуваннях
batch-ai-key-unavailable = Не вдалося відкрити системну в’язку ключів: вона може бути заблокована або відсутня (наприклад, GNOME Keyring чи KWallet).
## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Як у субтитрах
ai-language-english = Англійська
ai-language-russian = Російська
ai-language-ukrainian = Українська
ai-language-german = Німецька
ai-language-spanish = Іспанська
ai-language-french = Французька
batch-videos-count = { $n ->
    [one] { $n } відео
    [few] { $n } відео
    [many] { $n } відео
   *[other] { $n } відео
}
batch-minutes = { $n } хв
batch-ai-minutes-under = < 1 хв
batch-ai-duration-under-minute = менше хвилини
batch-ai-duration-hours = { $h } год
batch-ai-duration-hours-minutes = { $h } год { $m } хв
batch-ai-dollars-under = менше $0.01
batch-ai-skip-described = уже описано
batch-ai-skip-too-long = понад 30 хв
batch-ai-skip-unreadable = не читається
batch-ai-skipped = Пропущено: { $parts }.
batch-ai-progress-frame = кадр { $done } із { $total }
batch-ai-progress-waiting = чекаємо на { $provider }
batch-ai-progress-saving = зберігаємо
batch-ai-progress-retry-rate-limit = повтор через { $seconds } с (ліміт запитів)
batch-ai-progress-retry-temporary = повтор через { $seconds } с (тимчасовий збій)
batch-ai-stop-no-key = Зупинено: немає ключа API { $provider }. Укажіть його в налаштуваннях.
batch-ai-fail-no-key = Немає ключа API
batch-ai-fail-unreadable = Не вдалося прочитати відео
batch-ai-fail-too-long = Кліп задовгий для ШІ (понад 30 хв)
batch-ai-fail-not-saved = Не вдалося зберегти опис
batch-ai-stop-offline = Зупинено: немає зв’язку з { $provider }. Запустіть знову, щоб описати решту.

batch-action-generate-subtitles = Створити субтитри
batch-action-generate-subtitles-install-ffmpeg = щоб читати .mkv, .m2ts, .avi …, установіть ffmpeg з ffmpeg.org, додайте його в PATH і перезапустіть frename
batch-action-generate-subtitles-replace = Замінити наявні субтитри
batch-action-generate-subtitles-replace-hint = Розпізнає знову; вартість — як показано.
batch-action-generate-subtitles-srt = Субтитри SRT (clip.srt)
batch-action-generate-subtitles-premiere = Транскрипт Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = У Premiere Pro: панель «Текст» → «Транскрипт» → Import Static Transcript.
batch-action-generate-subtitles-privacy = Аудіо цих відео надсилається до Soniox і потім там видаляється.
batch-action-generate-subtitles-duration-hint = Займає кілька хвилин на годину аудіо. { batch-runs-in-background }. Закриття frename зупиняє запуск; готові субтитри зберігаються.
batch-action-generate-subtitles-hint = Розпізнає мовлення в кожному позначеному відео за допомогою Soniox і зберігає результат поруч: субтитри (clip.srt), де їх показує frename, та/або транскрипт для Premiere Pro.
batch-subtitles-transcribe = Розпізнати
batch-subtitles-transcribe-count = Розпізнати { $videos }
batch-subtitles-build-free = Створити { $count } (безкоштовно)
batch-subtitles-nothing = Нічого розпізнавати
batch-subtitles-estimating = Оцінюємо…
batch-subtitles-key-missing = Укажіть ключ API Soniox у налаштуваннях
batch-subtitles-key-rejected = Soniox відхилив ключ
batch-subtitles-cost-unknown = вартість невідома
batch-subtitles-typical-price = { $usd } (типова ціна)
batch-subtitles-unknown-length = + { $n } невідомої тривалості
batch-subtitles-plan-line = { $videos } для розпізнавання, { $duration } аудіо{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } уже має субтитри
    [few] { $n } уже мають субтитри
    [many] { $n } уже мають субтитри
   *[other] { $n } уже мають субтитри
}
batch-subtitles-rebuilt-free = { $n } відновлено безкоштовно зі збереженого транскрипту
batch-subtitles-no-speech-before = { $n } минулого разу без мовлення
batch-subtitles-no-audio = { $n ->
    [one] { $n } без аудіо
    [few] { $n } без аудіо
    [many] { $n } без аудіо
   *[other] { $n } без аудіо
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } має те саме ім’я субтитрів, що й інше відео
    [few] { $n } мають те саме ім’я субтитрів, що й інше відео
    [many] { $n } мають те саме ім’я субтитрів, що й інше відео
   *[other] { $n } мають те саме ім’я субтитрів, що й інше відео
}
batch-subtitles-unreadable = { $n } не вдалося прочитати ({ $how_to })
batch-subtitles-shared-name-reason = ім’я субтитрів збігається з іншим відео
batch-subtitles-unsupported-reason = формат аудіо не підтримується
batch-subtitles-progress-transcribing = розпізнаємо…
batch-subtitles-progress-extracting = витягуємо аудіо { $percent } %
batch-subtitles-progress-extracting-unknown = витягуємо аудіо
batch-subtitles-progress-uploading = завантажуємо { $percent } %
batch-subtitles-progress-uploading-unknown = завантажуємо
batch-subtitles-progress-transcribing-left = розпізнаємо… залишилося близько { $time }
batch-subtitles-progress-saving = зберігаємо
batch-subtitles-reason-already = субтитри вже були
batch-subtitles-reason-no-audio = немає аудіо
batch-subtitles-reason-no-speech = немає мовлення
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox недоступний
batch-subtitles-stop-key-rejected = Зупинено: Soniox відхилив ключ. Перевірте його в Налаштування → Субтитри.
batch-subtitles-stop-balance-empty = Зупинено: баланс Soniox порожній. Поповніть його на console.soniox.com.
batch-subtitles-stop-budget-used = Зупинено: місячний бюджет Soniox вичерпано. Збільште його на console.soniox.com.
batch-subtitles-stop-other = Зупинено: Soniox відмовився продовжувати ({ $error_type }).
batch-subtitles-stop-unreachable = Зупинено: Soniox недоступний. Перевірте підключення до інтернету.
batch-subtitles-report-spend = Soniox: щонайменше { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } завантаження не вдалося видалити
    [few] { $n } завантаження не вдалося видалити
    [many] { $n } завантажень не вдалося видалити
   *[other] { $n } завантажень не вдалося видалити
} в Soniox (див. журнал)
batch-subtitles-languages-need-key = Збережіть ключ Soniox, щоб обирати з усіх його мов.
batch-subtitles-languages-rejected = Soniox відхилив ключ.
batch-subtitles-languages-failed = Не вдалося отримати мови від Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } субтитр
    [few] { $n } субтитри
    [many] { $n } субтитрів
   *[other] { $n } субтитрів
}
batch-subtitles-duration-seconds = { $s } с
batch-subtitles-usd-under = менше $0.01
batch-subtitles-usd-about = близько ${ $amount }

## File list
folder-has-subtitles = Є субтитри
folder-closed-by-job = Закрито, поки її змінює пакетна дія
folder-filter-tip = Показати лише…

folder-search-placeholder = Знайти файл або коментар
folder-search-comments-loading = Шукаємо в коментарях… залишилося: { $n }
folder-search-clear = Очистити
folder-all = Усі
folder-invert = Інвертувати
folder-checked = Позначено: { $count }
folder-outcome-changed = Змінено
folder-outcome-unchanged = Нічого змінювати
folder-outcome-failed = Помилка, див. журнал
folder-rename-error-empty = Ім’я порожнє
folder-rename-error-bad-character = Заборонено: \ / : * ? " < > |
folder-rename-error-trailing = Не може закінчуватися крапкою або пробілом
folder-rename-error-exists = Файл із таким іменем уже є
folder-markers-not-saved = Позначки не збережено: файл лише для читання або використовується (закрийте його в Premiere, потім відкрийте файл і залиште його знову)
drag-out-not-saved = Не перетягнуто: не вдається зберегти файл (лише для читання або відкритий у Premiere)
folder-opening = Відкриваємо папку…
folder-empty-title = У цій папці немає відео
folder-empty-line = frename показує файли MP4, MOV, MKV та інші відеофайли.
folder-open-another = Відкрити іншу папку…
folder-no-match = Немає відповідних файлів
folder-show-all = Показати все
folder-locked = Вибір файлів заблоковано, поки виконується: { $action }
folder-checked-hidden = Позначено: { $count } · приховано: { $hidden }
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } позначений файл приховано пошуком або фільтром; дія виконається і для нього
    [few] { $hidden } позначені файли приховано пошуком або фільтром; дія виконається і для них
    [many] { $hidden } позначених файлів приховано пошуком або фільтром; дія виконається і для них
   *[other] { $hidden } позначених файлів приховано пошуком або фільтром; дія виконається і для них
}
folder-outcome-working = Виконується
folder-outcome-not-reached = Не опрацьовано
folder-window-empty-title = Відкрийте папку з кліпами
folder-window-empty-line = Або перетягніть папку у вікно. Клацніть правою кнопкою на кнопці, щоб відкрити один файл.
folder-window-open = Відкрити папку…

## Recent folders (the arrow next to the open button, and the empty screen)

recent-folders-tip = Нещодавні папки
recent-folders-title = Нещодавні папки
recent-folders-none = Ще не відкрито жодної папки
recent-folders-not-found = Не знайдено
recent-folders-remove-tip = Прибрати зі списку
recent-folders-clear = Очистити список
recent-folders-remove = Прибрати
recent-folders-keep = Залишити
recent-folders-missing-line = Не знайдено. Прибрати?
recent-folders-just-now = Щойно
recent-folders-minutes = { $count } хв тому
recent-folders-hours = { $count } год тому
recent-folders-days = { $count ->
    [one] { $count } день тому
    [few] { $count } дні тому
    [many] { $count } днів тому
   *[other] { $count } днів тому
}
recent-folders-weeks = { $count ->
    [one] { $count } тиждень тому
    [few] { $count } тижні тому
    [many] { $count } тижнів тому
   *[other] { $count } тижнів тому
}
recent-folders-months = { $count ->
    [one] { $count } місяць тому
    [few] { $count } місяці тому
    [many] { $count } місяців тому
   *[other] { $count } місяців тому
}
recent-folders-years = { $count ->
    [one] { $count } рік тому
    [few] { $count } роки тому
    [many] { $count } років тому
   *[other] { $count } років тому
}

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Показати в Провіднику
file-menu-show-in-file-manager = Показати у файловому менеджері
file-menu-copy-path = Копіювати повний шлях
file-menu-copy-name = Копіювати ім’я файлу
file-menu-copied = Скопійовано
file-menu-not-copied = Не скопійовано: буфер обміну недоступний
file-menu-not-shown = Не вдалося відкрити файловий менеджер

## Controls bar under the file list

folder-controls-filter-untagged = Без тегів
folder-controls-filter-subtitles = Субтитри
folder-controls-filter-comments = Коментарі
folder-controls-filter-markers = Позначки
folder-controls-scroll = Показати відкритий файл у списку
folder-controls-open = Відкрити папку
folder-controls-batch = Пакетні дії над позначеними файлами
folder-controls-batch-back = Назад до відкритого файлу
folder-controls-update-available = Доступне оновлення: { $version }
folder-controls-previous = Попередній файл
folder-controls-next = Наступний файл
folder-controls-more = Ще
folder-controls-open-file = Права кнопка: відкрити один файл

## Video

video-controls-back = Назад на 10 с
video-controls-play = Відтворити
video-controls-pause = Пауза
video-controls-forward = Уперед на 10 с
video-controls-frame-back = Кадр назад
video-controls-frame-forward = Кадр уперед
video-controls-set-in = Поставити точку входу
video-controls-set-out = Поставити точку виходу
video-controls-screenshot = Зберегти цей кадр
video-controls-add-marker = Додати позначку
video-controls-add-marker-hold = Утримуйте для діапазону; натисніть знову, щоб дати назву
video-controls-cannot-hold-markers = Цей файл не може містити позначки
video-controls-add-a-name = Додати назву
video-controls-rotate-left = Повернути ліворуч
video-controls-rotate-right = Повернути праворуч
video-controls-more = Ще
media-viewer-loading-slow = Чекаємо на файл… (хмарний файл може завантажуватися довго)
media-viewer-cannot-play = Цей кліп не вдається відтворити
media-viewer-no-picture = У цьому файлі немає відеозображення
video-timecode-copied = Таймкод скопійовано: { $time }
video-timecode-unreadable = «{ $text }» — не час у цьому кліпі
video-controls-volume-scroll = Гучність — прокрутіть, щоб змінити
rotate-cannot = Не вдається повернути: { $reason }
rotate-reason-missing = файлу вже немає
rotate-flag-right = 90° праворуч
rotate-flag-left = 90° ліворуч
rotate-flag-half = 180°
rotate-flag-none = немає
rotate-now = Поворот: { $flag }
rotate-turned = Повернуто: { $turn } · тепер поворот: { $flag }
rotate-failed = Не повернуто: { $reason }
undo-name-taken = Не змінено: інший файл уже має ім’я { $name }
undo-back-on-clip = Знову на цьому кліпі (скасовано перехід). Ще раз Ctrl+Z скасує його останню зміну.
redo-on-clip = Знову на цьому кліпі (повторено перехід).
rotate-reason-in-use = файл лише для читання або використовується
rotate-reason-format = цей формат не має прапорця повороту
rotate-reason-damaged = файл пошкоджено
rotate-reason-no-video = у файлі немає відеодоріжки
rotate-reason-matrix = відео має незвичну матрицю відображення
batch-action-rotate = Повернути відео
batch-action-rotate-hint = Змінює прапорець повороту кожного позначеного файлу MP4 і MOV; зображення не перекодовується, а коментарі, точки входу й виходу та позначки лишаються. Очікується, що Premiere Pro покаже кліп повернутим під час імпорту. Інші формати не мають прапорця повороту, і для них дія завершується помилкою. Повторний запуск повертає файли ще раз; «Скинути» прибирає будь-який поворот, зокрема записаний телефоном, тож портретний кліп із телефона після цього відтворюватиметься на боці.
batch-action-rotate-right = 90° праворуч (за годинниковою стрілкою)
batch-action-rotate-left = 90° ліворуч (проти годинникової стрілки)
batch-action-rotate-half = 180°
batch-action-rotate-reset = Скинути: без повороту (0°)
media-viewer-video-subtitle-list = Список субтитрів
media-viewer-video-marker-list = Список позначок
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 — перехід між позначками; Shift+перетягування — прилипання
media-viewer-video-tab-markers = Позначки
media-viewer-video-fullscreen = На весь екран
media-viewer-video-close-list = Закрити список
media-viewer-video-subtitle-file-unreadable = Не вдалося прочитати цей файл
# Shown over the picture when a clip opens where playback stopped last time; Home or a click goes to the start.
media-viewer-video-resumed = Продовжено з { $time } · Home: з початку

## Markers list
markers-in-out = Точки входу й виходу

markers-empty = Позначок ще немає
markers-add = Додати позначку
markers-ai-hint = Позначка ШІ: буде замінена, коли ШІ знову опише цей кліп
markers-keep-color = Залишити колір
markers-done = Готово
markers-delete = Видалити позначку
markers-cannot-hold-hint = Premiere читає позначки з файлів MP4 і MOV.
markers-color-green = Зелений
markers-color-red = Червоний
markers-color-orange = Помаранчевий
markers-color-yellow = Жовтий
markers-color-white = Білий
markers-color-blue = Синій
markers-color-cyan = Бірюзовий
markers-color-lavender = Лавандовий
markers-color-magenta = Пурпуровий
markers-color-other = Інший колір
markers-read-only = лише для читання
markers-name-placeholder = Назва
markers-ai-describe = Описати за допомогою ШІ: дати назву позначці й додати, що відбувається
markers-ai-stop = Зупинити опис
markers-ai-describe-unnamed = Описати без назви: { $count }
markers-ai-describe-unnamed-hint = Дати назву й описати за допомогою ШІ кожну позначку без назви, по кілька за раз. По одному запиту на кожну; назва, яку ви введете тим часом, зберігається.
markers-ai-price-about = близько { $dollars }
markers-ai-describe-unnamed-cost = По одному запиту на кожну: { $each } за позначку, { $total } за всі.
markers-ai-confirm = { $count ->
    [one] Описати { $count } позначку? Запит на кожну: усього { $total }.
    [few] Описати { $count } позначки? По запиту на кожну: усього { $total }.
    [many] Описати { $count } позначок? По запиту на кожну: усього { $total }.
   *[other] Описати { $count } позначок? По запиту на кожну: усього { $total }.
}
markers-ai-confirm-unpriced = { $count ->
    [one] Описати { $count } позначку? Платний запит на кожну.
    [few] Описати { $count } позначки? По одному платному запиту на кожну.
    [many] Описати { $count } позначок? По одному платному запиту на кожну.
   *[other] Описати { $count } позначок? По одному платному запиту на кожну.
}
markers-ai-confirm-yes = Описати
markers-ai-confirm-no = Скасувати
markers-ai-waiting = Очікує в черзі…
markers-ai-run = Описуємо: { $describing } · очікують: { $waiting }
markers-ai-run-last = Описуємо: { $describing }
markers-ai-stop-all = Зупинити все
markers-ai-describing = Описуємо…
markers-ai-done = Позначку описано
markers-ai-nothing-new = Позначка вже має цей опис
markers-no-marker-notice = Тут немає позначки
markers-ai-no-key = Немає ключа API { $provider }: укажіть його в налаштуваннях
markers-ai-failed = Не описано: { $reason }
markers-ai-failed-unknown = щось пішло не так
markers-read-only-notice = Ця позначка лише для читання
markers-ai-gone = Позначки вже немає: її опис не додано
markers-ai-stopping-for-batch = Спершу зупиняємо опис позначок… Зазвичай це кілька секунд.
folder-not-saved-marker-ai = Не збережено: опис позначки ще читав кліп. Спробуйте за мить.

## File workspace

file-workspace-search-placeholder = Знайдіть тег — або просто пишіть
file-workspace-search-clear = Очистити
file-workspace-comment-placeholder = Коментар...
file-workspace-comment-collapse = Назад до тегів
file-workspace-comment-expand = Розгорнути коментар

## Tag grid

tag-grid-star = Зірочка: тримати вгорі
tag-grid-unstar = Зняти зірочку
tag-grid-save = Додати до тегів папки
tag-grid-delete = Видалити «{ $tag }» з тегів папки
tag-grid-create = Створити «{ $tag }»
tag-grid-no-file = Відкрийте кліп, щоб додати теги
tag-grid-no-tags = Тегів ще немає
tag-grid-no-tags-hint = Введіть назву й натисніть Enter, щоб створити перший.
tag-grid-group-unsaved = Не в тегах папки
tag-grid-group-folder = Теги папки
tag-grid-more = +{ $count } ще

## Order strip

sync-panel-locked = Порядок нижче — для всієї папки
sync-panel-unlocked = Порядок нижче — лише для цього кліпу
sync-panel-unlock = Розблокувати
sync-panel-lock = Заблокувати
sync-panel-differs = Порядок відрізняється від порядку в папці
sync-panel-use-for-folder = Застосувати до папки
sync-panel-sort-like-folder = Упорядкувати, як у папці

## File name card

file-name-panel-no-tags = У цього кліпу немає тегів
file-name-panel-untag = Зняти тег
file-name-panel-clear-in = Прибрати точку входу
file-name-panel-clear-out = Прибрати точку виходу
file-name-panel-suggested-in-out = ШІ
file-name-panel-apply-suggested-in-out = Поставити In і Out за підказкою ШІ (округлено до цілих секунд)
file-name-panel-suggested-tags = Теги ШІ
file-name-panel-add-suggested-tag = Додати тег «{ $tag }» (упевненість { $percent } %)
file-name-panel-add-suggested-tag-unsure = Додати тег «{ $tag }»
file-name-panel-add-all-suggested-tags = Додати всі
file-name-panel-add-all-suggested-tags-tip = Додати кожен тег, який пропонує ШІ
file-name-panel-tag-ideas = Ідеї: { $ideas }
file-name-panel-tag-ideas-tip = Теги, які помітив ШІ, але яких немає в цій папці. Їх не додано.
file-name-panel-saved = Усі зміни збережено
file-name-panel-saved-to-recovery = Збережено для відновлення, буде застосовано, коли ви залишите кліп

## Updates (Settings)

updates-check = Перевірити оновлення
updates-not-installed = Оновлення працюють у встановленій версії
updates-from-store = Оновлення надходять із Microsoft Store
updates-from-releases = Нові версії: завантажуйте їх зі сторінки випусків
updates-open-releases = Відкрити сторінку випусків
updates-checking = Перевіряємо…
updates-downloading-named = Завантажуємо { $version }… { $percent } %
updates-downloading = Завантажуємо… { $percent } %
updates-restarting = Перезапускаємо…
updates-check-failed = Не вдалося перевірити оновлення: { $reason }
updates-update-failed = Не вдалося оновити: { $reason }
updates-close-other-window = Спершу закрийте інше вікно frename
updates-reason-github = GitHub не відповів, спробуйте пізніше
updates-reason-no-connection = немає з’єднання
updates-reason-read-only = папка лише для читання
updates-reason-not-installed = оновлення працюють у встановленій версії
updates-reason-download-stopped = завантаження несподівано зупинилося
updates-version-available = Доступна версія { $version }
updates-up-to-date = frename оновлено до найновішої версії
updates-update-and-restart = Оновити й перезапустити
updates-wait-for-batch = Зачекайте, поки завершиться пакетна дія
updates-current-version = frename { $version }
updates-check-on-start = Перевіряти оновлення під час запуску frename

recovery-restored = frename несподівано закрився. Вашу незбережену роботу над { $clip } ({ $what }) відновлено.
recovery-kept = frename несподівано закрився. Вашу незбережену роботу над { $clip } ({ $what }) не застосовано: { $why }. Читабельну копію, щоб ввести наново, збережено в { $folder }.
recovery-unreadable = frename несподівано закрився, і файл відновлення не вдалося прочитати. Його збережено в { $folder }.
recovery-why-gone = кліпу немає або його перейменовано
recovery-why-changed = кліп змінився після ваших правок
recovery-why-not-written = файл із таким іменем уже є, або кліп лише для читання чи використовується
recovery-tags = { $count ->
    [one] { $count } тег
    [few] { $count } теги
    [many] { $count } тегів
   *[other] { $count } тегів
}
recovery-markers = { $count ->
    [one] { $count } позначка
    [few] { $count } позначки
    [many] { $count } позначок
   *[other] { $count } позначок
}
recovery-comment = коментар
recovery-in-out = точки входу й виходу
recovery-edits = правки
folder-frame-saved = Кадр збережено
folder-not-saved = Не збережено: файл із таким іменем уже є, або він лише для читання чи використовується
markers-deleted-notice = Позначку видалено
markers-not-saved-notice = Позначки не збережено: файл лише для читання або використовується
markers-comment-not-saved-notice = Позначки не збережено: файл коментаря лише для читання або використовується
markers-copy = Копіювати
markers-copy-tip = Скопіювати позначки як текст, по одній на рядок
markers-export = Експортувати…
markers-export-tip = Зберегти позначки в текстовий файл
markers-exported = Позначки збережено в { $name }
markers-not-exported = Позначки не збережено: не вдалося записати файл
