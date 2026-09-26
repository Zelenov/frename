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
settings-commented-tag-hint = Тег ставится, когда у видео появляется комментарий, например { $tag }.IMG_0424.MOV, и снимается, когда комментарий удалён. В остальное время его ставите и снимаете вы. Название тега можно изменить.
settings-commented-tag-off = Видео с комментарием не получают тег.
settings-in-out = Точки входа и выхода
settings-in-out-in-video = Adobe: маркер внутри видеофайла (XMP; в Premiere Pro — subclip / «подклип»)
settings-in-out-file-name = В имени файла (in_HH_MM_SS / out_HH_MM_SS)
settings-in-out-note = Точки входа и выхода остаются там, где были, пока их не перенесут.
settings-in-out-move-into-videos = Перенести имеющиеся точки входа и выхода из имён файлов в видео…
settings-in-out-move-into-file-names = Перенести имеющиеся точки входа и выхода из видео в имена файлов…

## Batch mode

batch-title = Пакетные действия
batch-on-checked = для { $count ->
    [one] { $count } отмеченного файла
    [few] { $count } отмеченных файлов
   *[many] { $count } отмеченных файлов
}
batch-back = Назад к открытому файлу
batch-run = Применить к { $count ->
    [one] { $count } файлу
    [few] { $count } файлам
   *[many] { $count } файлам
}
batch-counts = ✓ изменено: { $done }   – без изменений: { $skipped }   ✗ с ошибкой: { $failed }
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
batch-failed = С ошибкой (подробности в журнале):

batch-action-move-comments = Перенести комментарии
batch-action-move-comments-hint = Переносит комментарий каждого отмеченного файла в выбранное место. Теги и точки входа и выхода остаются на месте.
batch-action-move-comments-into-videos = Из текстовых файлов в видео (XMP)
batch-action-move-comments-into-text-files = Из видео (XMP) в текстовые файлы

batch-action-move-in-out = Перенести точки входа и выхода
batch-action-move-in-out-hint = Переносит точки входа и выхода каждого отмеченного файла в выбранное место и переименовывает файлы, в имени которых они появляются или исчезают. Комментарии остаются на месте.
batch-action-move-in-out-into-videos = Из имён файлов в видео (маркер Adobe XMP)
batch-action-move-in-out-into-file-names = Из видео (маркер XMP) в имена файлов

batch-action-tag-commented = Тег видео с комментарием
batch-action-tag-commented-hint = Ставит тег «{ $tag }» каждому отмеченному видео с комментарием и снимает его с видео без комментария. Файлы, у которых тег меняется, переименовываются.
batch-action-tag-commented-hint-off = Ставит тег для видео с комментарием каждому отмеченному видео с комментарием и снимает его с видео без комментария. Сейчас этот тег выключен в настройках.
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

## Controls bar under the file list

folder-controls-filter = Фильтр
folder-controls-filter-active = Фильтр ({ $count })
folder-controls-filter-untagged = Без тегов
folder-controls-filter-subtitles = С субтитрами
folder-controls-filter-comments = С комментарием
folder-controls-scroll = Прокрутить к файлу
folder-controls-open = Открыть файл
folder-controls-settings = Настройки
folder-controls-batch = Пакетные действия с отмеченными файлами
folder-controls-batch-back = Назад к открытому файлу

## Video

video-controls-set-in = [  Точка входа
video-controls-set-out = ]  Точка выхода
media-viewer-video-show-subtitles = Показать список субтитров
media-viewer-video-hide-subtitles = Скрыть список субтитров
file-workspace-comment-placeholder = Комментарий...
