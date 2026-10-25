# frename の UI テキスト（日本語）。原文は i18n/en/frename.ftl で、すべてのファイルが
# 同じ id、属性、$引数を持ちます（src/i18n.rs のテストで確認）。
# キー: <機能>-<要素>[-<詳細>]。機能は src/features/ 内のフォルダー名です。

## 言語名: どのファイルでも同じ。各言語の表記で書きます。

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

## 設定ウィンドウ

settings-window-title = 設定
settings-page-interface = インターフェース
settings-page-saving = 保存
settings-page-ai = AI で説明
settings-close = 閉じる
settings-apply-note = 変更はすぐに反映されます。Tab でオプション間、Ctrl+Tab でページ間を移動します。
settings-language = 言語
settings-language-system = システム（{ $language }）
settings-video = ビデオ
settings-video-autoplay = 開いたときにビデオを自動再生する
settings-subtitle-size = 字幕のサイズ
settings-subtitle-size-small = 小
settings-subtitle-size-medium = 中
settings-subtitle-size-large = 大
settings-subtitle-size-extra-large = 特大
settings-timecode = 時間の表示
settings-timecode-hint = ビデオの下、マーカーと字幕のリスト、イン点／アウト点のチップ、および Ctrl+Shift+C でコピーする再生ヘッド位置の時間の書き方です。Premiere Pro と DaVinci Resolve は、クリップのフレームレートで時:分:秒:フレームを表示します。29.97 fps と 59.94 fps はドロップフレームです。フレームレートが一定でないクリップは、デフォルトの形式のままです。ファイルには、それぞれの形式で保存されます。
settings-timecode-default = デフォルト（mm:ss）
settings-timecode-premiere = Premiere Pro（HH:MM:SS:FF）
settings-timecode-resolve = DaVinci Resolve（HH:MM:SS:FF）
settings-tags = タグの色
settings-tags-monochrome = モノクロ
settings-tags-monochrome-hint = 追加したタグは濃いグレー、それ以外は薄いグレーで表示します。ほかの色は使いません。
settings-file-names = ファイル名
settings-tags-space-after = 各タグの後にスペース
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = ファイル名は、名前を変更するか保存するまでそのままです。
settings-tags-space-add = ファイル名にスペースを追加…
settings-tags-space-remove = ファイル名からスペースを削除…
settings-comments = コメント
settings-comments-in-video = ビデオファイルの中
settings-comments-in-video-hint = XMP、Premiere Pro の「説明」列
settings-comments-text-file = ビデオの隣のテキストファイル
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = コメントは、移動するまで今の場所に残ります。
settings-comments-move-into-videos = コメントをビデオに移動…
settings-comments-move-into-text-files = コメントをテキストファイルに移動…
settings-commented-tag = コメントのあるビデオにタグを付ける
settings-commented-tag-name = タグ
settings-commented-tag-hint = ビデオにコメントを書くとオンになり（例: { $tag }.IMG_0424.MOV）、消すとオフになります（AI の説明は対象外）。それ以外は自由に変更できます。
settings-commented-tag-off = コメントのあるビデオにタグは付きません。
settings-in-out = イン点／アウト点
settings-in-out-in-video = ビデオファイルの中
settings-in-out-in-video-hint = Premiere Pro のサブクリップマーカー
settings-in-out-comment = コメントの中（1 行）
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = イン点／アウト点は、移動するまで今の場所に残ります。
settings-in-out-move-into-videos = イン点／アウト点をビデオに移動…
settings-in-out-move-into-comments = イン点／アウト点をコメントに移動…
settings-markers = マーカーと範囲
settings-markers-in-video = ビデオファイルの中
settings-markers-in-video-hint = XMP、Premiere Pro のクリップ上に表示
settings-markers-comment = コメントの中（1 行に 1 つ）
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = マーカーは、移動するまで今の場所に残ります。
settings-markers-move-into-videos = マーカーをビデオに移動…
settings-markers-copy-into-comment = マーカーをコメントにコピー…
settings-markers-hint = 点も範囲も対象です。AI が見つけた瞬間も含みます。
settings-updates = アップデート
settings-version = バージョン
settings-release-notes = リリースノート
settings-old-title = 以前の frename の設定
settings-old-hint = zip フォルダーから実行していた frename の設定を引き継ぎます。
settings-old-scheduled = frename を再起動すると設定がインポートされます
settings-old-not-found = { $folder } に frename.db のある frename.exe がありません
settings-old-failed = インポートできませんでした: { $reason }
settings-old-import-button = 以前の frename フォルダーからインポート…
settings-ai = AI で説明
settings-ai-key-label = Anthropic API キー
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = console.anthropic.com → API keys でキーを取得してください。
settings-ai-key-remove-confirm = 保存した Anthropic キーを削除しますか？
settings-ai-used-by = Claude モデルを選んだとき、一括処理とマーカー（✨、Ctrl+F2）の「AI で説明」で使われます。
settings-ai-openai-key-label = OpenAI API キー
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = platform.openai.com/api-keys でキーを取得してください。
settings-ai-openai-key-remove-confirm = 保存した OpenAI キーを削除しますか？
settings-ai-openai-used-by = GPT モデルを選んだとき、一括処理とマーカー（✨、Ctrl+F2）の「AI で説明」で使われます。
settings-ai-gemini-key-label = Gemini API キー
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = aistudio.google.com/apikey でキーを取得してください。
settings-ai-gemini-key-remove-confirm = 保存した Gemini キーを削除しますか？
settings-ai-gemini-used-by = Gemini モデルを選んだとき、一括処理とマーカー（✨、Ctrl+F2）の「AI で説明」で使われます。
settings-ai-model-needs-key = { $model } — 先に { $provider } のキーを追加してください
settings-ai-model-label = モデル
settings-ai-language-label = 説明の言語
settings-ai-hint = Haiku、GPT-4.1 mini、Gemini Flash-Lite が最も低コストです。Sonnet、Opus、GPT-4.1、Gemini Flash はより細かい点まで拾います。モデルには、その提供元のキーが必要です。
settings-ai-moments-label = 瞬間
settings-ai-moments-important = 目立つ部分のみ
settings-ai-moments-full = クリップ全体
settings-ai-moments-important-hint = イン点とアウト点を設定する位置も提案します（IN と OUT の隣の AI ボタン）
settings-ai-moments-full-hint = 静かな部分も含め、すべての部分に 1 つずつ瞬間を付けます。イン点／アウト点の提案はありません
settings-ai-tag-suggestions = フォルダーのタグからもタグを提案する
settings-ai-tag-suggestions-hint = 同じリクエストで、フォルダーのどのタグが各クリップに合うかを AI が答えます。提案されたタグは、クリックしたときだけ追加されます。
settings-subtitles = 字幕
settings-subtitles-key-label = Soniox API キー
settings-subtitles-key-placeholder = キーを貼り付け
settings-subtitles-key-get = console.soniox.com でキーを取得してください。音声は Soniox に送信されます。
settings-subtitles-key-remove-confirm = 保存した Soniox キーを削除しますか？
settings-subtitles-languages-label = 言語
settings-subtitles-languages-locked = キーを保存すると、Soniox が対応するすべての言語から選べます。
settings-subtitles-languages-loading = Soniox から言語を取得しています…
settings-subtitles-languages-none = 未選択: 自動的に検出します。
settings-subtitles-languages-hint = 映像で話されている言語を、ヒントとして指定します。
settings-subtitles-cue-length-label = キューの長さ
settings-subtitles-cue-short = 短め
settings-subtitles-cue-short-hint = 1 行、最大 8 秒
settings-subtitles-cue-sentence = 1 文
settings-subtitles-hint = 一括処理の「字幕を生成」で使われます。キューの長さは新しい字幕に適用されます。
settings-key-store-windows = Windows 資格情報マネージャー
settings-key-store-macos = macOS キーチェーン
settings-key-store-other = システムのキーリング
settings-key-unavailable = システムのキーリングを開けませんでした
settings-key-unavailable-hint = ロックされているか、存在しない（GNOME Keyring や KWallet など）可能性があります。設定を開くたびに再確認します。
settings-key-remove-confirm-hint = もう一度貼り付ける必要があります。
settings-key-remove-ask = 削除…
settings-key-remove = キーを削除
settings-key-keep = 残す
settings-key-replace = 置き換え…
settings-key-hide = 隠す
settings-key-show = 表示
settings-key-save = キーを保存
settings-key-get = キーを取得
settings-key-billing = クレジットを追加
settings-key-cancel = キャンセル
settings-key-saved-in = このコンピューターの { $store } に保存済み
settings-key-save-into = 「キーを保存」で、このコンピューターの { $store } に保存します。

## 一括処理

batch-checked-count = { $count ->
   *[other] { $count } 個のファイルを選択中
}
batch-no-key = キーなし
batch-group-move = 場所間の移動
batch-group-fix = 名前とビデオの修正
batch-group-paid = 有料サービス
batch-reason-none-checked = ファイルが選択されていません
batch-reason-reading = クリップの長さを読み取り中…
batch-check-all = { $count ->
   *[other] { $count } 個のファイルをすべて選択
}
batch-job-running = 実行中
batch-job-finished = 完了
batch-job-stopped = 停止
batch-progress-files = { $finished } / { $total ->
   *[other] { $total } 個のファイル
}
batch-time-left = 残り約 { $time }
batch-time-estimating = 時間を見積もり中…
batch-time-spent = 経過 { $time }
batch-runs-in-background = 実行中も作業を続けられます
batch-clip-in-work = 使用中: { $action }
batch-folder-held = 処理中: { $action }
batch-indicator-hint = クリックでパネルとキャンセルを表示
batch-action-prepare = クリップを準備
batch-action-prepare-hint = 選択した各クリップを一度にまとめて処理します。字幕がなければ字幕（Soniox）を作り、続いて AI による説明（要約、マーカーとしての瞬間、コメント、タグの提案）を行います。料金、上限、進捗は全体で 1 つです。
batch-prepare-step-off = オフ
batch-prepare-nothing = これらのクリップでやることはありません
batch-prepare-report-cap = 上限 { $cap } のうち { $spent } を使用
batch-option-steps = ステップ
batch-prepare-step-subtitles = 先に字幕を作る（Soniox）
batch-prepare-step-description = AI でクリップを説明
batch-prepare-cap = 次の金額で停止
batch-prepare-cap-none = 上限なし（米ドル）
batch-prepare-cap-invalid = 上限はドルの数値で指定します（例: 5）
batch-prepare-left = 前回は { $count ->
   *[other] { $count } 個のクリップ
}を残して停止しました
batch-prepare-check-left = 選択する
batch-prepare-estimating = 各ステップの内容を確認中…
batch-prepare-plan-subtitles = 字幕
batch-prepare-plan-description = 説明
batch-prepare-plan-none = 処理なし
batch-prepare-plan-subtitles-line = { $videos ->
   *[other] { $videos } 本のビデオ
}を文字起こし、音声 { $duration }
batch-prepare-plan-description-line = { $videos ->
   *[other] { $videos } 本のビデオ
}を説明
batch-prepare-resume-note = 各ステップは済んでいる部分をスキップするので、停止した実行は再実行すれば続きから始まります。二重に課金されることはありません。
batch-prepare-run = { $clips ->
   *[other] { $clips } 個のクリップ
}を準備 · 約 { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = 似た映像
batch-prepare-groups-hint = 同じショットに見えるクリップ: 重複、再書き出し、カメラが動かなかったもの。
batch-prepare-group-clips = { $count } 個のクリップ
batch-prepare-check-group = これらのクリップを選択
batch-solo-queued = 待機中: { $action }
batch-suggest-in-job = このクリップの説明は、タグの提案なしで実行中のジョブに含まれています。終了後にもう一度お試しください
batch-suggest-no-tags = このフォルダーには提案元のタグがまだありません。先にいくつか追加してください
batch-solo-gave-up = 準備できませんでした
batch-solo-withdrawn = 取り下げました: { $action }
batch-solo-in-work = { $action } を実行中です。インジケーターまたはジョブのページからキャンセルしてください
batch-waiting-title = { $count ->
   *[other] { $count } 件の実行が待機中
}
batch-waiting-remove = 削除
batch-indicator-waiting = { $count } 件待機中
file-workspace-describe-clip = このクリップを AI で説明
file-workspace-subtitles-clip = このクリップの字幕を生成
file-workspace-describe-clip-queued = AI で説明: 順番待ち。クリックで取り下げ
file-workspace-describe-clip-running = AI で説明: 実行中
file-workspace-suggest-tags-clip = このクリップのタグを AI で提案（Ctrl+F6）
file-workspace-suggest-tags-clip-queued = タグを提案: 順番待ち。クリックで取り下げ
file-workspace-suggest-tags-clip-running = タグを提案: 実行中
file-workspace-subtitles-clip-queued = 字幕を生成: 順番待ち。クリックで取り下げ
file-workspace-subtitles-clip-running = 字幕を生成: 実行中
batch-run-again = { $count ->
   *[other] { $count } 個のファイル
}で再実行
batch-result-done = 完了: { $total ->
   *[other] { $total } 個のファイル
}
batch-result-done-detail = { $changed } 件を変更、{ $unchanged } 件は変更不要でした
batch-result-problems = 問題ありで完了: { $total ->
   *[other] { $total } 個のファイル
}のうち { $failed } 個が未処理
batch-figure-unchanged = 変更不要
batch-figure-not-done = 未処理
batch-figure-not-reached = 未到達
batch-files-not-done = 未処理のファイル
batch-table-file = ファイル
batch-table-why = 理由

batch-run-move-comments = { $count ->
   *[other] { $count } 件のコメント
}を移動
batch-run-move-in-out = { $count ->
   *[other] { $count } 個のファイル
}のイン点／アウト点を移動
batch-run-convert = { $count ->
   *[other] { $count } 個のファイル
}を変換
batch-run-rotate = { $count ->
   *[other] { $count } 本のビデオ
}を回転
batch-run-tag = { $count ->
   *[other] { $count } 個のファイル
}にタグ付け
batch-run-fix-tags = { $count ->
   *[other] { $count } 個の名前
}のタグを並べ替え
batch-run-rename = { $count ->
   *[other] { $count } 個のファイル
}の名前を変更
batch-run-reload = { $count ->
   *[other] { $count } 個のファイル
}を再読み込み
batch-run-export-markers = { $count ->
   *[other] { $count } 個のファイル
}のマーカーを書き出し
batch-service-soniox = Soniox
batch-change-renames = ファイル名を変更します
batch-change-videos = ビデオに書き込みます
batch-change-text-files = ビデオの隣にテキストファイルを書き込みます
batch-change-comments = コメントに書き込みます
batch-change-subtitles = ビデオの隣に字幕ファイルを書き込みます
batch-change-records = frename 自身の記録のみ変更します
batch-option-direction = 方向
batch-option-language = 言語
batch-option-tag-suggestions = タグの提案
batch-option-described = 説明済みのビデオ
batch-option-subtitled = 字幕付きのビデオ
batch-option-subtitles-write = 書き込むファイル
batch-option-spacing = スペース
batch-option-tag = タグ
batch-option-turn = 回転
batch-plan-videos = ビデオ
batch-plan-length = 長さ
batch-plan-cost = 料金
batch-plan-time = 時間
batch-reason-estimate = 見積もり待ち
batch-reason-subtitles-no-format = 書き込むファイルを 1 つ以上選択してください
batch-set-key = キーを設定…
batch-check-key = キーを確認…
batch-subtitles-languages-auto = ビデオごとに検出
batch-action-describe-ai-run-waiting = ビデオを説明
batch-action-fix-tags-order = 順序: タグリストのとおり。未登録のタグが先頭。
batch-action-markers-comment-hint-short = 時間のあるコメント行をマーカーにするか、マーカーをコメントにコピーします。
batch-action-markers-to-comment-hint = マーカーは残ります。もう一度実行しても二重には追加されません。
batch-action-rotate-hint-short = MP4 と MOV のクリップを回転フラグで回転します。再エンコードはしません。
batch-action-tag-commented-off = コメント付きビデオへのタグ付けはオフです
batch-action-tag-commented-choose = タグを選択…

batch-title = 一括処理
batch-done-label-changed = 変更
batch-done-label-subtitled = 字幕付き
batch-ai-at-least = 最低
batch-cancel = キャンセル
batch-stopping = 停止中…
batch-stop-over-budget = 停止しました: 次のクリップで上限 { $cap } を超える可能性があります（{ $spent } 使用済み）。残りを処理するには上限を上げてください。
batch-stopped = { $total ->
   *[other] { $total } 個のファイル
}のうち { $finished } 個の処理後に停止しました。
batch-close = 閉じる
batch-failed-subtitles = 字幕なし:
batch-written-subtitles = 書き込んだファイル:
batch-table-written = 書き込み
batch-merged-markers = 統合した重複マーカー:
batch-table-merged = 統合
batch-markers-merged = { $count ->
   *[other] { $count } 個の重複マーカーを統合しました
}
batch-add-credit = クレジットを追加
batch-open-log = ログを開く

batch-action-move-comments = コメントを移動
batch-action-move-comments-hint = 選択した各ファイルのコメントを、含まれているイン点／アウト点とともに指定の場所へ移動します。ビデオ内のタグとイン点／アウト点はそのまま残ります。
batch-action-move-comments-into-videos = テキストファイルからビデオへ（XMP）
batch-action-move-comments-into-text-files = ビデオ（XMP）からテキストファイルへ

batch-action-move-in-out = イン点／アウト点: コメント ⇄ ビデオ（XMP）
batch-action-move-in-out-hint = 選択した各ファイルのイン点／アウト点を指定の場所へ移動します。コメントのそれ以外の部分は元の場所に残ります。
batch-action-move-in-out-into-videos = コメントからビデオへ（Adobe XMP マーカー）
batch-action-move-in-out-into-comments = ビデオ（XMP マーカー）からコメントへ

batch-action-markers-comment = マーカー ⇄ コメント
batch-action-markers-comment-to-markers = 時間のあるコメント行をマーカーに
batch-action-markers-to-comment = マーカーをコメントに（コピー: マーカーは残ります）
batch-action-markers-comment-hint = 「03:24 — Take 3 — nice light」という行は、3:24 にある「Take 3」という名前のマーカーで、コメントは「nice light」になります。「0:41-0:47 — Lion」は 0:41 から 0:47 までのマーカーです。名前とコメントは最初の「 — 」または「 -- 」で区切られ、通常の「 - 」では区切られません。AI の説明の瞬間（「0:00–0:14 Street.」）は白いマーカーになり、説明にも残ります。どちらの向きでも、もう一度実行して二重に追加されることはありません。

batch-action-tag-commented = コメント付きビデオにタグ付け
batch-action-tag-commented-hint = 自分のコメントがある選択済みの各ビデオに「{ $tag }」タグを追加し（AI の説明は対象外）、コメントがないものからは削除します。タグが変わったファイルは名前が変更されます。
batch-action-tag-commented-hint-off = 自分のコメントがある選択済みの各ビデオに、コメント付きビデオ用のタグを追加し、コメントがないものからは削除します。このタグは設定でオフになっています。
batch-action-fix-tags = 優先度でタグを修正
batch-action-fix-tags-hint = 選択した各ファイル名のタグを、タグパネルの順に並べます。パネルで上にあるタグほど、名前の前の方に来ます。フォルダーにまだ登録されていないタグは、タグパネルと同様に先頭になります。順序が変わったファイルは名前が変更されます。

batch-action-respace-tags = タグのスペースを適用
batch-action-respace-tags-hint-space = 設定に従い、選択した各ファイルの名前を各タグの後にスペースが入るよう変更します: Food. Goat. clip.mp4。
batch-action-respace-tags-hint-no-space = 設定に従い、選択した各ファイルの名前をタグの後にスペースが入らないよう変更します: Food.Goat.clip.mp4。
batch-action-respace-tags-status-space = スペース: 各タグの後にスペースあり
batch-action-respace-tags-status-no-space = スペース: タグの後にスペースなし
batch-action-export-markers = マーカーをテキストに書き出し
batch-action-export-markers-hint = 選択した各ファイルのマーカーを、隣のテキストファイル clip.markers.txt に書き出します。マーカー 1 つにつき 1 行で、時間順です。例: 「0:41–0:47 [red] — Lion crossing — walks left to right」。緑以外の色は角括弧で示し、コメントは名前の後に続きます。「マーカー ⇄ コメント」は、このような行をマーカーとして読み戻します。マーカーのないファイルにはテキストファイルを作りません。前回のテキストファイルは置き換えられます。
batch-export-markers-cannot-hold = このファイルにはマーカーを保存できません
batch-action-reload-files = キャッシュをリセットして再読み込み
batch-action-reload-files-hint = 選択した各ファイルのコメントとイン点／アウト点をファイル自体から読み直し、フォルダーが記憶していた内容を置き換えます。別のプログラムでファイルを変更した後に使います。記憶している値がなかった、または古かったファイルは変更ありとして数えます。

batch-action-describe-ai = AI で説明
batch-action-describe-ai-run = { $videos }を説明 · 約 { $dollars }
batch-action-describe-ai-estimating = 見積もり中… { $known } / { $total }
batch-action-describe-ai-none = 説明するビデオがありません。
batch-action-describe-ai-hint = { batch-runs-in-background }。キャンセルしても説明済みのビデオは残り、再実行するとスキップされます。
batch-action-describe-ai-no-subtitles = 字幕なし（映像のみを説明）: { $videos }。
batch-action-describe-ai-redo = AI の説明がすでにあるビデオもやり直す
batch-action-describe-ai-hint-panel = 選択した各ビデオで何がいつ起こるかを説明します。要約と時間範囲付きのセグメントが、コメントの AI の説明に入ります。自分で書いたテキストは残ります。フレームと字幕は、選択したモデルの提供元である { $provider } に送信されます。
batch-ai-change = 変更
batch-ai-tag-suggestions-off = オフ
batch-ai-tag-suggestions-no-tags = オンですが、このフォルダーにはまだタグがありません
batch-ai-tag-suggestions-on = { $tags ->
   *[other] フォルダーの { $tags } 個のタグから
}
batch-ai-key-missing = 設定で { $provider } API キーを設定してください
batch-ai-key-unavailable = システムのキーリングを開けませんでした。ロックされているか、存在しない（GNOME Keyring や KWallet など）可能性があります。
## AI の説明の言語名: 設定のピッカーと上の { $language }。

ai-language-same-as-subtitles = 字幕と同じ
ai-language-english = 英語
ai-language-russian = ロシア語
ai-language-ukrainian = ウクライナ語
ai-language-german = ドイツ語
ai-language-spanish = スペイン語
ai-language-french = フランス語
batch-videos-count = { $n ->
   *[other] { $n } 本のビデオ
}
batch-minutes = { $n } 分
batch-ai-minutes-under = 1 分未満
batch-ai-duration-under-minute = 1 分未満
batch-ai-duration-hours = { $h } 時間
batch-ai-duration-hours-minutes = { $h } 時間 { $m } 分
batch-ai-dollars-under = $0.01 未満
batch-ai-skip-described = 説明済み
batch-ai-skip-too-long = 30 分超
batch-ai-skip-unreadable = 読み取り不可
batch-ai-skipped = スキップ: { $parts }。
batch-ai-progress-frame = フレーム { $done } / { $total }
batch-ai-progress-waiting = { $provider } の応答待ち
batch-ai-progress-saving = 保存中
batch-ai-progress-retry-rate-limit = { $seconds } 秒後に再試行（レート制限）
batch-ai-progress-retry-temporary = { $seconds } 秒後に再試行（一時的な障害）
batch-ai-stop-no-key = 停止しました: { $provider } API キーがありません。設定で入力してください。
batch-ai-fail-no-key = API キーがありません
batch-ai-fail-unreadable = ビデオを読み取れませんでした
batch-ai-fail-too-long = クリップが AI には長すぎます（30 分超）
batch-ai-fail-not-saved = 説明を保存できませんでした
batch-ai-stop-offline = 停止しました: { $provider } に接続できません。もう一度実行すると、残りを説明します。

batch-action-generate-subtitles = 字幕を生成
batch-action-generate-subtitles-install-ffmpeg = .mkv、.m2ts、.avi などを読み込むには、ffmpeg.org から ffmpeg をインストールして PATH に追加し、frename を再起動してください
batch-action-generate-subtitles-replace = 既存の字幕を置き換える
batch-action-generate-subtitles-replace-hint = 文字起こしをやり直します。表示のとおり課金されます。
batch-action-generate-subtitles-srt = SRT 字幕（clip.srt）
batch-action-generate-subtitles-premiere = Premiere Pro の文字起こし（clip.premiere.json）
batch-action-generate-subtitles-premiere-hint = Premiere Pro で: テキストパネル → 文字起こし → 静的な文字起こしを読み込み。
batch-action-generate-subtitles-privacy = これらのビデオの音声は Soniox に送信され、その後そこで削除されます。
batch-action-generate-subtitles-duration-hint = 音声 1 時間あたり数分かかります。{ batch-runs-in-background }。frename を閉じると実行が止まり、完成した字幕は残ります。
batch-action-generate-subtitles-hint = 選択した各ビデオの音声を Soniox で文字起こしし、隣に保存します。frename が表示する字幕（clip.srt）や、Premiere Pro の文字起こしを作成できます。
batch-subtitles-transcribe = 文字起こし
batch-subtitles-transcribe-count = { $videos }を文字起こし
batch-subtitles-build-free = { $count } 件を作成（無料）
batch-subtitles-nothing = 文字起こしするものがありません
batch-subtitles-estimating = 見積もり中…
batch-subtitles-key-missing = 設定で Soniox API キーを設定してください
batch-subtitles-key-rejected = Soniox がキーを拒否しました
batch-subtitles-cost-unknown = 料金不明
batch-subtitles-typical-price = { $usd }（標準的な料金）
batch-subtitles-unknown-length = + 長さ不明 { $n } 件
batch-subtitles-plan-line = { $videos }を文字起こし、音声 { $duration }{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
   *[other] { $n } 件はすでに字幕あり
}
batch-subtitles-rebuilt-free = { $n } 件は保存済みの文字起こしから無料で再作成
batch-subtitles-no-speech-before = { $n } 件は前回、発話が検出されませんでした
batch-subtitles-no-audio = { $n ->
   *[other] { $n } 件は音声なし
}
batch-subtitles-shared-name = { $n ->
   *[other] { $n } 件は字幕名が別のビデオと重複
}
batch-subtitles-unreadable = { $n } 件は読み取り不可（{ $how_to }）
batch-subtitles-shared-name-reason = 字幕名が別のビデオと重複
batch-subtitles-unsupported-reason = 音声形式は非対応
batch-subtitles-progress-transcribing = 文字起こし中…
batch-subtitles-progress-extracting = 音声を抽出中 { $percent } %
batch-subtitles-progress-extracting-unknown = 音声を抽出中
batch-subtitles-progress-uploading = アップロード中 { $percent } %
batch-subtitles-progress-uploading-unknown = アップロード中
batch-subtitles-progress-transcribing-left = 文字起こし中… 残り約 { $time }
batch-subtitles-progress-saving = 保存中
batch-subtitles-reason-already = すでに字幕あり
batch-subtitles-reason-no-audio = 音声なし
batch-subtitles-reason-no-speech = 発話なし
batch-subtitles-reason-soniox = Soniox: { $message }
batch-subtitles-reason-unreachable = Soniox に接続できません
batch-subtitles-stop-key-rejected = 停止しました: Soniox がキーを拒否しました。設定 → 字幕で確認してください。
batch-subtitles-stop-balance-empty = 停止しました: Soniox の残高がありません。console.soniox.com でチャージしてください。
batch-subtitles-stop-budget-used = 停止しました: Soniox の月間予算を使い切りました。console.soniox.com で引き上げてください。
batch-subtitles-stop-other = 停止しました: Soniox が処理の続行を拒否しました（{ $error_type }）。
batch-subtitles-stop-unreachable = 停止しました: Soniox に接続できません。インターネット接続を確認してください。
batch-subtitles-report-spend = Soniox: 最低 { $duration } · { $usd }
batch-subtitles-report-failed-deletes = Soniox 上の { $n ->
   *[other] { $n } 件のアップロード
}を削除できませんでした（ログを参照）
batch-subtitles-languages-need-key = Soniox キーを保存すると、すべての言語から選べます。
batch-subtitles-languages-rejected = Soniox がキーを拒否しました。
batch-subtitles-languages-failed = Soniox から言語を取得できませんでした。
batch-subtitles-count = { $n ->
   *[other] { $n } 件の字幕
}
batch-subtitles-duration-seconds = { $s } 秒
batch-subtitles-usd-under = $0.01 未満
batch-subtitles-usd-about = 約 ${ $amount }

## ファイルリスト
folder-has-subtitles = 字幕あり
folder-closed-by-job = 一括処理による変更中は閉じられています
folder-filter-tip = 絞り込み…

folder-search-placeholder = ファイルまたはコメントを検索
folder-search-comments-loading = コメントを検索中… 残り { $n }
folder-search-clear = クリア
folder-all = すべて
folder-invert = 反転
folder-checked = { $count } 個選択中
folder-outcome-changed = 変更あり
folder-outcome-unchanged = 変更不要
folder-outcome-failed = 失敗（ログを参照）
folder-rename-error-empty = 名前が空です
folder-rename-error-bad-character = 使用できない文字: \ / : * ? " < > |
folder-rename-error-trailing = 末尾にピリオドやスペースは使えません
folder-rename-error-exists = 同じ名前のファイルがあります
folder-markers-not-saved = マーカーを保存できませんでした: ファイルが読み取り専用か使用中です（Premiere で閉じてから、このファイルを開いてから別のファイルへ移動してください）
drag-out-not-saved = ドラッグできませんでした: ファイルを保存できません（読み取り専用、または Premiere で開いています）
folder-opening = フォルダーを開いています…
folder-empty-title = このフォルダーにビデオはありません
folder-empty-line = frename は MP4、MOV、MKV などのビデオファイルを表示します。
folder-open-another = 別のフォルダーを開く…
folder-no-match = 一致するファイルがありません
folder-show-all = すべて表示
folder-locked = { $action } の実行中は選択を変更できません
folder-checked-hidden = { $count } 個選択中 · { $hidden } 個非表示
folder-checked-hidden-tip = { $hidden ->
   *[other] 選択中の { $hidden } 個のファイルが検索またはフィルターで非表示になっています。アクションはそれらにも実行されます
}
folder-outcome-working = 処理中
folder-outcome-not-reached = 未到達
folder-window-empty-title = クリップのフォルダーを開く
folder-window-empty-line = またはフォルダーをウィンドウにドロップします。ボタンを右クリックすると 1 つのファイルを開きます。
folder-window-open = フォルダーを開く…

## 最近使ったフォルダー（開くボタンの横の矢印と、空の画面）

recent-folders-tip = 最近使ったフォルダー
recent-folders-title = 最近使ったフォルダー
recent-folders-none = まだフォルダーを開いていません
recent-folders-not-found = 見つかりません
recent-folders-remove-tip = リストから削除
recent-folders-clear = リストを消去
recent-folders-remove = 削除
recent-folders-keep = 残す
recent-folders-missing-line = 見つかりません。削除しますか？
recent-folders-just-now = たった今
recent-folders-minutes = { $count } 分前
recent-folders-hours = { $count } 時間前
recent-folders-days = { $count ->
   *[other] { $count } 日前
}
recent-folders-weeks = { $count ->
   *[other] { $count } 週間前
}
recent-folders-months = { $count ->
   *[other] { $count } か月前
}
recent-folders-years = { $count ->
   *[other] { $count } 年前
}

## ファイルメニュー（ファイルを右クリック、または F11 / Shift+F11 / Ctrl+F11）

file-menu-show-in-explorer = エクスプローラーで表示
file-menu-show-in-file-manager = ファイルマネージャーで表示
file-menu-copy-path = フルパスをコピー
file-menu-copy-name = ファイル名をコピー
file-menu-copied = コピーしました
file-menu-not-copied = コピーできませんでした: クリップボードを利用できません
file-menu-not-shown = ファイルマネージャーを開けませんでした

## ファイルリスト下のコントロールバー

folder-controls-filter-untagged = タグなし
folder-controls-filter-subtitles = 字幕
folder-controls-filter-comments = コメント
folder-controls-filter-markers = マーカー
folder-controls-scroll = 開いているファイルをリストに表示
folder-controls-open = フォルダーを開く
folder-controls-batch = 選択したファイルの一括処理
folder-controls-batch-back = 開いているファイルに戻る
folder-controls-update-available = アップデートあり: { $version }
folder-controls-whats-new = { $version } の新機能
folder-controls-previous = 前のファイル
folder-controls-next = 次のファイル
folder-controls-more = その他
folder-controls-open-file = 右クリック: 1 つのファイルを開く

## ビデオ

video-controls-back = 10 秒戻る
video-controls-play = 再生
video-controls-pause = 一時停止
video-controls-forward = 10 秒進む
video-controls-frame-back = 1 フレーム戻る
video-controls-frame-forward = 1 フレーム進む
video-controls-set-in = イン点を設定
video-controls-set-out = アウト点を設定
video-controls-screenshot = このフレームを保存
video-controls-add-marker = マーカーを追加
video-controls-add-marker-hold = 長押しで範囲指定、もう一度押して名前を付ける
video-controls-cannot-hold-markers = このファイルにはマーカーを保存できません
video-controls-add-a-name = 名前を追加
video-controls-rotate-left = 左に回転
video-controls-rotate-right = 右に回転
video-controls-more = その他
media-viewer-loading-slow = ファイルを待っています…（クラウド上のファイルは時間がかかることがあります）
media-viewer-cannot-play = このクリップは再生できません
media-viewer-no-picture = このファイルには映像がありません
video-timecode-copied = タイムコードをコピーしました: { $time }
video-timecode-unreadable = 「{ $text }」はこのクリップの時間ではありません
video-controls-volume-scroll = 音量 — スクロールで変更
rotate-cannot = 回転できません: { $reason }
rotate-reason-missing = ファイルが見つかりません
rotate-flag-right = 右 90°
rotate-flag-left = 左 90°
rotate-flag-half = 180°
rotate-flag-none = なし
rotate-now = 回転: { $flag }
rotate-turned = { $turn } 回転 · 現在の回転: { $flag }
rotate-failed = 回転できませんでした: { $reason }
undo-name-taken = 変更しませんでした: すでに別のファイルが { $name } という名前です
undo-back-on-clip = このクリップに戻りました（移動を元に戻しました）。もう一度 Ctrl+Z で直前の変更を元に戻します。
redo-on-clip = このクリップに再び移動しました（移動をやり直しました）。
rotate-reason-in-use = ファイルが読み取り専用か使用中です
rotate-reason-format = この形式には回転フラグがありません
rotate-reason-damaged = ファイルが破損しています
rotate-reason-no-video = ファイルにビデオトラックがありません
rotate-reason-matrix = ビデオの表示マトリックスが特殊です
batch-action-rotate = ビデオを回転
batch-action-rotate-hint = 選択した各 MP4 と MOV ファイルの回転フラグを変更します。映像は再エンコードされず、コメント、イン点／アウト点、マーカーは残ります。Premiere Pro は読み込み時に、回転したクリップとして表示する想定です。ほかの形式には回転フラグがないため失敗します。再実行すると、さらに回転します。「リセット」は、スマートフォンが記録したものを含め、すべての回転を取り除くため、スマートフォンの縦向きクリップは横倒しで再生されます。
batch-action-rotate-right = 右 90°（時計回り）
batch-action-rotate-left = 左 90°（反時計回り）
batch-action-rotate-half = 180°
batch-action-rotate-reset = リセット: 回転なし（0°）
media-viewer-video-subtitle-list = 字幕リスト
media-viewer-video-marker-list = マーカーリスト
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 でマーカー間を移動、Shift+ドラッグでスナップ
media-viewer-video-tab-markers = マーカー
media-viewer-video-fullscreen = フルスクリーン
media-viewer-video-close-list = リストを閉じる
media-viewer-video-subtitle-file-unreadable = このファイルを読み取れませんでした
# クリップを開いたとき、前回再生が止まった位置から再開すると映像の上に表示されます。Home またはクリックで先頭に移動します。
media-viewer-video-resumed = { $time } から再開 · Home: 最初から

## マーカーリスト
markers-in-out = イン点／アウト点

markers-empty = マーカーはまだありません
markers-add = マーカーを追加
markers-ai-hint = AI マーカー: AI がこのクリップを再度説明すると置き換えられます
markers-keep-color = 色を保持
markers-done = 完了
markers-delete = マーカーを削除
markers-cannot-hold-hint = Premiere は MP4 と MOV ファイルからマーカーを読み込みます。
markers-color-green = グリーン
markers-color-red = レッド
markers-color-orange = オレンジ
markers-color-yellow = イエロー
markers-color-white = ホワイト
markers-color-blue = ブルー
markers-color-cyan = シアン
markers-color-lavender = ラベンダー
markers-color-magenta = マゼンタ
markers-color-other = その他の色
markers-read-only = 読み取り専用
markers-name-placeholder = 名前
markers-ai-describe = AI で説明: マーカーに名前を付け、起きていることを追加
markers-ai-stop = 説明を停止
markers-ai-describe-unnamed = 名前のない { $count } 個を説明
markers-ai-describe-unnamed-hint = 名前のないマーカーすべてに、AI で名前を付けて説明します。数個ずつ処理し、リクエストはマーカー 1 つにつき 1 回です。その間に入力した名前は残ります。
markers-ai-price-about = 約 { $dollars }
markers-ai-describe-unnamed-cost = 1 つにつき 1 リクエスト: マーカー 1 つ { $each }、全部で { $total }。
markers-ai-confirm = { $count ->
   *[other] { $count } 個のマーカーを説明しますか？ 1 つにつき 1 リクエスト: 合計 { $total }。
}
markers-ai-confirm-unpriced = { $count ->
   *[other] { $count } 個のマーカーを説明しますか？ 1 つにつき 1 回の有料リクエストです。
}
markers-ai-confirm-yes = 説明する
markers-ai-confirm-no = キャンセル
markers-ai-waiting = 順番待ち…
markers-ai-run = { $describing } 個を説明中 · { $waiting } 個待機中
markers-ai-run-last = { $describing } 個を説明中
markers-ai-stop-all = すべて停止
markers-ai-describing = 説明中…
markers-ai-done = マーカーを説明しました
markers-ai-nothing-new = このマーカーにはすでに同じ説明があります
markers-no-marker-notice = ここにマーカーはありません
markers-ai-no-key = { $provider } API キーがありません。設定で入力してください
markers-ai-failed = 説明できませんでした: { $reason }
markers-ai-failed-unknown = 問題が発生しました
markers-read-only-notice = このマーカーは読み取り専用です
markers-ai-gone = マーカーがなくなったため、説明は追加されませんでした
markers-ai-stopping-for-batch = 先にマーカーの説明を停止しています… 通常は数秒かかります。
folder-not-saved-marker-ai = 保存できませんでした: マーカーの説明がまだクリップを読み取っていました。少ししてからもう一度お試しください。

## ファイルワークスペース

file-workspace-search-placeholder = タグを検索 — または入力するだけ
file-workspace-search-clear = クリア
file-workspace-comment-placeholder = コメント...
file-workspace-comment-collapse = タグに戻る
file-workspace-comment-expand = コメントを展開

## タググリッド

tag-grid-star = スター: 先頭に固定
tag-grid-unstar = スターを外す
tag-grid-save = フォルダーのタグに追加
tag-grid-delete = 「{ $tag }」をフォルダーのタグから削除
tag-grid-create = 「{ $tag }」を作成
tag-grid-no-file = クリップを開いてタグを付けます
tag-grid-no-tags = タグはまだありません
tag-grid-no-tags-hint = 名前を入力して Enter キーを押すと、最初のタグを作成します。
tag-grid-group-unsaved = フォルダーのタグにないもの
tag-grid-group-folder = フォルダーのタグ
tag-grid-more = ほか { $count } 個

## 並び順ストリップ

sync-panel-locked = 下での並べ替えはフォルダー全体に適用されます
sync-panel-unlocked = 下での並べ替えはこのクリップのみに適用されます
sync-panel-unlock = ロック解除
sync-panel-lock = ロック
sync-panel-differs = フォルダーと順序が異なります
sync-panel-use-for-folder = フォルダーに適用
sync-panel-sort-like-folder = フォルダーと同じ順にする

## ファイル名カード

file-name-panel-no-tags = このクリップにタグはありません
file-name-panel-untag = タグを外す
file-name-panel-clear-in = イン点をクリア
file-name-panel-clear-out = アウト点をクリア
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = イン点とアウト点を AI の提案に設定（秒単位に丸め）
file-name-panel-suggested-tags = AI のタグ
file-name-panel-add-suggested-tag = タグ「{ $tag }」を追加（確度 { $percent }%）
file-name-panel-add-suggested-tag-unsure = タグ「{ $tag }」を追加
file-name-panel-add-all-suggested-tags = すべて追加
file-name-panel-add-all-suggested-tags-tip = AI が提案したタグをすべて追加
file-name-panel-tag-ideas = 候補: { $ideas }
file-name-panel-tag-ideas-tip = AI が気づいた、このフォルダーにないタグです。追加はされません。
file-name-panel-saved = すべての変更を保存しました
file-name-panel-saved-to-recovery = 復元用に保存しました。クリップを離れるときに適用されます

## アップデート（設定）

updates-check = アップデートを確認
updates-not-installed = アップデートはインストール版で利用できます
updates-from-store = アップデートは Microsoft Store から提供されます
updates-from-releases = 新しいバージョン: リリースページからダウンロードしてください
updates-open-releases = リリースページを開く
updates-checking = 確認中…
updates-downloading-named = { $version } をダウンロード中… { $percent }%
updates-downloading = ダウンロード中… { $percent }%
updates-restarting = 再起動中…
updates-check-failed = アップデートを確認できませんでした: { $reason }
updates-update-failed = アップデートできませんでした: { $reason }
updates-close-other-window = 先にもう 1 つの frename ウィンドウを閉じてください
updates-reason-github = GitHub が応答しません。後でもう一度お試しください
updates-reason-no-connection = 接続がありません
updates-reason-read-only = フォルダーが読み取り専用です
updates-reason-not-installed = アップデートはインストール版で利用できます
updates-reason-download-stopped = ダウンロードが予期せず停止しました
updates-version-available = バージョン { $version } が利用できます
updates-up-to-date = frename は最新です
updates-update-and-restart = アップデートして再起動
updates-wait-for-batch = 一括処理の完了をお待ちください
updates-current-version = frename { $version }
updates-check-on-start = frename の起動時にアップデートを確認する
updates-notes-show = リリースノート
updates-notes-hide = リリースノートを隠す
updates-notes-all-releases = GitHub のすべてのリリース
updates-notes-history = すべてのバージョン（新しい順）
updates-notes-update = { $version } の変更点
updates-notes-whats-new = { $version } の新機能
updates-notes-loading = リリースノートを読み込み中…
updates-notes-unavailable = 最新バージョンのノートを読み込めませんでした。
updates-notes-more = さらに表示

recovery-restored = frename が予期せず終了しました。{ $clip } の未保存の作業（{ $what }）を復元しました。
recovery-kept = frename が予期せず終了しました。{ $clip } の未保存の作業（{ $what }）は適用されませんでした: { $why }。入力し直すための読みやすいコピーが { $folder } にあります。
recovery-unreadable = frename が予期せず終了し、復元ファイルを読み取れませんでした。ファイルは { $folder } に保管されています。
recovery-why-gone = クリップがなくなったか、名前が変更されました
recovery-why-changed = 編集後にクリップが変更されました
recovery-why-not-written = 同じ名前のファイルがあるか、クリップが読み取り専用または使用中です
recovery-tags = { $count ->
   *[other] { $count } 個のタグ
}
recovery-markers = { $count ->
   *[other] { $count } 個のマーカー
}
recovery-comment = コメント
recovery-in-out = イン点／アウト点
recovery-edits = 編集
folder-frame-saved = フレームを保存しました
folder-not-saved = 保存できませんでした: 同じ名前のファイルがすでにあるか、読み取り専用または使用中です
markers-deleted-notice = マーカーを削除しました
markers-not-saved-notice = マーカーを保存できませんでした: ファイルが読み取り専用か使用中です
markers-comment-not-saved-notice = マーカーを保存できませんでした: コメントファイルが読み取り専用か使用中です
markers-copy = コピー
markers-copy-tip = マーカーを 1 行ずつテキストとしてコピー
markers-export = 書き出し…
markers-export-tip = マーカーをテキストファイルとして保存
markers-exported = マーカーを { $name } に保存しました
markers-not-exported = マーカーを保存できませんでした: ファイルに書き込めません

batch-option-notes = AI へのメモ
batch-ai-notes-info = このフォルダーの映像についての自分用のメモです。説明、タグの提案、マーカーの説明など、ここでのすべての AI リクエストに添えて送信されます。フォルダーの .frename ファイルに保存されます。
batch-ai-notes-placeholder = 例: 一番よく映る男性はユージーンです。広告看板にマーカーを付けてください。お茶が映ったらタグ tea を提案してください。
batch-ai-notes-hint = フレームとともに { $provider } に送信されます。名前はこのメモからのみ付けられ、frename は顔を認識しません。説明済みのビデオはスキップされます。新しいメモでやり直すには「AI の説明がすでにあるビデオもやり直す」にチェックを入れてください。
batch-ai-notes-too-long = メモが { $chars } 文字あり、上限は { $max } 文字です。実行するには短くしてください。
batch-ai-notes-too-long-reason = AI へのメモが長すぎます
batch-option-notes-once = 今回のみの追記
batch-ai-notes-once-placeholder = 例: このクリップ: 赤い服の女性はアナです
file-workspace-ai-notes-on = メモあり
file-workspace-ai-notes-tip = AI へのメモは、このフォルダーのすべての AI リクエストに添えられます。「AI で説明」（一括処理）で編集できます: { $notes }
batch-ai-notes-no-subtitles = メモは話されている内容についてのようですが、{ $videos }に字幕がありません。AI が言葉を読み取れるよう、先に字幕を生成してください（字幕を生成）。
