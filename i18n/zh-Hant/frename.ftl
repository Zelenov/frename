# frename 介面文字，來源語言。i18n/ 底下其他每個檔案的 id、
# 屬性與 $參數都與此檔完全相同（由 src/i18n.rs 的測試檢查）。
# 鍵名：<功能>-<元素>[-<細節>]，功能即 src/features/ 底下的資料夾。

## 語言名稱：每個檔案都相同，各以該語言自己的文字書寫。

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

## 設定視窗

settings-window-title = 設定
settings-page-interface = 介面
settings-page-saving = 儲存
settings-page-ai = 用 AI 描述
settings-close = 關閉
settings-apply-note = 變更會立即生效。Tab 在選項之間移動，Ctrl+Tab 在頁面之間切換。
settings-language = 語言
settings-language-system = 系統（{ $language }）
settings-video = 影片
settings-video-autoplay = 開啟影片時自動播放
settings-subtitle-size = 字幕大小
settings-subtitle-size-small = 小
settings-subtitle-size-medium = 中
settings-subtitle-size-large = 大
settings-subtitle-size-extra-large = 特大
settings-timecode = 時間顯示
settings-timecode-hint = 影片下方、標記與字幕清單、入點／出點標籤上的時間寫法，以及 Ctrl+Shift+C 複製的播放頭時間。Premiere Pro 與 DaVinci Resolve 以片段的影格率顯示時:分:秒:影格；29.97 與 59.94 fps 為捨棄影格。影格率不固定的片段沿用預設樣式。檔案本身的格式不變。
settings-timecode-default = 預設（mm:ss）
settings-timecode-premiere = Premiere Pro（HH:MM:SS:FF）
settings-timecode-resolve = DaVinci Resolve（HH:MM:SS:FF）
settings-tags = 標籤顏色
settings-tags-monochrome = 單色
settings-tags-monochrome-hint = 你新增的標籤以較深的灰色顯示，其他的以較淺的灰色顯示，不使用其他顏色。
settings-file-names = 檔名
settings-tags-space-after = 每個標籤後加空格
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = 檔案在重新命名或儲存之前，名稱維持不變。
settings-tags-space-add = 在檔名中加入空格…
settings-tags-space-remove = 從檔名中移除空格…
settings-comments = 註解
settings-comments-in-video = 寫在影片檔內
settings-comments-in-video-hint = XMP，即 Premiere Pro 的「說明」欄
settings-comments-text-file = 寫在影片旁的文字檔
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = 在移動之前，檔案的註解維持在原處。
settings-comments-move-into-videos = 將註解移入影片…
settings-comments-move-into-text-files = 將註解移入文字檔…
settings-commented-tag = 為有註解的影片加標籤
settings-commented-tag-name = 標籤
settings-commented-tag-hint = 在影片上寫下註解時會勾選，例如 { $tag }.IMG_0424.MOV；清除註解時取消勾選（AI 描述不算）。其餘時候可自行更改。
settings-commented-tag-off = 有註解的影片不加標籤。
settings-in-out = 入點／出點
settings-in-out-in-video = 寫在影片檔內
settings-in-out-in-video-hint = Premiere Pro 中的子片段標記
settings-in-out-comment = 寫在註解中，合為一行
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = 在移動之前，檔案的入點／出點維持在原處。
settings-in-out-move-into-videos = 將入點／出點移入影片…
settings-in-out-move-into-comments = 將入點／出點移入註解…
settings-markers = 標記與範圍
settings-markers-in-video = 寫在影片檔內
settings-markers-in-video-hint = XMP，顯示在 Premiere Pro 的片段上
settings-markers-comment = 寫在註解中，每個一行
settings-markers-comment-example = 0:41–0:47 — 獅子
settings-markers-note = 在移動之前，檔案的標記維持在原處。
settings-markers-move-into-videos = 將標記移入影片…
settings-markers-copy-into-comment = 將標記複製到註解…
settings-markers-hint = 單點與範圍標記皆適用，AI 找到的時刻也一樣。
settings-updates = 更新
settings-version = 版本
settings-release-notes = 版本資訊
settings-old-title = 舊版 frename 的設定
settings-old-hint = 匯入你以 zip 資料夾執行的舊版 frename 的設定。
settings-old-scheduled = frename 重新啟動時將匯入設定
settings-old-not-found = { $folder } 中沒有附 frename.db 的 frename.exe
settings-old-failed = 無法匯入：{ $reason }
settings-old-import-button = 從舊版 frename 資料夾匯入…
settings-ai = 用 AI 描述
settings-ai-key-label = Anthropic API 金鑰
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = 前往 console.anthropic.com → API keys 取得金鑰。
settings-ai-key-remove-confirm = 要移除已儲存的 Anthropic 金鑰嗎？
settings-ai-used-by = 選用 Claude 模型時，批次模式與標記上的「用 AI 描述」（✨、Ctrl+F2）會使用此金鑰。
settings-ai-openai-key-label = OpenAI API 金鑰
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = 前往 platform.openai.com/api-keys 取得金鑰。
settings-ai-openai-key-remove-confirm = 要移除已儲存的 OpenAI 金鑰嗎？
settings-ai-openai-used-by = 選用 GPT 模型時，批次模式與標記上的「用 AI 描述」（✨、Ctrl+F2）會使用此金鑰。
settings-ai-gemini-key-label = Gemini API 金鑰
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = 前往 aistudio.google.com/apikey 取得金鑰。
settings-ai-gemini-key-remove-confirm = 要移除已儲存的 Gemini 金鑰嗎？
settings-ai-gemini-used-by = 選用 Gemini 模型時，批次模式與標記上的「用 AI 描述」（✨、Ctrl+F2）會使用此金鑰。
settings-ai-model-needs-key = { $model } — 請先新增 { $provider } 金鑰
settings-ai-model-label = 模型
settings-ai-language-label = 描述語言
settings-ai-hint = Haiku、GPT-4.1 mini 與 Gemini Flash-Lite 最便宜；Sonnet、Opus、GPT-4.1 與 Gemini Flash 觀察得更細。模型需要其所屬公司的金鑰。
settings-ai-moments-label = 時刻
settings-ai-moments-important = 只標出突出的部分
settings-ai-moments-full = 涵蓋整個片段
settings-ai-moments-important-hint = 同時建議入點與出點的位置（IN 與 OUT 旁的 AI 按鈕）
settings-ai-moments-full-hint = 每一段都標出時刻，安靜的部分也一樣；不建議入點與出點
settings-ai-tag-suggestions = 同時依資料夾的標籤建議標籤
settings-ai-tag-suggestions-hint = 在同一次請求中，AI 會指出資料夾中的哪些標籤適合每個片段。建議的標籤要等你點選才會加入。
settings-subtitles = 字幕
settings-subtitles-key-label = Soniox API 金鑰
settings-subtitles-key-placeholder = 貼上金鑰
settings-subtitles-key-get = 前往 console.soniox.com 取得金鑰。音訊會傳送給 Soniox。
settings-subtitles-key-remove-confirm = 要移除已儲存的 Soniox 金鑰嗎？
settings-subtitles-languages-label = 語言
settings-subtitles-languages-locked = 儲存金鑰後，即可從 Soniox 支援的所有語言中選擇。
settings-subtitles-languages-loading = 正在從 Soniox 取得語言…
settings-subtitles-languages-none = 未勾選：自動偵測。
settings-subtitles-languages-hint = 素材中說的語言，作為提示。
settings-subtitles-cue-length-label = 字幕長度
settings-subtitles-cue-short = 短
settings-subtitles-cue-short-hint = 一行，最長 8 秒
settings-subtitles-cue-sentence = 一句話
settings-subtitles-hint = 批次模式的「產生字幕」會使用。字幕長度套用於新的字幕。
settings-key-store-windows = Windows 認證管理員
settings-key-store-macos = macOS 鑰匙圈
settings-key-store-other = 系統金鑰環
settings-key-unavailable = 無法開啟系統金鑰環
settings-key-unavailable-hint = 可能已鎖定，或根本沒有（例如 GNOME Keyring 或 KWallet）。每次開啟設定時都會重新檢查。
settings-key-remove-confirm-hint = 之後需要重新貼上。
settings-key-remove-ask = 移除…
settings-key-remove = 移除金鑰
settings-key-keep = 保留
settings-key-replace = 取代…
settings-key-hide = 隱藏
settings-key-show = 顯示
settings-key-save = 儲存金鑰
settings-key-get = 取得金鑰
settings-key-billing = 儲值
settings-key-cancel = 取消
settings-key-saved-in = 已儲存於這台電腦的{ $store }
settings-key-save-into = 「儲存金鑰」會將金鑰存入這台電腦的{ $store }。

## 批次模式

batch-checked-count = { $count ->
   *[other] 已勾選 { $count } 個檔案
}
batch-no-key = 沒有金鑰
batch-group-move = 在不同位置之間移動
batch-group-fix = 修正名稱與影片
batch-group-paid = 付費服務
batch-reason-none-checked = 未勾選任何檔案
batch-reason-reading = 正在讀取片段長度…
batch-check-all = 全選 { $count ->
   *[other] { $count } 個檔案
}
batch-job-running = 執行中
batch-job-finished = 已完成
batch-job-stopped = 已停止
batch-progress-files = { $finished } / { $total ->
   *[other] { $total } 個檔案
}
batch-time-left = 約剩 { $time }
batch-time-estimating = 正在估算時間…
batch-time-spent = 已耗時 { $time }
batch-runs-in-background = 執行期間你可以繼續工作
batch-clip-in-work = 使用中：{ $action }
batch-folder-held = 忙碌中：{ $action }
batch-indicator-hint = 點一下開啟面板與取消
batch-action-prepare = 準備片段
batch-action-prepare-hint = 對每個勾選的片段一次完成：沒有字幕就先產生字幕（Soniox），再用 AI 描述：摘要、以標記標出的時刻、註解與標籤建議。全部共用一個價格、一個上限與一個進度。
batch-prepare-step-off = 已關閉
batch-prepare-nothing = 這些片段沒有需要做的事
batch-prepare-report-cap = 已花費 { $spent }，上限為 { $cap }
batch-option-steps = 步驟
batch-prepare-step-subtitles = 先產生字幕（Soniox）
batch-prepare-step-description = 用 AI 描述片段
batch-prepare-cap = 花費達到此金額即停止
batch-prepare-cap-none = 不設上限（美元）
batch-prepare-cap-invalid = 上限是美元金額，例如 5
batch-prepare-left = 上次執行停止時，還剩 { $count ->
   *[other] { $count } 個片段
} 未處理
batch-prepare-check-left = 勾選這些片段
batch-prepare-estimating = 正在評估每個步驟的內容…
batch-prepare-plan-subtitles = 字幕
batch-prepare-plan-description = 描述
batch-prepare-plan-none = 沒有需要做的事
batch-prepare-plan-subtitles-line = { $videos ->
   *[other] { $videos } 部影片
} 待轉錄，音訊共 { $duration }
batch-prepare-plan-description-line = { $videos ->
   *[other] { $videos } 部影片
} 待描述
batch-prepare-resume-note = 每個步驟都會略過已完成的部分，所以停止的執行只要再跑一次就能接續：不會重複付費。
batch-prepare-run = 準備 { $clips ->
   *[other] { $clips } 個片段
} · 約 { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = 相似的素材
batch-prepare-groups-hint = 看起來是同一個鏡頭的片段：重複的檔案、重新匯出的檔案、沒有移動的攝影機。
batch-prepare-group-clips = { $count } 個片段
batch-prepare-check-group = 勾選這些片段
batch-solo-queued = 已排入佇列：{ $action }
batch-suggest-in-job = 執行中的工作已包含此片段的描述（不含標籤建議）。請在工作結束後再試一次
batch-suggest-no-tags = 這個資料夾還沒有可供建議的標籤：請先新增一些
batch-solo-gave-up = 無法準備就緒
batch-solo-withdrawn = 已撤回：{ $action }
batch-solo-in-work = { $action } 進行中。請從狀態列或工作頁面取消
batch-waiting-title = { $count ->
   *[other] { $count } 項執行等待中
}
batch-waiting-remove = 移除
batch-indicator-waiting = { $count } 項等待中
file-workspace-describe-clip = 用 AI 描述這個片段
file-workspace-subtitles-clip = 為這個片段產生字幕
file-workspace-describe-clip-queued = 用 AI 描述：等待輪到。點一下撤回
file-workspace-describe-clip-running = 用 AI 描述：處理中
file-workspace-suggest-tags-clip = 用 AI 為這個片段建議標籤（Ctrl+F6）
file-workspace-suggest-tags-clip-queued = 建議標籤：等待輪到。點一下撤回
file-workspace-suggest-tags-clip-running = 建議標籤：處理中
file-workspace-subtitles-clip-queued = 產生字幕：等待輪到。點一下撤回
file-workspace-subtitles-clip-running = 產生字幕：處理中
batch-run-again = 對 { $count ->
   *[other] { $count } 個檔案
} 再執行一次
batch-result-done = 完成：{ $total ->
   *[other] { $total } 個檔案
}
batch-result-done-detail = { $changed } 個已變更，{ $unchanged } 個無需變更
batch-result-problems = 完成但有問題：{ $total ->
   *[other] { $total } 個檔案
}中有 { $failed } 個未完成
batch-figure-unchanged = 未變更
batch-figure-not-done = 未完成
batch-figure-not-reached = 未處理到
batch-files-not-done = 未完成的檔案
batch-table-file = 檔案
batch-table-why = 原因

batch-run-move-comments = 移動 { $count ->
   *[other] { $count } 則註解
}
batch-run-move-in-out = 移動 { $count ->
   *[other] { $count } 個檔案
} 的入點／出點
batch-run-convert = 轉換 { $count ->
   *[other] { $count } 個檔案
}
batch-run-rotate = 旋轉 { $count ->
   *[other] { $count } 部影片
}
batch-run-tag = 為 { $count ->
   *[other] { $count } 個檔案
} 加標籤
batch-run-fix-tags = 重新排列 { $count ->
   *[other] { $count } 個名稱
} 中的標籤
batch-run-rename = 重新命名 { $count ->
   *[other] { $count } 個檔案
}
batch-run-reload = 重新載入 { $count ->
   *[other] { $count } 個檔案
}
batch-run-export-markers = 匯出 { $count ->
   *[other] { $count } 個檔案
} 的標記
batch-service-soniox = Soniox
batch-change-renames = 重新命名檔案
batch-change-videos = 寫入影片
batch-change-text-files = 在影片旁寫入文字檔
batch-change-comments = 寫入註解
batch-change-subtitles = 在影片旁寫入字幕檔
batch-change-records = 只變更 frename 自己的記錄
batch-option-direction = 方向
batch-option-language = 語言
batch-option-tag-suggestions = 標籤建議
batch-option-described = 已描述的影片
batch-option-subtitled = 已有字幕的影片
batch-option-subtitles-write = 要寫入的檔案
batch-option-spacing = 間距
batch-option-tag = 標籤
batch-option-turn = 旋轉
batch-plan-videos = 影片
batch-plan-length = 長度
batch-plan-cost = 費用
batch-plan-time = 時間
batch-reason-estimate = 等待估算
batch-reason-subtitles-no-format = 請至少勾選一種要寫入的檔案
batch-set-key = 設定金鑰…
batch-check-key = 檢查金鑰…
batch-subtitles-languages-auto = 依各影片自動偵測
batch-action-describe-ai-run-waiting = 描述影片
batch-action-fix-tags-order = 順序：依標籤清單，未知的標籤排最前面。
batch-action-markers-comment-hint-short = 將帶時間的註解行轉為標記，或將標記複製到註解。
batch-action-markers-to-comment-hint = 標記會保留；再執行一次不會重複新增。
batch-action-rotate-hint-short = 依旋轉旗標旋轉 MP4 與 MOV 片段，不重新編碼。
batch-action-tag-commented-off = 為有註解的影片加標籤的功能已關閉
batch-action-tag-commented-choose = 選擇標籤…

batch-title = 批次動作
batch-done-label-changed = 已變更
batch-done-label-subtitled = 已加字幕
batch-ai-at-least = 至少
batch-cancel = 取消
batch-stopping = 正在停止…
batch-stop-over-budget = 已停止：下一個片段可能使這次執行超出 { $cap } 的上限（已花費 { $spent }）。提高上限即可完成其餘部分。
batch-stopped = 已停止，已處理 { $finished } / { $total ->
   *[other] { $total } 個檔案
}。
batch-close = 關閉
batch-failed-subtitles = 未加字幕：
batch-written-subtitles = 已寫入的檔案：
batch-table-written = 已寫入
batch-merged-markers = 已合併的重複標記：
batch-table-merged = 已合併
batch-markers-merged = { $count ->
   *[other] 已合併 { $count } 個重複標記
}
batch-add-credit = 儲值
batch-open-log = 開啟記錄

batch-action-move-comments = 移動註解
batch-action-move-comments-hint = 將每個勾選檔案的註解連同其中的入點／出點移到所選位置。影片內保存的標籤與入點／出點維持原狀。
batch-action-move-comments-into-videos = 從文字檔移入影片（XMP）
batch-action-move-comments-into-text-files = 從影片（XMP）移入文字檔

batch-action-move-in-out = 入點／出點：註解 ⇄ 影片（XMP）
batch-action-move-in-out-hint = 將每個勾選檔案的入點／出點移到所選位置。註解的其餘部分維持原處。
batch-action-move-in-out-into-videos = 從註解移入影片（Adobe XMP 標記）
batch-action-move-in-out-into-comments = 從影片（XMP 標記）移入註解

batch-action-markers-comment = 標記 ⇄ 註解
batch-action-markers-comment-to-markers = 將帶時間的註解行轉為標記
batch-action-markers-to-comment = 將標記轉入註解（複製：標記會保留）
batch-action-markers-comment-hint = 像「03:24 — 第 3 次拍攝 — 光線很好」這樣的一行，會成為位於 3:24、名稱為「第 3 次拍攝」、註解為「光線很好」的標記；「0:41-0:47 — 獅子」則是從 0:41 到 0:47 的標記。名稱與註解在第一個「 — 」或「 -- 」處分開，單純的「 - 」不會。AI 描述中的時刻（「0:00–0:14 街道。」）會成為白色標記，並保留在描述中。不論哪個方向，再執行一次都不會重複新增。

batch-action-tag-commented = 為有註解的影片加標籤
batch-action-tag-commented-hint = 為每個勾選且有你自己註解的影片加上「{ $tag }」標籤（AI 描述不算），並從沒有註解的影片移除。標籤有變動的檔案會被重新命名。
batch-action-tag-commented-hint-off = 為每個勾選且有你自己註解的影片加上「有註解影片」的標籤，並從沒有註解的影片移除。此標籤在設定中已關閉。
batch-action-fix-tags = 依優先順序修正標籤
batch-action-fix-tags-hint = 將每個勾選檔案名稱中的標籤，依標籤面板的順序排列：標籤在面板中越前面，在名稱中也越前面。資料夾尚未收錄的標籤和在標籤面板中一樣排最前面。順序有變動的檔案會被重新命名。

batch-action-respace-tags = 套用標籤間距
batch-action-respace-tags-hint-space = 依設定中的選項，重新命名每個勾選的檔案，在每個標籤後加空格：Food. Goat. clip.mp4。
batch-action-respace-tags-hint-no-space = 依設定中的選項，重新命名每個勾選的檔案，標籤後不加空格：Food.Goat.clip.mp4。
batch-action-respace-tags-status-space = 間距：每個標籤後加空格
batch-action-respace-tags-status-no-space = 間距：標籤後不加空格
batch-action-export-markers = 將標記匯出為文字
batch-action-export-markers-hint = 將每個勾選檔案的標記寫成旁邊的文字檔 clip.markers.txt：每個標記一行，依時間排序，例如「0:41–0:47 [red] — 獅子過馬路 — 由左走到右」。綠色以外的顏色寫在方括號內；註解接在名稱之後。「標記 ⇄ 註解」可將這類行讀回成標記。沒有標記的檔案不會產生文字檔，先前執行產生的文字檔會被取代。
batch-export-markers-cannot-hold = 此檔案無法存放標記
batch-action-reload-files = 重設快取並重新載入
batch-action-reload-files-hint = 重新從檔案本身讀取每個勾選檔案的註解與入點／出點，並取代資料夾記住的內容。在其他程式修改過檔案後使用。記住的值缺漏或已過期的檔案會算作已變更。

batch-action-describe-ai = 用 AI 描述
batch-action-describe-ai-run = 描述 { $videos } · 約 { $dollars }
batch-action-describe-ai-estimating = 正在估算… { $known } / { $total }
batch-action-describe-ai-none = 沒有要描述的影片。
batch-action-describe-ai-hint = { batch-runs-in-background }。取消會保留已描述的影片；再執行一次會略過它們。
batch-action-describe-ai-no-subtitles = 沒有字幕（只描述畫面）：{ $videos }。
batch-action-describe-ai-redo = 重做已有 AI 描述的影片
batch-action-describe-ai-hint-panel = 描述每個勾選影片中發生的事與發生的時間：摘要與帶時間範圍的段落會寫入註解的 AI 描述；你自己寫的文字會保留。畫面與字幕會傳送給 { $provider }，也就是所選模型所屬的公司。
batch-ai-change = 變更
batch-ai-tag-suggestions-off = 關閉
batch-ai-tag-suggestions-no-tags = 已開啟，但這個資料夾還沒有標籤
batch-ai-tag-suggestions-on = 來自資料夾的 { $tags ->
   *[other] { $tags } 個標籤
}
batch-ai-key-missing = 請在設定中設定 { $provider } API 金鑰
batch-ai-key-unavailable = 無法開啟系統金鑰環：可能已鎖定，或根本沒有（例如 GNOME Keyring 或 KWallet）。
## AI 描述的語言名稱：設定中的選單，以及上方的 { $language }。

ai-language-same-as-subtitles = 與字幕相同
ai-language-english = 英文
ai-language-russian = 俄文
ai-language-ukrainian = 烏克蘭文
ai-language-german = 德文
ai-language-spanish = 西班牙文
ai-language-french = 法文
batch-videos-count = { $n ->
   *[other] { $n } 部影片
}
batch-minutes = { $n } 分
batch-ai-minutes-under = < 1 分
batch-ai-duration-under-minute = 不到一分鐘
batch-ai-duration-hours = { $h } 小時
batch-ai-duration-hours-minutes = { $h } 小時 { $m } 分
batch-ai-dollars-under = 低於 $0.01
batch-ai-skip-described = 已描述
batch-ai-skip-too-long = 超過 30 分鐘
batch-ai-skip-unreadable = 無法讀取
batch-ai-skipped = 已略過：{ $parts }。
batch-ai-progress-frame = 第 { $done } / { $total } 張影格
batch-ai-progress-waiting = 等待 { $provider }
batch-ai-progress-saving = 儲存中
batch-ai-progress-retry-rate-limit = { $seconds } 秒後重試（達到速率限制）
batch-ai-progress-retry-temporary = { $seconds } 秒後重試（暫時性失敗）
batch-ai-stop-no-key = 已停止：沒有 { $provider } API 金鑰。請在設定中設定。
batch-ai-fail-no-key = 沒有 API 金鑰
batch-ai-fail-unreadable = 無法讀取影片
batch-ai-fail-too-long = 片段太長，無法使用 AI（超過 30 分鐘）
batch-ai-fail-not-saved = 無法儲存描述
batch-ai-stop-offline = 已停止：無法連線到 { $provider }。再執行一次即可描述其餘部分。

batch-action-generate-subtitles = 產生字幕
batch-action-generate-subtitles-install-ffmpeg = 若要讀取 .mkv、.m2ts、.avi …，請從 ffmpeg.org 安裝 ffmpeg，將它加入 PATH，然後重新啟動 frename
batch-action-generate-subtitles-replace = 取代現有字幕
batch-action-generate-subtitles-replace-hint = 重新轉錄；費用如所示。
batch-action-generate-subtitles-srt = SRT 字幕（clip.srt）
batch-action-generate-subtitles-premiere = Premiere Pro 逐字稿（clip.premiere.json）
batch-action-generate-subtitles-premiere-hint = 在 Premiere Pro 中：文字面板 → 逐字稿 → 匯入靜態逐字稿。
batch-action-generate-subtitles-privacy = 這些影片的音訊會傳送給 Soniox，之後會在該處刪除。
batch-action-generate-subtitles-duration-hint = 每小時的音訊約需幾分鐘。{ batch-runs-in-background }。關閉 frename 會停止執行；已完成的字幕會保留。
batch-action-generate-subtitles-hint = 用 Soniox 轉錄每個勾選影片中的語音，並將結果存在影片旁：字幕（clip.srt，frename 在此顯示），以及／或 Premiere Pro 逐字稿。
batch-subtitles-transcribe = 轉錄
batch-subtitles-transcribe-count = 轉錄 { $videos }
batch-subtitles-build-free = 建立 { $count } 個（免費）
batch-subtitles-nothing = 沒有要轉錄的內容
batch-subtitles-estimating = 正在估算…
batch-subtitles-key-missing = 請在設定中設定 Soniox API 金鑰
batch-subtitles-key-rejected = Soniox 拒絕了此金鑰
batch-subtitles-cost-unknown = 費用未知
batch-subtitles-typical-price = { $usd }（一般價格）
batch-subtitles-unknown-length = + { $n } 個長度未知
batch-subtitles-plan-line = { $videos } 待轉錄，音訊共 { $duration }{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
   *[other] { $n } 個已有字幕
}
batch-subtitles-rebuilt-free = { $n } 個由已儲存的逐字稿免費重建
batch-subtitles-no-speech-before = { $n } 個上次沒有語音
batch-subtitles-no-audio = { $n ->
   *[other] { $n } 個沒有音訊
}
batch-subtitles-shared-name = { $n ->
   *[other] { $n } 個的字幕名稱與其他影片相同
}
batch-subtitles-unreadable = { $n } 個無法讀取（{ $how_to }）
batch-subtitles-shared-name-reason = 字幕名稱與其他影片相同
batch-subtitles-unsupported-reason = 不支援此音訊格式
batch-subtitles-progress-transcribing = 轉錄中…
batch-subtitles-progress-extracting = 正在擷取音訊 { $percent } %
batch-subtitles-progress-extracting-unknown = 正在擷取音訊
batch-subtitles-progress-uploading = 上傳中 { $percent } %
batch-subtitles-progress-uploading-unknown = 上傳中
batch-subtitles-progress-transcribing-left = 轉錄中… 約剩 { $time }
batch-subtitles-progress-saving = 儲存中
batch-subtitles-reason-already = 已有字幕
batch-subtitles-reason-no-audio = 沒有音訊
batch-subtitles-reason-no-speech = 沒有語音
batch-subtitles-reason-soniox = Soniox：{ $message }
batch-subtitles-reason-unreachable = 無法連線到 Soniox
batch-subtitles-stop-key-rejected = 已停止：Soniox 拒絕了此金鑰。請到「設定」→「字幕」檢查。
batch-subtitles-stop-balance-empty = 已停止：Soniox 餘額已用完。請到 console.soniox.com 儲值。
batch-subtitles-stop-budget-used = 已停止：Soniox 的每月預算已用完。請到 console.soniox.com 調高。
batch-subtitles-stop-other = 已停止：Soniox 拒絕繼續（{ $error_type }）。
batch-subtitles-stop-unreachable = 已停止：無法連線到 Soniox。請檢查網路連線。
batch-subtitles-report-spend = Soniox：至少 { $duration } · { $usd }
batch-subtitles-report-failed-deletes = 有 { $n ->
   *[other] { $n } 個上傳檔案
}無法在 Soniox 上刪除（請見記錄）
batch-subtitles-languages-need-key = 請儲存 Soniox 金鑰，以便從其所有語言中選擇。
batch-subtitles-languages-rejected = Soniox 拒絕了此金鑰。
batch-subtitles-languages-failed = 無法從 Soniox 取得語言。
batch-subtitles-count = { $n ->
   *[other] { $n } 個字幕
}
batch-subtitles-duration-seconds = { $s } 秒
batch-subtitles-usd-under = 低於 $0.01
batch-subtitles-usd-about = 約 ${ $amount }

## 檔案清單
folder-has-subtitles = 有字幕
folder-closed-by-job = 批次動作變更期間已關閉
folder-filter-tip = 只顯示…

folder-search-placeholder = 尋找檔案或註解
folder-search-comments-loading = 正在搜尋註解… 還剩 { $n } 個
folder-search-clear = 清除
folder-all = 全部
folder-invert = 反向選取
folder-checked = 已勾選 { $count } 個
folder-outcome-changed = 已變更
folder-outcome-unchanged = 無需變更
folder-outcome-failed = 失敗，請見記錄
folder-rename-error-empty = 名稱不可為空
folder-rename-error-bad-character = 不允許：\ / : * ? " < > |
folder-rename-error-trailing = 不可以句點或空格結尾
folder-rename-error-exists = 已有同名檔案
folder-markers-not-saved = 標記未儲存：檔案為唯讀或使用中（請在 Premiere 中關閉它，然後開啟該檔案再離開）
drag-out-not-saved = 未拖出：無法儲存檔案（唯讀，或已在 Premiere 中開啟）
folder-opening = 正在開啟資料夾…
folder-empty-title = 這個資料夾中沒有影片
folder-empty-line = frename 會顯示 MP4、MOV、MKV 與其他影片檔。
folder-open-another = 開啟其他資料夾…
folder-no-match = 沒有符合的檔案
folder-show-all = 顯示全部
folder-locked = { $action } 執行期間無法變更勾選
folder-checked-hidden = 已勾選 { $count } 個 · 隱藏 { $hidden } 個
folder-checked-hidden-tip = { $hidden ->
   *[other] 有 { $hidden } 個已勾選的檔案被搜尋或篩選隱藏；動作也會對它們執行
}
folder-outcome-working = 處理中
folder-outcome-not-reached = 未處理到
folder-window-empty-title = 開啟片段資料夾
folder-window-empty-line = 或將資料夾拖放到視窗上。在按鈕上按右鍵可開啟單一檔案。
folder-window-open = 開啟資料夾…

## 最近的資料夾（開啟按鈕旁的箭頭，以及空白畫面）

recent-folders-tip = 最近的資料夾
recent-folders-title = 最近的資料夾
recent-folders-none = 尚未開啟過資料夾
recent-folders-not-found = 找不到
recent-folders-remove-tip = 從清單中移除
recent-folders-clear = 清除清單
recent-folders-remove = 移除
recent-folders-keep = 保留
recent-folders-missing-line = 找不到。要移除嗎？
recent-folders-just-now = 剛剛
recent-folders-minutes = { $count } 分鐘前
recent-folders-hours = { $count } 小時前
recent-folders-days = { $count ->
   *[other] { $count } 天前
}
recent-folders-weeks = { $count ->
   *[other] { $count } 週前
}
recent-folders-months = { $count ->
   *[other] { $count } 個月前
}
recent-folders-years = { $count ->
   *[other] { $count } 年前
}

## 檔案選單（在檔案上按右鍵，或按 F11 / Shift+F11 / Ctrl+F11）

file-menu-show-in-explorer = 在檔案總管中顯示
file-menu-show-in-file-manager = 在檔案管理員中顯示
file-menu-copy-path = 複製完整路徑
file-menu-copy-name = 複製檔名
file-menu-copied = 已複製
file-menu-not-copied = 未複製：剪貼簿無法使用
file-menu-not-shown = 無法開啟檔案管理員

## 檔案清單下方的控制列

folder-controls-filter-untagged = 未加標籤
folder-controls-filter-subtitles = 字幕
folder-controls-filter-comments = 註解
folder-controls-filter-markers = 標記
folder-controls-scroll = 在清單中顯示已開啟的檔案
folder-controls-open = 開啟資料夾
folder-controls-batch = 對勾選的檔案執行批次動作
folder-controls-batch-back = 回到已開啟的檔案
app-bar-single = 單一檔案
app-bar-batch = 批次
folder-controls-update-available = 有可用更新：{ $version }
folder-controls-whats-new = { $version } 的新功能
folder-controls-previous = 上一個檔案
folder-controls-next = 下一個檔案
folder-controls-open-file = 按右鍵：開啟單一檔案

## 影片

video-controls-back = 倒退 10 秒
video-controls-play = 播放
video-controls-pause = 暫停
video-controls-forward = 快轉 10 秒
video-controls-frame-back = 後退一格
video-controls-frame-forward = 前進一格
video-controls-set-in = 設定入點
video-controls-set-out = 設定出點
video-controls-screenshot = 儲存這個影格
video-controls-add-marker = 新增標記
video-controls-add-marker-hold = 按住可建立範圍；再按一次可命名
video-controls-cannot-hold-markers = 此檔案無法存放標記
video-controls-add-a-name = 新增名稱
video-controls-rotate-left = 向左旋轉
video-controls-rotate-right = 向右旋轉
video-controls-more = 更多
media-viewer-loading-slow = 正在等待檔案…（雲端檔案可能需要一些時間）
media-viewer-cannot-play = 無法播放這個片段
media-viewer-no-picture = 此檔案沒有影像
video-timecode-copied = 已複製時間碼：{ $time }
video-timecode-unreadable = 「{ $text }」不是這個片段中的時間
video-controls-volume-scroll = 音量 — 捲動滾輪調整
rotate-cannot = 無法旋轉：{ $reason }
rotate-reason-missing = 檔案已不存在
rotate-flag-right = 向右 90°
rotate-flag-left = 向左 90°
rotate-flag-half = 180°
rotate-flag-none = 無
rotate-now = 旋轉：{ $flag }
rotate-turned = 已旋轉 { $turn } · 目前旋轉 { $flag }
rotate-failed = 未旋轉：{ $reason }
undo-name-taken = 未變更：已有其他檔案名為 { $name }
undo-back-on-clip = 已回到這個片段（已復原移動）。再按一次 Ctrl+Z 會復原它的上一個變更。
redo-on-clip = 已再次回到這個片段（已重做移動）。
rotate-reason-in-use = 檔案為唯讀或使用中
rotate-reason-format = 此格式沒有旋轉旗標
rotate-reason-damaged = 檔案已損毀
rotate-reason-no-video = 檔案沒有視訊軌
rotate-reason-matrix = 影片的顯示矩陣不尋常
batch-action-rotate = 旋轉影片
batch-action-rotate-hint = 變更每個勾選的 MP4 與 MOV 檔的旋轉旗標；畫面不會重新編碼，註解、入點／出點與標記都會保留。預期 Premiere Pro 匯入時會顯示旋轉後的片段。其他格式沒有旋轉旗標，會失敗。再執行一次會再旋轉一次檔案；「重設」會移除所有旋轉，包括手機錄製時產生的旋轉，所以手機的直式片段之後會橫著播放。
batch-action-rotate-right = 向右 90°（順時針）
batch-action-rotate-left = 向左 90°（逆時針）
batch-action-rotate-half = 180°
batch-action-rotate-reset = 重設：不旋轉（0°）
media-viewer-video-subtitle-list = 字幕清單
media-viewer-video-marker-list = 標記清單
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 在標記之間跳轉；Shift+拖曳會吸附
media-viewer-video-tab-markers = 標記
media-viewer-video-fullscreen = 全螢幕
media-viewer-video-close-list = 關閉清單
side-search-tip = 搜尋
side-search-placeholder-subtitles = 在字幕中尋找
side-search-placeholder-markers = 在標記中尋找
side-search-clear = 清除
side-search-next = 下一個符合項
side-search-previous = 上一個符合項
side-search-count = { $shown } / { $total }
side-search-nothing = 找不到「{ $query }」
side-search-nothing-clear = 清除搜尋
media-viewer-video-subtitle-file-unreadable = 無法讀取此檔案
# 片段開啟時，若從上次停止的位置繼續播放，會顯示在畫面上方；按 Home 或點一下可回到開頭。
media-viewer-video-resumed = 已從 { $time } 繼續 · Home：從頭開始

## 標記清單
markers-in-out = 入點／出點

markers-empty = 尚無標記
markers-add = 新增標記
markers-ai-hint = AI 標記：AI 再次描述這個片段時會被取代
markers-keep-color = 保留顏色
markers-done = 完成
markers-delete = 刪除標記
markers-cannot-hold-hint = Premiere 從 MP4 與 MOV 檔讀取標記。
markers-color-green = 綠色
markers-color-red = 紅色
markers-color-orange = 橙色
markers-color-yellow = 黃色
markers-color-white = 白色
markers-color-blue = 藍色
markers-color-cyan = 青色
markers-color-lavender = 薰衣草紫
markers-color-magenta = 洋紅色
markers-color-other = 其他顏色
markers-read-only = 唯讀
markers-name-placeholder = 名稱
markers-ai-describe = 用 AI 描述：為標記命名並補上發生的事
markers-ai-stop = 停止描述
markers-ai-describe-unnamed = 描述 { $count } 個未命名的標記
markers-ai-describe-unnamed-hint = 用 AI 為每個沒有名稱的標記命名並描述，一次處理幾個。每個標記一次請求；期間你自己輸入的名稱會保留。
markers-ai-price-about = 約 { $dollars }
markers-ai-describe-unnamed-cost = 每個標記一次請求：每個 { $each }，全部 { $total }。
markers-ai-confirm = { $count ->
   *[other] 要描述 { $count } 個標記嗎？每個標記一次請求：共 { $total }。
}
markers-ai-confirm-unpriced = { $count ->
   *[other] 要描述 { $count } 個標記嗎？每個標記一次付費請求。
}
markers-ai-confirm-yes = 描述
markers-ai-confirm-no = 取消
markers-ai-waiting = 等待輪到…
markers-ai-run = 描述中 { $describing } · 等待 { $waiting }
markers-ai-run-last = 描述中 { $describing }
markers-ai-stop-all = 全部停止
markers-ai-describing = 描述中…
markers-ai-done = 標記已描述
markers-ai-nothing-new = 標記已有這段描述
markers-no-marker-notice = 這裡沒有標記
markers-ai-no-key = 沒有 { $provider } API 金鑰：請在設定中設定
markers-ai-failed = 未描述：{ $reason }
markers-ai-failed-unknown = 發生錯誤
markers-read-only-notice = 該標記為唯讀
markers-ai-gone = 標記已不存在：未加入其描述
markers-ai-stopping-for-batch = 正在先停止標記描述… 通常只需幾秒。
folder-not-saved-marker-ai = 未儲存：標記描述仍在讀取片段。請稍後再試。

## 檔案工作區

file-workspace-search-placeholder = 尋找標籤，或直接輸入
file-workspace-search-clear = 清除
file-workspace-comment-placeholder = 註解...
file-workspace-comment-collapse = 回到標籤
file-workspace-comment-expand = 展開註解

## 標籤格

tag-grid-star = 星號：固定在最上方
tag-grid-unstar = 取消星號
tag-grid-save = 加入資料夾的標籤
tag-grid-delete = 從資料夾的標籤中刪除「{ $tag }」
tag-grid-create = 建立「{ $tag }」
tag-grid-no-file = 開啟片段即可加標籤
tag-grid-no-tags = 尚無標籤
tag-grid-no-tags-hint = 輸入名稱並按 Enter 建立第一個標籤。
tag-grid-group-unsaved = 不在資料夾的標籤中
tag-grid-group-folder = 資料夾標籤
tag-grid-more = 還有 { $count } 個

## 排序列

sync-panel-locked = 在下方重新排序會重排整個資料夾
sync-panel-unlocked = 在下方重新排序只會變更這個片段
sync-panel-unlock = 解除鎖定
sync-panel-lock = 鎖定
sync-panel-differs = 順序與資料夾不同
sync-panel-use-for-folder = 套用到資料夾
sync-panel-sort-like-folder = 依資料夾排序

## 檔名卡片

file-name-panel-no-tags = 這個片段沒有標籤
file-name-panel-untag = 移除標籤
file-name-panel-clear-in = 清除入點
file-name-panel-clear-out = 清除出點
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = 將入點與出點設為 AI 的建議（四捨五入到整數秒）
file-name-panel-suggested-tags = AI 標籤
file-name-panel-add-suggested-tag = 加入標籤「{ $tag }」（{ $percent }% 把握）
file-name-panel-add-suggested-tag-unsure = 加入標籤「{ $tag }」
file-name-panel-add-all-suggested-tags = 全部加入
file-name-panel-add-all-suggested-tags-tip = 加入 AI 建議的所有標籤
file-name-panel-tag-ideas = 靈感：{ $ideas }
file-name-panel-tag-ideas-tip = AI 注意到、但這個資料夾沒有的標籤。它們不會被加入。
file-name-panel-saved = 所有變更已儲存
file-name-panel-saved-to-recovery = 已存入復原檔，離開片段時套用

## 更新（設定）

updates-check = 檢查更新
updates-not-installed = 更新僅適用於已安裝的版本
updates-from-store = 更新來自 Microsoft Store
updates-from-releases = 新版本：請從發行頁面下載
updates-open-releases = 開啟發行頁面
updates-checking = 檢查中…
updates-downloading-named = 正在下載 { $version }… { $percent }%
updates-downloading = 下載中… { $percent }%
updates-restarting = 正在重新啟動…
updates-check-failed = 無法檢查更新：{ $reason }
updates-update-failed = 無法更新：{ $reason }
updates-close-other-window = 請先關閉另一個 frename 視窗
updates-reason-github = GitHub 沒有回應，請稍後再試
updates-reason-no-connection = 沒有網路連線
updates-reason-read-only = 資料夾為唯讀
updates-reason-not-installed = 更新僅適用於已安裝的版本
updates-reason-download-stopped = 下載意外中止
updates-version-available = 版本 { $version } 可供更新
updates-up-to-date = frename 已是最新版本
updates-update-and-restart = 更新並重新啟動
updates-wait-for-batch = 請等待批次完成
updates-current-version = frename { $version }
updates-check-on-start = frename 啟動時檢查更新
updates-notes-show = 版本資訊
updates-notes-hide = 隱藏版本資訊
updates-notes-all-releases = GitHub 上的所有發行版本
updates-notes-history = 所有版本，最新的在前
updates-notes-update = { $version } 的更新內容
updates-notes-whats-new = { $version } 的新功能
updates-notes-loading = 正在載入版本資訊…
updates-notes-unavailable = 無法載入最新版本的資訊。
updates-notes-more = 顯示更多

recovery-restored = frename 意外關閉。已還原你在 { $clip } 上尚未儲存的工作（{ $what }）。
recovery-kept = frename 意外關閉。你在 { $clip } 上尚未儲存的工作（{ $what }）未被套用：{ $why }。可供重新輸入的可讀副本位於 { $folder }。
recovery-unreadable = frename 意外關閉，且無法讀取復原檔。它保存在 { $folder }。
recovery-why-gone = 片段已不存在或已被重新命名
recovery-why-changed = 片段在你編輯後已變更
recovery-why-not-written = 已有同名檔案，或片段為唯讀或使用中
recovery-tags = { $count ->
   *[other] { $count } 個標籤
}
recovery-markers = { $count ->
   *[other] { $count } 個標記
}
recovery-comment = 註解
recovery-in-out = 入點／出點
recovery-edits = 編輯
folder-frame-saved = 影格已儲存
folder-not-saved = 未儲存：已有同名檔案，或檔案為唯讀或使用中
markers-deleted-notice = 標記已刪除
markers-not-saved-notice = 標記未儲存：檔案為唯讀或使用中
markers-comment-not-saved-notice = 標記未儲存：註解檔為唯讀或使用中
markers-copy = 複製
markers-copy-tip = 將標記複製為文字，每個一行
markers-export = 匯出…
markers-export-tip = 將標記儲存為文字檔
markers-exported = 標記已儲存至 { $name }
markers-not-exported = 標記未儲存：無法寫入檔案

batch-option-notes = 給 AI 的備註
batch-ai-notes-info = 你對這個資料夾素材的備註，會附在此處每個 AI 請求中：描述、標籤建議與標記描述。儲存在資料夾的 .frename 檔中。
batch-ai-notes-placeholder = 例如：最常出現的男士是 Eugene。標出廣告看板。看到茶就建議標籤 tea。
batch-ai-notes-hint = 會連同畫面一起傳送給 { $provider }。名稱只來自這些備註；frename 不辨識人臉。已描述的影片會被略過：勾選「重做」即可用新的備註重新描述。
batch-ai-notes-too-long = 備註有 { $chars } 個字元；上限為 { $max }。請縮短後再執行。
batch-ai-notes-too-long-reason = 給 AI 的備註太長
batch-option-notes-once = 這次執行的額外一行
batch-ai-notes-once-placeholder = 例如：這個片段：穿紅衣的女士是 Ana
file-workspace-ai-notes-on = 備註已開啟
file-workspace-ai-notes-tip = 給 AI 的備註會附在此資料夾的每個 AI 請求中。請在「用 AI 描述」（批次動作）中編輯：{ $notes }
batch-ai-notes-no-subtitles = 你的備註似乎與說話內容有關，但有 { $videos } 沒有字幕：請先產生字幕（產生字幕），AI 才能讀到說話內容。
