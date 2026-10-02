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
settings-page-interface = Интерфейс
settings-page-saving = Хранение
settings-page-ai = Описание от AI
settings-close = Закрыть
settings-apply-note = Изменения применяются сразу. Ctrl+Tab — следующая страница.
settings-language = Язык
settings-language-system = Как в системе ({ $language })
settings-video = Видео
settings-video-autoplay = Сразу воспроизводить открытое видео
settings-tags = Цвет тегов
settings-tags-monochrome = Одноцветные
settings-tags-monochrome-hint = Добавленные вами теги — тёмно-серые, остальные — светло-серые, других цветов нет.
settings-file-names = Имена файлов
settings-tags-space-after = Пробел после каждого тега
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = Имена файлов останутся прежними, пока файлы не переименуют или не сохранят.
settings-tags-space-add = Добавить пробел в имена файлов…
settings-tags-space-remove = Убрать пробел из имён файлов…
settings-comments = Комментарии
settings-comments-in-video = Внутри видеофайла
settings-comments-in-video-hint = XMP; в Premiere Pro — колонка Description / «Описание»
settings-comments-text-file = В текстовом файле рядом с видео
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = Комментарии остаются там, где были, пока их не перенесут.
settings-comments-move-into-videos = Перенести комментарии в видео…
settings-comments-move-into-text-files = Перенести комментарии в текстовые файлы…
settings-commented-tag = Ставить тег видео с комментарием
settings-commented-tag-name = Тег
settings-commented-tag-hint = Тег ставится, когда вы пишете комментарий к видео, например { $tag }.IMG_0424.MOV, и снимается, когда вы его удаляете (описания от AI не считаются). В остальное время его ставите и снимаете вы.
settings-commented-tag-off = Видео с комментарием не получают тег.
settings-in-out = Точки входа и выхода
settings-in-out-in-video = Внутри видеофайла
settings-in-out-in-video-hint = Маркер subclip / «подклип» в Premiere Pro
settings-in-out-comment = В комментарии, одной строкой
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = Точки входа и выхода остаются там, где были, пока их не перенесут.
settings-in-out-move-into-videos = Перенести точки входа и выхода в видео…
settings-in-out-move-into-comments = Перенести точки входа и выхода в комментарии…
settings-markers = Маркеры и диапазоны
settings-markers-in-video = Внутри видеофайла
settings-markers-in-video-hint = XMP; Premiere Pro показывает их на клипе
settings-markers-comment = В комментарии, по одной строке
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = Маркеры остаются там, где были, пока их не перенесут.
settings-markers-move-into-videos = Перенести маркеры в видео…
settings-markers-copy-into-comment = Скопировать маркеры в комментарии…
settings-markers-hint = И точки, и диапазоны, в том числе найденные AI.
settings-updates = Обновления
settings-version = Версия
settings-old-title = Настройки из старой версии frename
settings-old-hint = Перенести настройки frename, запущенного из zip-папки.
settings-old-scheduled = Настройки будут перенесены при перезапуске frename
settings-old-not-found = В { $folder } нет frename.exe с frename.db
settings-old-failed = Не удалось перенести: { $reason }
settings-old-import-button = Импортировать из папки старой версии frename…
settings-ai = Описать с помощью AI
settings-ai-key-label = Ключ API Anthropic
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = Получить ключ на console.anthropic.com → API keys.
settings-ai-key-remove-confirm = Удалить сохранённый ключ Anthropic?
settings-ai-used-by = Используется действием «Описать с помощью AI» в пакетном режиме и на маркерах (✨, Ctrl+F2).
settings-ai-model-label = Модель
settings-ai-language-label = Язык описания
settings-ai-hint = Haiku дешевле всех; Sonnet и Opus замечают больше.
settings-subtitles = Субтитры
settings-subtitles-key-label = Ключ API Soniox
settings-subtitles-key-placeholder = Вставьте ключ
settings-subtitles-key-get = Получить ключ на console.soniox.com. Аудио отправляется в Soniox.
settings-subtitles-key-remove-confirm = Удалить сохранённый ключ Soniox?
settings-subtitles-languages-label = Языки
settings-subtitles-languages-locked = Сохраните ключ, чтобы выбирать из всех языков Soniox.
settings-subtitles-languages-loading = Получение списка языков от Soniox…
settings-subtitles-languages-none = Ничего не отмечено: язык определяется автоматически.
settings-subtitles-languages-hint = Языки, на которых говорят в кадре, как подсказка.
settings-subtitles-cue-length-label = Длина субтитров
settings-subtitles-cue-short = Короткие
settings-subtitles-cue-short-hint = Одна строка, до 8 с
settings-subtitles-cue-sentence = По предложению
settings-subtitles-hint = Используется действием «Распознать субтитры» в пакетном режиме. Длина субтитров применяется к новым субтитрам.
settings-key-store-windows = Windows Credential Manager
settings-key-store-macos = связке ключей macOS
settings-key-store-other = системном хранилище паролей
settings-key-unavailable = Не удалось открыть системное хранилище паролей
settings-key-unavailable-hint = Оно может быть заблокировано или отсутствовать (например, GNOME Keyring или KWallet). Настройки проверяют это заново при каждом открытии.
settings-key-remove-confirm-hint = Его придётся вставить снова.
settings-key-remove-ask = Удалить…
settings-key-remove = Удалить ключ
settings-key-keep = Оставить
settings-key-replace = Заменить…
settings-key-hide = Скрыть
settings-key-show = Показать
settings-key-save = Сохранить ключ
settings-key-cancel = Отмена
settings-key-saved-in = Сохранён в { $store } на этом компьютере
settings-key-save-into = «Сохранить ключ» помещает его в { $store } на этом компьютере.

## Batch mode

batch-back-while-running = Сначала отмените действие
batch-checked-count = { $count ->
    [one] отмечен { $count } файл
    [few] отмечено { $count } файла
   *[many] отмечено { $count } файлов
}
batch-no-key = Нет ключа
batch-group-move = Перенос между местами
batch-group-fix = Исправить имена и видео
batch-group-paid = Платные сервисы
batch-reason-none-checked = Нет отмеченных файлов
batch-reason-reading = Читаем длину клипов…
batch-check-all = Отметить все { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-job-running = выполняется
batch-job-finished = завершено
batch-job-stopped = остановлено
batch-progress-files = { $finished } из { $total ->
    [one] { $total } файла
    [few] { $total } файлов
   *[many] { $total } файлов
}
batch-time-left = осталось около { $time }
batch-time-estimating = оцениваем время…
batch-time-spent = прошло { $time }
batch-locked-until-end = Папка заблокирована до конца
batch-run-again = Запустить снова для { $count ->
    [one] { $count } файла
    [few] { $count } файлов
   *[many] { $count } файлов
}
batch-result-done = Готово: { $total ->
    [one] { $total } файл
    [few] { $total } файла
   *[many] { $total } файлов
}
batch-result-done-detail = изменено: { $changed }, без изменений: { $unchanged }
batch-result-problems = Готово с ошибками: не выполнено { $failed } из { $total ->
    [one] { $total } файла
    [few] { $total } файлов
   *[many] { $total } файлов
}
batch-figure-unchanged = без изменений
batch-figure-not-done = не выполнено
batch-figure-not-reached = не дошли
batch-files-not-done = Не выполнено
batch-table-file = Файл
batch-table-why = Почему

batch-run-move-comments = Перенести { $count ->
    [one] { $count } комментарий
    [few] { $count } комментария
   *[many] { $count } комментариев
}
batch-run-move-in-out = Перенести точки входа и выхода: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-run-convert = Преобразовать: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-run-rotate = Повернуть: { $count ->
    [one] { $count } видео
    [few] { $count } видео
   *[many] { $count } видео
}
batch-run-tag = Обновить тег: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-run-fix-tags = Упорядочить теги: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-run-rename = Переименовать: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-run-reload = Перечитать: { $count ->
    [one] { $count } файл
    [few] { $count } файла
   *[many] { $count } файлов
}
batch-service-anthropic = Anthropic
batch-service-soniox = Soniox
batch-change-renames = Переименовывает файлы
batch-change-videos = Пишет в видео
batch-change-text-files = Пишет текстовые файлы рядом с видео
batch-change-comments = Пишет в комментарии
batch-change-subtitles = Пишет файлы субтитров рядом с видео
batch-change-records = Меняет только записи самого frename
batch-option-direction = Направление
batch-option-language = Язык
batch-option-described = Уже описанные
batch-option-subtitled = С субтитрами
batch-option-subtitles-write = Какие файлы записать
batch-option-spacing = Пробелы
batch-option-tag = Тег
batch-option-turn = Поворот
batch-plan-videos = Видео
batch-plan-length = Длина
batch-plan-cost = Стоимость
batch-plan-time = Время
batch-reason-estimate = Ждём оценку
batch-reason-subtitles-no-format = Отметьте хотя бы один файл для записи
batch-set-key = Указать ключ…
batch-check-key = Проверить ключ…
batch-subtitles-languages-auto = Определяется в каждом видео
batch-action-describe-ai-run-waiting = Описать видео
batch-action-fix-tags-order = Порядок: как в списке тегов, неизвестные теги первыми.
batch-action-markers-comment-hint-short = Превращает строки комментария с временем в маркеры или копирует маркеры в комментарий.
batch-action-markers-to-comment-hint = Маркеры остаются; повторный запуск ничего не добавляет дважды.
batch-action-rotate-hint-short = Поворачивает клипы MP4 и MOV флагом поворота, без перекодирования.
batch-action-tag-commented-off = Тег для видео с комментарием выключен
batch-action-tag-commented-choose = Выбрать тег…

batch-title = Пакетные действия
batch-done-label-changed = изменено
batch-done-label-subtitled = субтитровано
batch-ai-at-least = как минимум
batch-cancel = Отмена
batch-stopping = Остановка…
batch-stopped = Остановлено после { $finished } из { $total ->
    [one] { $total } файла
    [few] { $total } файлов
   *[many] { $total } файлов
}.
batch-close = Закрыть
batch-failed-subtitles = Без субтитров:
batch-written-subtitles = Записанные файлы:
batch-table-written = Записано
batch-merged-markers = Объединённые дубли маркеров:
batch-table-merged = Объединено
batch-markers-merged = { $count ->
    [one] объединён { $count } дубль маркера
    [few] объединено { $count } дубля маркера
   *[many] объединено { $count } дублей маркера
}
batch-add-credit = Пополнить счёт
batch-open-log = Открыть журнал

batch-action-move-comments = Перенести комментарии
batch-action-move-comments-hint = Переносит комментарий каждого отмеченного файла в выбранное место вместе с точками входа и выхода в нём. Теги и точки входа и выхода внутри видео остаются на месте.
batch-action-move-comments-into-videos = Из текстовых файлов в видео (XMP)
batch-action-move-comments-into-text-files = Из видео (XMP) в текстовые файлы

batch-action-move-in-out = Точки входа и выхода: комментарий ⇄ видео (XMP)
batch-action-move-in-out-hint = Переносит точки входа и выхода каждого отмеченного файла в выбранное место. Остальной комментарий остаётся на месте.
batch-action-move-in-out-into-videos = Из комментариев в видео (маркер Adobe XMP)
batch-action-move-in-out-into-comments = Из видео (маркер XMP) в комментарии

batch-action-markers-comment = Маркеры ⇄ комментарий
batch-action-markers-comment-to-markers = Строки комментария с временем в маркеры
batch-action-markers-to-comment = Маркеры в комментарий (копия: маркеры остаются)
batch-action-markers-comment-hint = Строка вида «03:24 — Дубль 3 — приятный свет» — это маркер на 3:24 с именем «Дубль 3» и комментарием «приятный свет»; «0:41-0:47 — Лев» — маркер с 0:41 до 0:47. Имя и комментарий делятся по первому « — » или « -- », а не по простому « - ». Моменты описания от AI («0:00–0:14 Улица.») становятся белыми маркерами и остаются в описании. Повторный запуск в любую сторону не добавляет ничего дважды.

batch-action-tag-commented = Тег видео с комментарием
batch-action-tag-commented-hint = Ставит тег «{ $tag }» каждому отмеченному видео с вашим комментарием (описания от AI не считаются) и снимает его с видео без комментария. Файлы, у которых тег меняется, переименовываются.
batch-action-tag-commented-hint-off = Ставит тег для видео с комментарием каждому отмеченному видео с вашим комментарием и снимает его с видео без комментария. Сейчас этот тег выключен в настройках.
batch-action-fix-tags = Упорядочить теги по приоритету
batch-action-fix-tags-hint = Расставляет теги в имени каждого отмеченного файла в порядке панели тегов: чем выше тег в панели, тем раньше он в имени. Теги, которых папка ещё не знает, идут первыми, как в панели тегов. Файлы, у которых порядок меняется, переименовываются.

batch-action-respace-tags = Применить пробелы после тегов
batch-action-respace-tags-hint-space = Переименовывает каждый отмеченный файл так, чтобы после каждого тега стоял пробел, как задано в настройках: Food. Goat. clip.mp4.
batch-action-respace-tags-hint-no-space = Переименовывает каждый отмеченный файл так, чтобы после тегов не было пробелов, как задано в настройках: Food.Goat.clip.mp4.
batch-action-respace-tags-status-space = Пробел после каждого тега
batch-action-respace-tags-status-no-space = Без пробелов после тегов
batch-action-reload-files = Сбросить кэш и перечитать
batch-action-reload-files-hint = Заново читает комментарий и точки входа и выхода каждого отмеченного файла из самого файла и заменяет то, что папка о нём запомнила. Нужно, если файлы менялись в другой программе. Файлы, у которых запомненное отсутствовало или устарело, считаются изменёнными.

batch-action-describe-ai = Описать с помощью AI
batch-action-describe-ai-run = Описать { $videos } · около { $dollars }
batch-action-describe-ai-estimating = Оценка… { $known } / { $total }
batch-action-describe-ai-none = Нет видео для описания.
batch-action-describe-ai-hint = Папка заблокирована до конца. Отмена сохраняет уже описанные видео; повторный запуск пропускает их.
batch-action-describe-ai-no-subtitles = Без субтитров (описана только картинка): { $videos }.
batch-action-describe-ai-redo = Переописать видео, у которых уже есть описание от AI
batch-action-describe-ai-hint-panel = Описывает происходящее в каждом отмеченном видео с привязкой ко времени: краткое содержание и отрезки по времени добавляются в описание от AI в его комментарии; ваш собственный текст сохраняется. Кадры и субтитры отправляются в Anthropic.
batch-ai-change = Изменить
batch-ai-key-missing = Укажите ключ API Anthropic в настройках
batch-ai-key-unavailable = Не удалось открыть системное хранилище паролей: оно может быть заблокировано или отсутствовать (например, GNOME Keyring или KWallet).
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
batch-ai-skipped = Пропущено: { $parts }.
batch-ai-progress-frame = кадр { $done } из { $total }
batch-ai-progress-waiting = ожидание ответа Claude
batch-ai-progress-saving = сохранение
batch-ai-stop-no-key = Остановлено: нет ключа API Anthropic. Укажите его в настройках.
batch-ai-fail-no-key = Нет ключа API
batch-ai-fail-unreadable = Видео не удалось прочитать
batch-ai-fail-too-long = Видео слишком длинное для AI (больше 30 мин)
batch-ai-fail-not-saved = Не удалось сохранить описание
batch-ai-stop-offline = Остановлено: нет соединения с Anthropic. Запустите ещё раз, чтобы описать оставшиеся.

batch-action-generate-subtitles = Распознать субтитры
batch-action-generate-subtitles-install-ffmpeg = чтобы читать .mkv, .m2ts, .avi …, установите ffmpeg с ffmpeg.org, добавьте его в PATH и перезапустите frename
batch-action-generate-subtitles-replace = Заменить имеющиеся субтитры
batch-action-generate-subtitles-replace-hint = Распознаёт заново; стоимость как показано.
batch-action-generate-subtitles-srt = Субтитры SRT (clip.srt)
batch-action-generate-subtitles-premiere = Транскрипт для Premiere Pro (clip.premiere.json)
batch-action-generate-subtitles-premiere-hint = В Premiere Pro: панель «Текст» → «Транскрипция» → «Импортировать статическую транскрипцию».
batch-action-generate-subtitles-privacy = Аудио этих видео отправляется в Soniox и затем удаляется там.
batch-action-generate-subtitles-duration-hint = Занимает несколько минут на час аудио; папка заблокирована до конца. Закрытие frename останавливает распознавание; готовые субтитры сохраняются.
batch-action-generate-subtitles-hint = Распознаёт речь каждого отмеченного видео через Soniox и сохраняет результат рядом с ним: субтитры (clip.srt), где их показывает frename, и/или транскрипт для Premiere Pro.
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
folder-has-subtitles = Есть субтитры
folder-closed-by-job = Закрыт, пока его меняет пакетное действие
folder-filter-tip = Показать только…

folder-search-placeholder = Найти файл или слово в комментарии
folder-search-comments-loading = Ищем в комментариях… осталось { $n }
folder-search-clear = Очистить
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
drag-out-not-saved = Не перетащено: файл не сохранён (только чтение или открыт в Premiere)
folder-opening = Открываем папку…
folder-empty-title = В этой папке нет видео
folder-empty-line = frename показывает MP4, MOV, MKV и другие видеофайлы.
folder-open-another = Открыть другую папку…
folder-no-match = Нет подходящих файлов
folder-show-all = Показать все
folder-locked = Заблокировано, пока выполняется «{ $action }»
folder-checked-hidden = Отмечено: { $count } · скрыто: { $hidden }
folder-checked-hidden-tip = { $hidden ->
    [one] { $hidden } отмеченный файл скрыт поиском или фильтром; действие применится и к нему
    [few] { $hidden } отмеченных файла скрыты поиском или фильтром; действие применится и к ним
   *[many] { $hidden } отмеченных файлов скрыты поиском или фильтром; действие применится и к ним
}
folder-outcome-working = В работе
folder-outcome-not-reached = Не обработан
folder-window-empty-title = Откройте папку с клипами
folder-window-empty-line = Или перетащите папку в окно. Правый клик по кнопке открывает один файл.
folder-window-open = Открыть папку…

## File menu (right-click on a file, or F11 / Shift+F11 / Ctrl+F11)

file-menu-show-in-explorer = Показать в проводнике
file-menu-show-in-file-manager = Показать в файловом менеджере
file-menu-copy-path = Копировать полный путь
file-menu-copy-name = Копировать имя файла
file-menu-copied = Скопировано
file-menu-not-copied = Не скопировано: буфер обмена недоступен
file-menu-not-shown = Не удалось открыть файловый менеджер

## Controls bar under the file list

folder-controls-filter-untagged = Без тегов
folder-controls-filter-subtitles = С субтитрами
folder-controls-filter-comments = С комментарием
folder-controls-filter-markers = С маркерами
folder-controls-scroll = Показать открытый файл в списке
folder-controls-open = Открыть папку
folder-controls-batch = Пакетные действия с отмеченными файлами
folder-controls-batch-back = Назад к открытому файлу
folder-controls-update-available = Доступно обновление: { $version }
folder-controls-previous = Предыдущий файл
folder-controls-next = Следующий файл
folder-controls-open-file = Правый клик: открыть один файл

## Video

video-controls-back = Назад на 10 с
video-controls-play = Воспроизвести
video-controls-pause = Пауза
video-controls-forward = Вперёд на 10 с
video-controls-frame-back = На кадр назад
video-controls-frame-forward = На кадр вперёд
video-controls-set-in = Поставить точку входа
video-controls-set-out = Поставить точку выхода
video-controls-screenshot = Сохранить этот кадр
video-controls-add-marker = Добавить маркер
video-controls-add-marker-hold = Удерживайте для диапазона; нажмите ещё раз, чтобы назвать
video-controls-cannot-hold-markers = Этот файл не может хранить маркеры
video-controls-add-a-name = Добавить имя
video-controls-rotate-left = Повернуть влево
video-controls-rotate-right = Повернуть вправо
video-controls-more = Ещё
media-viewer-loading-slow = Ждём файл… (файл из облака может загружаться долго)
media-viewer-cannot-play = Этот клип не воспроизводится
media-viewer-no-picture = В этом файле нет видеоизображения
video-controls-volume-scroll = Громкость — прокрутите, чтобы изменить
rotate-cannot = Нельзя повернуть: { $reason }
rotate-reason-missing = файла больше нет на месте
rotate-flag-right = 90° вправо
rotate-flag-left = 90° влево
rotate-flag-half = 180°
rotate-flag-none = нет
rotate-now = Поворот: { $flag }
rotate-turned = Повёрнуто на { $turn } · теперь поворот: { $flag }
rotate-failed = Не повёрнуто: { $reason }
undo-back-on-clip = Снова на этом ролике (переход отменён). Ещё раз Ctrl+Z — отмена его последнего изменения.
redo-on-clip = Снова на этом ролике (переход повторён).
rotate-reason-in-use = файл только для чтения или занят
rotate-reason-format = в этом формате нет флага поворота
rotate-reason-damaged = файл повреждён
rotate-reason-no-video = в файле нет видеодорожки
rotate-reason-matrix = у видео необычная матрица отображения
batch-action-rotate = Повернуть видео
batch-action-rotate-hint = Меняет флаг поворота каждого отмеченного файла MP4 и MOV; изображение не перекодируется, комментарии, точки входа и выхода и маркеры остаются. Ожидается, что Premiere Pro покажет клип повёрнутым при импорте. В других форматах флага поворота нет, для них действие не выполнится. Повторный запуск поворачивает файлы ещё раз; «Сбросить» убирает любой поворот, в том числе записанный телефоном, так что портретный клип с телефона будет лежать на боку.
batch-action-rotate-right = На 90° вправо (по часовой стрелке)
batch-action-rotate-left = На 90° влево (против часовой стрелки)
batch-action-rotate-half = На 180°
batch-action-rotate-reset = Сбросить: без поворота (0°)
media-viewer-video-subtitle-list = Список субтитров
media-viewer-video-marker-list = Список маркеров
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 — переход между маркерами; Shift+перетаскивание — привязка
media-viewer-video-tab-markers = Маркеры
media-viewer-video-fullscreen = Во весь экран
media-viewer-video-close-list = Закрыть список

## Markers list
markers-in-out = Точки входа и выхода

markers-empty = Маркеров пока нет
markers-add = Добавить маркер
markers-ai-hint = Маркер AI: заменяется при повторном описании этого клипа
markers-keep-color = Оставить цвет
markers-done = Готово
markers-delete = Удалить маркер
markers-cannot-hold-hint = Premiere читает маркеры из файлов MP4 и MOV.
markers-color-green = Зелёный
markers-color-red = Красный
markers-color-orange = Оранжевый
markers-color-yellow = Жёлтый
markers-color-white = Белый
markers-color-blue = Синий
markers-color-cyan = Голубой
markers-color-lavender = Лавандовый
markers-color-magenta = Пурпурный
markers-color-other = Другой цвет
markers-read-only = только чтение
markers-name-placeholder = Имя
markers-ai-describe = Описать с помощью AI: назвать маркер и добавить, что происходит
markers-ai-stop = Остановить описание
markers-ai-describing = Описываю…
markers-ai-done = Маркер описан
markers-ai-nothing-new = У маркера уже есть это описание
markers-no-marker-notice = Здесь нет маркера
markers-ai-no-key = Нет ключа Anthropic API: задайте его в настройках
markers-ai-failed = Не описано: { $reason }
markers-ai-failed-unknown = что-то пошло не так
markers-read-only-notice = Этот маркер только для чтения
markers-ai-gone = Маркера больше нет: описание не добавлено
markers-ai-stopping-for-batch = Сначала останавливаю описание маркеров… Это может занять пару минут.

## File workspace

file-workspace-search-placeholder = Найти тег — или просто печатайте
file-workspace-search-clear = Очистить
file-workspace-comment-placeholder = Комментарий...
file-workspace-comment-collapse = Назад к тегам
file-workspace-comment-expand = Развернуть комментарий

## Tag grid

tag-grid-star = Звезда: держать наверху
tag-grid-unstar = Убрать звезду
tag-grid-save = Добавить в теги папки
tag-grid-delete = Удалить «{ $tag }» из тегов папки
tag-grid-create = Создать «{ $tag }»
tag-grid-no-file = Откройте клип, чтобы ставить теги
tag-grid-no-tags = Тегов пока нет
tag-grid-no-tags-hint = Наберите название и нажмите Enter, чтобы создать первый.
tag-grid-group-unsaved = Нет в тегах папки
tag-grid-group-folder = Теги папки
tag-grid-more = ещё { $count }

## Order strip

sync-panel-locked = Перестановка ниже меняет порядок папки
sync-panel-unlocked = Перестановка ниже меняет только этот клип
sync-panel-unlock = Открепить
sync-panel-lock = Закрепить
sync-panel-differs = Порядок не как в папке
sync-panel-use-for-folder = Сделать порядком папки
sync-panel-sort-like-folder = Упорядочить как в папке

## File name card

file-name-panel-no-tags = У клипа нет тегов
file-name-panel-untag = Снять тег
file-name-panel-clear-in = Убрать точку входа
file-name-panel-clear-out = Убрать точку выхода
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = Поставить вход и выход, которые предлагает AI

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

recovery-restored = frename закрылся неожиданно. Несохранённая работа над { $clip } восстановлена ({ $what }).
recovery-kept = frename закрылся неожиданно. Несохранённая работа над { $clip } ({ $what }) не применена: { $why }. Читаемая копия, чтобы набрать заново, лежит в { $folder }.
recovery-unreadable = frename закрылся неожиданно, а файл восстановления не удалось прочитать. Он сохранён в { $folder }.
recovery-why-gone = ролика нет или он переименован
recovery-why-changed = ролик изменился после ваших правок
recovery-why-not-written = файл с таким именем уже есть, либо ролик только для чтения или занят
recovery-tags = { $count ->
    [one] { $count } тег
    [few] { $count } тега
   *[many] { $count } тегов
}
recovery-markers = { $count ->
    [one] { $count } маркер
    [few] { $count } маркера
   *[many] { $count } маркеров
}
recovery-comment = комментарий
recovery-in-out = точки входа и выхода
recovery-edits = правки
