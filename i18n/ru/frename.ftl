# frename UI text in Russian. Same ids, attributes and $arguments as i18n/en/frename.ftl, in the
# same order (checked by the tests in src/i18n.rs).
#
# Glossary (docs/design/localization.md): every string uses these terms.
#   tag / tags                        тег / теги (never «метка»: that is Premiere's Label)
#   tag checked / unchecked on video  тег ставится / снимается
#   in / out points                   точки входа / выхода
#   comment                           комментарий
#   subtitles                         субтитры
#   screenshot                        скриншот
#   marker (Premiere)                 маркер
#   range (a marker with a duration)  диапазон
#   subclip (Premiere)                subclip / «подклип»
#   Description column (Premiere)     колонка Description / «Описание»
#   checked (files)                   отмечено
#   All / Invert (check boxes)        Все / Обратить
#   video / clip                      видео
#   tag panel                         панель тегов
#   file name                         имя файла
#   a tag name inside a sentence      in «…» quotes: «Commented»
#   untagged (filter)                 без тегов
#   changed / unchanged / failed      изменено / без изменений / с ошибкой (colon form)
#   Cancel / Stopping…                Отмена / Остановка…
#   log, "see the log"                журнал, «подробности в журнале»
#   System (language list)            Как в системе (…)
#   batch action                      пакетное действие
#   file list / folder                список файлов / папка
#   settings                          настройки
#   update (a newer frename)          обновление
#   AI description (Describe with AI) описание от AI
#   subtitles generation (Soniox)     распознавание речи / субтитры
#
# Plurals: numbers are integers, so the categories are one / few / many, many as the default.

## Language names: the same in every file, each written in its own language.

language-name-en = English
language-name-ru = Русский

## Settings window

settings-window-title = Настройки
settings-language = Язык
settings-language-system = Как в системе ({ $language })
settings-video = Видео
settings-video-autoplay = Сразу воспроизводить открытое видео
settings-tags = Теги
settings-tags-monochrome = Одноцветные теги
settings-tags-space-after = Пробел после каждого тега в имени файла (Food. Goat. clip.mp4)
settings-tags-space-note = Имена файлов останутся прежними, пока файлы не переименуют или не сохранят.
settings-tags-space-add = Добавить пробел в имена имеющихся файлов…
settings-tags-space-remove = Убрать пробел из имён имеющихся файлов…
settings-comments = Комментарии
settings-comments-in-video = Внутри видеофайла (XMP; в Premiere Pro — колонка Description / «Описание»)
settings-comments-text-file = В файле .comment.txt рядом с видео
settings-comments-note = Комментарии остаются там, где были, пока их не перенесут.
settings-comments-move-into-videos = Перенести имеющиеся комментарии из текстовых файлов в видео…
settings-comments-move-into-text-files = Перенести имеющиеся комментарии из видео в текстовые файлы…
settings-commented-tag = Ставить тег видео с комментарием
settings-commented-tag-hint = Тег ставится, когда вы пишете комментарий к видео, например { $tag }.IMG_0424.MOV, и снимается, когда вы его удаляете (описания от AI не считаются). В остальное время его ставите и снимаете вы.
settings-commented-tag-off = Видео с комментарием не получают тег.
settings-in-out = Точки входа и выхода
settings-in-out-in-video = Adobe: маркер внутри видеофайла (XMP; в Premiere Pro — subclip / «подклип»)
settings-in-out-file-name = В имени файла (in_HH_MM_SS / out_HH_MM_SS)
settings-in-out-note = Точки входа и выхода остаются там, где были, пока их не перенесут.
settings-in-out-move-into-videos = Перенести имеющиеся точки входа и выхода из имён файлов в видео…
settings-in-out-move-into-file-names = Перенести имеющиеся точки входа и выхода из видео в имена файлов…
settings-markers = Маркеры и диапазоны
settings-markers-in-video = Внутри видеофайла (XMP; Premiere Pro показывает их на клипе)
settings-markers-comment = В комментарии, по одной строке (0:41–0:47 — Lion)
settings-markers-note = Маркеры остаются там, где были, пока их не перенесут.
settings-markers-move-into-videos = Перенести имеющиеся маркеры из комментариев в видео…
settings-markers-copy-into-comment = Скопировать имеющиеся маркеры из видео в комментарии…
settings-markers-hint = Как точки, так и диапазоны. Моменты, которые находит «Описать с помощью AI», идут так же: белые маркеры или строки описания.
settings-updates = Обновления
settings-old-title = Настройки из старой версии frename
settings-old-scheduled = Настройки будут перенесены при перезапуске frename
settings-old-not-found = В { $folder } нет frename.exe с frename.db
settings-old-failed = Не удалось перенести: { $reason }
settings-old-import-button = Импортировать из папки старой версии frename…
settings-ai = AI
settings-ai-key-label = Ключ API Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Получить ключ на console.anthropic.com → API keys.
settings-ai-model-label = Модель
settings-ai-language-label = Язык описания
settings-ai-hint = Используется действием «Описать с помощью AI» в пакетном режиме. Haiku дешевле всех и подходит для большинства клипов; Sonnet и Opus замечают больше и стоят дороже.
settings-subtitles = Субтитры
settings-subtitles-key-label = Ключ API Soniox
settings-subtitles-key-placeholder = Вставьте ключ
settings-subtitles-key-get = Получить ключ на console.soniox.com. Аудио отправляется в Soniox для распознавания.
settings-subtitles-languages-label = Языки
settings-subtitles-languages-locked = Сохраните ключ, чтобы выбирать из всех языков Soniox.
settings-subtitles-languages-loading = Получение списка языков от Soniox…
settings-subtitles-languages-none = Ничего не отмечено: язык определяется автоматически.
settings-subtitles-languages-hint = Языки, на которых говорят в кадре, как подсказка.
settings-subtitles-cue-length-label = Длина субтитров
settings-subtitles-cue-short = Короткие (одна строка, до 8 с)
settings-subtitles-cue-sentence = По предложению
settings-subtitles-hint = Используется действием «Распознать субтитры» в пакетном режиме. Длина субтитров применяется к новым субтитрам.
settings-key-store-windows = Windows Credential Manager
settings-key-store-macos = связке ключей macOS
settings-key-store-other = системном хранилище паролей
settings-key-unavailable = Не удалось открыть системное хранилище паролей
settings-key-unavailable-hint = Оно может быть заблокировано или отсутствовать (например, GNOME Keyring или KWallet). Настройки проверяют это заново при каждом открытии.
settings-key-remove-confirm = Удалить сохранённый ключ? Его придётся вставить снова.
settings-key-remove = Удалить
settings-key-keep = Оставить
settings-key-saved = Ключ сохранён
settings-key-replace = Заменить
settings-key-hide = Скрыть
settings-key-show = Показать
settings-key-save = Сохранить
settings-key-saved-in = Сохранён в { $store } на этом компьютере.
settings-key-save-into = «Сохранить» помещает его в { $store } на этом компьютере.

## Batch mode

batch-title = Пакетные действия
batch-on-checked = для { $count ->
    [one] { $count } отмеченного файла
    [few] { $count } отмеченных файлов
   *[many] { $count } отмеченных файлов
}
batch-run = Применить к { $count ->
    [one] { $count } файлу
    [few] { $count } файлам
   *[many] { $count } файлам
}
batch-done-label-changed = изменено
batch-done-label-subtitled = субтитровано
batch-counts = ✓ { $done_label }: { $done }   – без изменений: { $skipped }   ✗ с ошибкой: { $failed }
batch-ai-at-least = как минимум
batch-ai-spend-line = AI: { $spend }
batch-cancel = Отмена
batch-stopping = Остановка…
batch-stopped = Остановлено после { $finished } из { $total ->
    [one] { $total } файла
    [few] { $total } файлов
   *[many] { $total } файлов
}.
batch-finished = Готово: { $total } { $total ->
    [one] файл
    [few] файла
   *[many] файлов
}.
batch-close = Закрыть
batch-failed-subtitles = Без субтитров:
batch-failed-plain = С ошибкой:
batch-failed-with-log = С ошибкой (подробности в журнале):
batch-retry = Повторить
batch-add-credit = Пополнить счёт
batch-open-log = Открыть журнал

batch-action-move-comments = Перенести комментарии
batch-action-move-comments-hint = Переносит комментарий каждого отмеченного файла в выбранное место. Теги и точки входа и выхода остаются на месте.
batch-action-move-comments-into-videos = Из текстовых файлов в видео (XMP)
batch-action-move-comments-into-text-files = Из видео (XMP) в текстовые файлы

batch-action-move-in-out = Перенести точки входа и выхода
batch-action-move-in-out-hint = Переносит точки входа и выхода каждого отмеченного файла в выбранное место и переименовывает файлы, в имени которых они появляются или исчезают. Комментарии остаются на месте.
batch-action-move-in-out-into-videos = Из имён файлов в видео (маркер Adobe XMP)
batch-action-move-in-out-into-file-names = Из видео (маркер XMP) в имена файлов

batch-action-markers-comment = Маркеры ⇄ комментарий
batch-action-markers-comment-to-markers = Строки комментария с временем в маркеры
batch-action-markers-to-comment = Маркеры в комментарий (копия: маркеры остаются)
batch-action-markers-comment-hint = Строка вида «03:24 — Дубль 3 — приятный свет» — это маркер на 3:24 с именем «Дубль 3» и комментарием «приятный свет»; «0:41-0:47 — Лев» — маркер с 0:41 до 0:47. Имя и комментарий делятся по первому « — » или « -- », а не по простому « - ». Моменты описания от AI («0:00–0:14 Улица.») становятся белыми маркерами и остаются в описании. Повторный запуск в любую сторону не добавляет ничего дважды.

batch-action-tag-commented = Тег видео с комментарием
batch-action-tag-commented-hint = Ставит тег «{ $tag }» каждому отмеченному видео с вашим комментарием (описания от AI не считаются) и снимает его с видео без комментария. Файлы, у которых тег меняется, переименовываются.
batch-action-tag-commented-hint-off = Ставит тег для видео с комментарием каждому отмеченному видео с вашим комментарием и снимает его с видео без комментария. Сейчас этот тег выключен в настройках.
batch-action-tag-commented-status = Тег: { $tag }
batch-action-tag-commented-status-off = Тег: выключен
batch-action-tag-commented-settings = Настройки тега…

batch-action-fix-tags = Упорядочить теги по приоритету
batch-action-fix-tags-hint = Расставляет теги в имени каждого отмеченного файла в порядке панели тегов: чем выше тег в панели, тем раньше он в имени. Теги, которых папка ещё не знает, идут первыми, как в панели тегов. Файлы, у которых порядок меняется, переименовываются.

batch-action-respace-tags = Применить пробелы после тегов
batch-action-respace-tags-hint-space = Переименовывает каждый отмеченный файл так, чтобы после каждого тега стоял пробел, как задано в настройках: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Переименовывает каждый отмеченный файл так, чтобы после тегов не было пробелов, как задано в настройках: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Пробел после каждого тега
batch-action-respace-tags-status-no-space = Без пробелов после тегов
batch-action-respace-tags-settings = Настройки пробелов…

batch-action-reload-files = Сбросить кэш и перечитать
batch-action-reload-files-hint = Заново читает комментарий и точки входа и выхода каждого отмеченного файла из самого файла и заменяет то, что папка о нём запомнила. Нужно, если файлы менялись в другой программе. Файлы, у которых запомненное отсутствовало или устарело, считаются изменёнными.

batch-action-describe-ai = Описать с помощью AI
batch-action-describe-ai-run = Описать { $videos }
batch-action-describe-ai-estimating = Оценка… { $known } / { $total }
batch-action-describe-ai-none = Нет видео для описания.
batch-action-describe-ai-plan = { $videos }, { $minutes } · примерно { $dollars } моделью { $model }
batch-action-describe-ai-hint = Займёт примерно { $duration }. Папка заблокирована до конца. Отмена сохраняет уже описанные видео; повторный запуск пропускает их.
batch-action-describe-ai-no-subtitles = Без субтитров (описана только картинка): { $videos }.
batch-action-describe-ai-redo = Переописать видео, у которых уже есть описание от AI
batch-action-describe-ai-hint-panel = Описывает происходящее в каждом отмеченном видео с привязкой ко времени: краткое содержание и отрезки по времени добавляются в описание от AI в его комментарии; ваш собственный текст сохраняется. Кадры и субтитры отправляются в Anthropic.
batch-ai-change = Изменить
batch-ai-open-settings = Открыть настройки
batch-ai-key-missing = Укажите ключ API Anthropic в настройках
batch-ai-key-unavailable = Не удалось открыть системное хранилище паролей: оно может быть заблокировано или отсутствовать (например, GNOME Keyring или KWallet).
batch-ai-language-same-as-subtitles = Описания на языке субтитров (на английском, если субтитров нет)
batch-ai-language = Описания на языке: { $language }

## AI description language names: the Settings picker and { $language } above.

ai-language-same-as-subtitles = Как в субтитрах
ai-language-english = Английский
ai-language-russian = Русский
ai-language-ukrainian = Украинский
ai-language-german = Немецкий
ai-language-spanish = Испанский
ai-language-french = Французский
batch-videos-count = { $n ->
    [one] { $n } видео
    [few] { $n } видео
   *[many] { $n } видео
}
batch-minutes = { $n } мин
batch-ai-minutes-under = < 1 мин
batch-ai-duration-under-minute = меньше минуты
batch-ai-duration-hours = { $h } ч
batch-ai-duration-hours-minutes = { $h } ч { $m } мин
batch-ai-dollars-under = меньше $0.01
batch-ai-skip-described = уже описано
batch-ai-skip-too-long = длиннее 30 мин
batch-ai-skip-unreadable = не читается
batch-ai-skip-photos = { $n ->
    [one] { $n } фото
    [few] { $n } фото
   *[many] { $n } фото
}
batch-ai-skipped = Пропущено: { $parts }.
batch-ai-progress-frame = кадр { $done } из { $total }
batch-ai-progress-waiting = ожидание ответа Claude
batch-ai-progress-saving = сохранение
batch-ai-stop-no-key = Остановлено: нет ключа API Anthropic. Укажите его в настройках.
batch-ai-fail-no-key = Нет ключа API
batch-ai-fail-unreadable = Видео не удалось прочитать
batch-ai-fail-not-saved = Не удалось сохранить описание
batch-ai-stop-offline = Остановлено: нет соединения с Anthropic. Запустите ещё раз, чтобы описать оставшиеся.

batch-action-generate-subtitles = Распознать субтитры
batch-action-generate-subtitles-install-ffmpeg = чтобы читать .mkv, .m2ts, .avi …, установите ffmpeg с ffmpeg.org, добавьте его в PATH и перезапустите frename
batch-action-generate-subtitles-replace = Заменить имеющиеся субтитры
batch-action-generate-subtitles-replace-hint = Распознаёт заново; стоимость как показано.
batch-action-generate-subtitles-privacy = Аудио этих видео отправляется в Soniox и затем удаляется там.
batch-action-generate-subtitles-duration-hint = Занимает несколько минут на час аудио; папка заблокирована до конца. Закрытие frename останавливает распознавание; готовые субтитры сохраняются.
batch-action-generate-subtitles-hint = Распознаёт речь каждого отмеченного видео через Soniox и сохраняет субтитры рядом с ним (clip.srt) — там, где их показывает frename.
batch-subtitles-transcribe = Распознать
batch-subtitles-transcribe-count = Распознать { $videos }
batch-subtitles-build-free = Собрать { $count } (бесплатно)
batch-subtitles-nothing = Нечего распознавать
batch-subtitles-estimating = Оценка…
batch-subtitles-key-missing = Укажите ключ API Soniox в настройках
batch-subtitles-key-rejected = Soniox отклонил ключ
batch-subtitles-cost-unknown = стоимость неизвестна
batch-subtitles-typical-price = { $usd } (обычная цена)
batch-subtitles-unknown-length = + { $n } неизвестной длины
batch-subtitles-plan-line = { $videos } на распознавание, { $duration } аудио{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
    [one] { $n } уже с субтитрами
    [few] { $n } уже с субтитрами
   *[many] { $n } уже с субтитрами
}
batch-subtitles-rebuilt-free = { $n } собрано бесплатно из сохранённой расшифровки
batch-subtitles-no-speech-before = { $n } в прошлый раз без речи
batch-subtitles-no-audio = { $n ->
    [one] { $n } без звука
    [few] { $n } без звука
   *[many] { $n } без звука
}
batch-subtitles-shared-name = { $n ->
    [one] { $n } делит имя субтитров с другим видео
    [few] { $n } делят имя субтитров с другим видео
   *[many] { $n } делят имя субтитров с другим видео
}
batch-subtitles-unreadable = { $n } не читается ({ $how_to })
batch-subtitles-shared-name-reason = делит имя субтитров с другим видео
batch-subtitles-unsupported-reason = формат аудио не поддерживается
batch-subtitles-progress-transcribing = распознавание
batch-subtitles-reason-already = уже были субтитры
batch-subtitles-reason-no-audio = нет звука
batch-subtitles-reason-no-speech = нет речи
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = не удалось связаться с Soniox
batch-subtitles-stop-key-rejected = Остановлено: Soniox отклонил ключ. Проверьте его в настройках → Субтитры.
batch-subtitles-stop-balance-empty = Остановлено: баланс Soniox пуст. Пополните его на console.soniox.com.
batch-subtitles-stop-budget-used = Остановлено: месячный бюджет Soniox исчерпан. Увеличьте его на console.soniox.com.
batch-subtitles-stop-other = Остановлено: Soniox отказался продолжать ({ $error_type }).
batch-subtitles-stop-unreachable = Остановлено: не удалось связаться с Soniox. Проверьте подключение к интернету.
batch-subtitles-report-spend = Soniox: как минимум { $duration } · { $usd }
batch-subtitles-report-failed-deletes = { $n ->
    [one] { $n } загрузку
    [few] { $n } загрузки
   *[many] { $n } загрузок
} не удалось удалить в Soniox (подробности в журнале)
batch-subtitles-languages-need-key = Сохраните ключ Soniox, чтобы выбирать из всех его языков.
batch-subtitles-languages-rejected = Soniox отклонил ключ.
batch-subtitles-languages-failed = Не удалось получить список языков от Soniox.
batch-subtitles-count = { $n ->
    [one] { $n } субтитр
    [few] { $n } субтитра
   *[many] { $n } субтитров
}
batch-subtitles-duration-seconds = { $s } с
batch-subtitles-usd-under = меньше $0.01
batch-subtitles-usd-about = примерно ${ $amount }

## File list

folder-all = Все
folder-invert = Обратить
folder-checked = Отмечено: { $count }
folder-outcome-changed = Изменено
folder-outcome-unchanged = Нечего менять
folder-outcome-failed = Ошибка, подробности в журнале
folder-rename-error-empty = Имя пустое
folder-rename-error-bad-character = Нельзя: \ / : * ? " < > |
folder-rename-error-trailing = Не может кончаться точкой или пробелом
folder-rename-error-exists = Файл с таким именем уже есть
folder-markers-not-saved = Маркеры не сохранены: файл доступен только для чтения или занят (закройте его в Premiere, затем откройте файл и снова закройте)
drag-out-not-saved = Не перетаскивается: не удалось сохранить файл (только для чтения или открыт в другой программе, например Premiere; закройте его там и повторите)

## Controls bar under the file list

folder-controls-filter = Фильтр
folder-controls-filter-active = Фильтр ({ $count })
folder-controls-filter-untagged = Без тегов
folder-controls-filter-subtitles = С субтитрами
folder-controls-filter-comments = С комментарием
folder-controls-filter-markers = С маркерами
folder-controls-scroll = Прокрутить к файлу
folder-controls-open = Открыть папку (правый клик: открыть файл)
folder-controls-batch = Пакетные действия с отмеченными файлами
folder-controls-batch-back = Назад к открытому файлу
folder-controls-update-available = Доступно обновление: { $version }

## Video

video-controls-set-in = [  Точка входа
video-controls-set-out = ]  Точка выхода
video-controls-screenshot = Сохранить этот кадр как JPEG (F12)
video-controls-add-marker = Добавить маркер (F2, удерживайте для диапазона; ещё раз — назвать)
video-controls-cannot-hold-markers = Этот файл не может хранить маркеры
video-controls-add-a-name = Добавить имя
media-viewer-video-show-subtitles = Показать список субтитров
media-viewer-video-hide-subtitles = Скрыть список субтитров
media-viewer-video-markers-hint = Маркеры (Shift+F1 / Shift+F3 — переход, Shift+перетаскивание — привязка)
media-viewer-video-tab-markers = Маркеры

## Markers list

markers-empty = Маркеров пока нет
markers-add = 📍 Добавить маркер (F2)
markers-ai-hint = Маркер AI: заменяется при повторном описании этого клипа
markers-keep-color = Оставить цвет
markers-done-enter = Готово (Enter)
markers-delete = Удалить маркер
markers-read-only = только чтение
markers-name-placeholder = Имя

## File workspace

file-workspace-comment-placeholder = Комментарий...
file-workspace-comment-collapse = Назад к тегам
file-workspace-comment-expand = Развернуть комментарий

## Updates (Settings)

updates-check = Проверить обновления
updates-not-installed = Обновления работают только в установленной версии
updates-checking = Проверка…
updates-downloading-named = Загрузка { $version }… { $percent }%
updates-downloading = Загрузка… { $percent }%
updates-restarting = Перезапуск…
updates-check-failed = Не удалось проверить обновления: { $reason }
updates-update-failed = Не удалось обновить: { $reason }
updates-version-available = Доступна версия { $version }
updates-up-to-date = frename обновлён до последней версии
updates-update-and-restart = Обновить и перезапустить
updates-wait-for-batch = Подождите завершения пакетного действия
updates-current-version = frename { $version }
updates-check-on-start = Проверять обновления при запуске frename
