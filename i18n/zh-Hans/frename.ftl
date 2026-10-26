# frename 界面文本，源语言文件。i18n/ 下的其他文件与此文件的 id、属性和 $参数完全一致
# （由 src/i18n.rs 中的测试检查）。
# 键名：<功能>-<元素>[-<细节>]，功能即 src/features/ 下的文件夹名。

## 语言名称：所有文件中都相同，每种语言用其自身书写。

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

## 设置窗口

settings-window-title = 设置
settings-page-interface = 界面
settings-page-saving = 保存
settings-page-ai = AI 描述
settings-close = 关闭
settings-apply-note = 更改立即生效。Tab 在选项间切换，Ctrl+Tab 在页面间切换。
settings-language = 语言
settings-language-system = 系统（{ $language }）
settings-video = 视频
settings-video-autoplay = 打开时自动播放视频
settings-subtitle-size = 字幕大小
settings-subtitle-size-small = 小
settings-subtitle-size-medium = 中
settings-subtitle-size-large = 大
settings-subtitle-size-extra-large = 特大
settings-timecode = 时间显示
settings-timecode-hint = 视频下方、标记和字幕列表、入点/出点标签上的时间写法，以及 Ctrl+Shift+C 复制的播放头时间。Premiere Pro 和 DaVinci Resolve 按素材帧率显示 时:分:秒:帧；29.97 和 59.94 fps 为丢帧。帧率不稳定的素材使用默认样式。文件保持自己的格式。
settings-timecode-default = 默认（mm:ss）
settings-timecode-premiere = Premiere Pro（HH:MM:SS:FF）
settings-timecode-resolve = DaVinci Resolve（HH:MM:SS:FF）
settings-tags = 标签颜色
settings-tags-monochrome = 单色
settings-tags-monochrome-hint = 你添加的标签显示为较深的灰色，其他标签为较浅的灰色，不使用其他颜色。
settings-file-names = 文件名
settings-tags-space-after = 每个标签后加空格
settings-tags-space-example = Food. Goat. clip.mp4
settings-tags-space-note = 文件在重命名或保存之前保持原名。
settings-tags-space-add = 为文件名添加空格…
settings-tags-space-remove = 从文件名中删除空格…
settings-comments = 注释
settings-comments-in-video = 写入视频文件内
settings-comments-in-video-hint = XMP，即 Premiere Pro 中的“说明”列
settings-comments-text-file = 写入视频旁的文本文件
settings-comments-text-file-example = clip.comment.txt
settings-comments-note = 文件的注释在移动之前保留在原处。
settings-comments-move-into-videos = 将注释移入视频…
settings-comments-move-into-text-files = 将注释移入文本文件…
settings-commented-tag = 给有注释的视频加标签
settings-commented-tag-name = 标签
settings-commented-tag-hint = 为视频写下注释时勾选，例如 { $tag }.IMG_0424.MOV；清除注释时取消勾选（AI 描述不算）。除此之外由你决定。
settings-commented-tag-off = 有注释的视频不加标签。
settings-in-out = 入点/出点
settings-in-out-in-video = 写入视频文件内
settings-in-out-in-video-hint = Premiere Pro 中的子剪辑标记
settings-in-out-comment = 写入注释，占一行
settings-in-out-comment-example = In/Out: 00:01:05.250 – 00:02:10.000
settings-in-out-note = 文件的入点/出点在移动之前保留在原处。
settings-in-out-move-into-videos = 将入点/出点移入视频…
settings-in-out-move-into-comments = 将入点/出点移入注释…
settings-markers = 标记和范围
settings-markers-in-video = 写入视频文件内
settings-markers-in-video-hint = XMP，显示在 Premiere Pro 的素材上
settings-markers-comment = 写入注释，每个一行
settings-markers-comment-example = 0:41–0:47 — Lion
settings-markers-note = 文件的标记在移动之前保留在原处。
settings-markers-move-into-videos = 将标记移入视频…
settings-markers-copy-into-comment = 将标记复制到注释…
settings-markers-hint = 时间点和范围都包括，AI 找出的片段也一样。
settings-updates = 更新
settings-version = 版本
settings-release-notes = 发行说明
settings-old-title = 旧版 frename 的设置
settings-old-hint = 导入你从 zip 文件夹运行的 frename 的设置。
settings-old-scheduled = frename 重启时将导入设置
settings-old-not-found = 在 { $folder } 中没有带 frename.db 的 frename.exe
settings-old-failed = 无法导入：{ $reason }
settings-old-import-button = 从旧 frename 文件夹导入…
settings-ai = AI 描述
settings-ai-key-label = Anthropic API 密钥
settings-ai-key-placeholder = sk-ant-…
settings-ai-key-get = 前往 console.anthropic.com → API keys 获取密钥。
settings-ai-key-remove-confirm = 删除已保存的 Anthropic 密钥？
settings-ai-used-by = 选择 Claude 模型时，用于批量模式和标记上的 AI 描述（✨，Ctrl+F2）。
settings-ai-openai-key-label = OpenAI API 密钥
settings-ai-openai-key-placeholder = sk-…
settings-ai-openai-key-get = 前往 platform.openai.com/api-keys 获取密钥。
settings-ai-openai-key-remove-confirm = 删除已保存的 OpenAI 密钥？
settings-ai-openai-used-by = 选择 GPT 模型时，用于批量模式和标记上的 AI 描述（✨，Ctrl+F2）。
settings-ai-gemini-key-label = Gemini API 密钥
settings-ai-gemini-key-placeholder = AIza…
settings-ai-gemini-key-get = 前往 aistudio.google.com/apikey 获取密钥。
settings-ai-gemini-key-remove-confirm = 删除已保存的 Gemini 密钥？
settings-ai-gemini-used-by = 选择 Gemini 模型时，用于批量模式和标记上的 AI 描述（✨，Ctrl+F2）。
settings-ai-model-needs-key = { $model }：请先添加 { $provider } 密钥
settings-ai-model-label = 模型
settings-ai-language-label = 描述语言
settings-ai-hint = Haiku、GPT-4.1 mini 和 Gemini Flash-Lite 最便宜；Sonnet、Opus、GPT-4.1 和 Gemini Flash 观察更细致。模型需要其所属公司的密钥。
settings-ai-moments-label = 片段
settings-ai-moments-important = 只标出突出的部分
settings-ai-moments-full = 覆盖整段素材
settings-ai-moments-important-hint = 同时建议入点和出点的位置（IN 和 OUT 旁的 AI 标签）
settings-ai-moments-full-hint = 每个部分都有一个片段，包括安静的部分；不建议入点和出点
settings-ai-tag-suggestions = 同时根据文件夹的标签推荐标签
settings-ai-tag-suggestions-hint = 在同一次请求中，AI 会指出文件夹中的哪些标签适合每段素材。推荐的标签只有在你点击后才会添加。
settings-subtitles = 字幕
settings-subtitles-key-label = Soniox API 密钥
settings-subtitles-key-placeholder = 粘贴密钥
settings-subtitles-key-get = 前往 console.soniox.com 获取密钥。音频会发送给 Soniox。
settings-subtitles-key-remove-confirm = 删除已保存的 Soniox 密钥？
settings-subtitles-languages-label = 语言
settings-subtitles-languages-locked = 保存密钥后，可从 Soniox 支持的所有语言中选择。
settings-subtitles-languages-loading = 正在从 Soniox 获取语言…
settings-subtitles-languages-none = 未勾选：自动检测。
settings-subtitles-languages-hint = 素材中使用的语言，作为提示。
settings-subtitles-cue-length-label = 字幕条长度
settings-subtitles-cue-short = 短
settings-subtitles-cue-short-hint = 一行，最长 8 秒
settings-subtitles-cue-sentence = 一句话
settings-subtitles-hint = 用于批量模式中的“生成字幕”。字幕条长度适用于新字幕。
settings-key-store-windows = Windows 凭据管理器
settings-key-store-macos = macOS 钥匙串
settings-key-store-other = 系统密钥环
settings-key-unavailable = 无法打开系统密钥环
settings-key-unavailable-hint = 它可能已锁定，或者不存在（如 GNOME Keyring 或 KWallet）。每次打开设置时都会重新检查。
settings-key-remove-confirm-hint = 之后需要重新粘贴。
settings-key-remove-ask = 删除…
settings-key-remove = 删除密钥
settings-key-keep = 保留
settings-key-replace = 替换…
settings-key-hide = 隐藏
settings-key-show = 显示
settings-key-save = 保存密钥
settings-key-get = 获取密钥
settings-key-billing = 充值
settings-key-cancel = 取消
settings-key-saved-in = 已保存在这台电脑的{ $store }中
settings-key-save-into = “保存密钥”会将其保存在这台电脑的{ $store }中。

## 批量模式

batch-checked-count = 已勾选 { $count ->
   *[other] { $count } 个文件
}
batch-no-key = 无密钥
batch-group-move = 在不同位置间移动
batch-group-fix = 修复名称和视频
batch-group-paid = 付费服务
batch-reason-none-checked = 未勾选文件
batch-reason-reading = 正在读取素材时长…
batch-check-all = 全选 { $count ->
   *[other] { $count } 个文件
}
batch-job-running = 运行中
batch-job-finished = 已完成
batch-job-stopped = 已停止
batch-progress-files = { $finished }/{ $total ->
   *[other] { $total }
} 个文件
batch-time-left = 约剩 { $time }
batch-time-estimating = 正在估算时间…
batch-time-spent = 已用 { $time }
batch-runs-in-background = 运行期间可以继续工作
batch-clip-in-work = 使用中：{ $action }
batch-folder-held = 忙碌：{ $action }
batch-indicator-hint = 点击打开面板并可取消
batch-action-prepare = 准备素材
batch-action-prepare-hint = 对每个已勾选的素材一次完成：没有字幕时先生成字幕（Soniox），然后用 AI 描述：摘要、作为标记的片段、注释和标签推荐。统一计价、统一上限、统一进度。
batch-prepare-step-off = 已关闭
batch-prepare-nothing = 这些素材无需处理
batch-prepare-report-cap = 已花费 { $spent }，上限 { $cap }
batch-option-steps = 步骤
batch-prepare-step-subtitles = 先生成字幕（Soniox）
batch-prepare-step-description = 用 AI 描述素材
batch-prepare-cap = 花费上限
batch-prepare-cap-none = 无上限（美元）
batch-prepare-cap-invalid = 上限应为美元数字，如 5
batch-prepare-left = 上次运行停止时还剩 { $count ->
   *[other] { $count } 个素材
}
batch-prepare-check-left = 勾选它们
batch-prepare-estimating = 正在计算每一步的工作量…
batch-prepare-plan-subtitles = 字幕
batch-prepare-plan-description = 描述
batch-prepare-plan-none = 无需处理
batch-prepare-plan-subtitles-line = 待转录 { $videos ->
   *[other] { $videos } 个视频
}，音频共 { $duration }
batch-prepare-plan-description-line = 待描述 { $videos ->
   *[other] { $videos } 个视频
}
batch-prepare-resume-note = 每一步都会跳过已有的内容，所以中途停止的任务再次运行即可继续：不会重复付费。
batch-prepare-run = 准备 { $clips ->
   *[other] { $clips } 个素材
} · 约 { $dollars }
batch-service-prepare = Soniox + { $provider }
batch-prepare-groups-title = 相似素材
batch-prepare-groups-hint = 看起来是同一镜头的素材：重复文件、重新导出的文件、机位没动的拍摄。
batch-prepare-group-clips = { $count } 个素材
batch-prepare-check-group = 勾选这些素材
batch-solo-queued = 已排队：{ $action }
batch-suggest-in-job = 正在运行的任务中已包含这段素材的描述，但没有标签推荐。请在它结束后再试
batch-suggest-no-tags = 此文件夹还没有可供推荐的标签：请先添加一些
batch-solo-gave-up = 无法准备就绪
batch-solo-withdrawn = 已撤回：{ $action }
batch-solo-in-work = { $action }正在进行。请在状态栏或任务页面中取消
batch-waiting-title = { $count ->
   *[other] { $count } 个任务等待中
}
batch-waiting-remove = 移除
batch-indicator-waiting = { $count } 个等待中
file-workspace-describe-clip = 用 AI 描述这段素材
file-workspace-subtitles-clip = 为这段素材生成字幕
file-workspace-describe-clip-queued = AI 描述：等待轮到它。点击撤回
file-workspace-describe-clip-running = AI 描述：处理中
file-workspace-suggest-tags-clip = 用 AI 为这段素材推荐标签（Ctrl+F6）
file-workspace-suggest-tags-clip-queued = 推荐标签：等待轮到它。点击撤回
file-workspace-suggest-tags-clip-running = 推荐标签：处理中
file-workspace-subtitles-clip-queued = 生成字幕：等待轮到它。点击撤回
file-workspace-subtitles-clip-running = 生成字幕：处理中
batch-run-again = 对 { $count ->
   *[other] { $count } 个文件
}再次运行
batch-result-done = 完成：{ $total ->
   *[other] { $total } 个文件
}
batch-result-done-detail = { $changed } 个已更改，{ $unchanged } 个无需更改
batch-result-problems = 完成，但有问题：{ $total ->
   *[other] { $total } 个文件
}中有 { $failed } 个未完成
batch-figure-unchanged = 未更改
batch-figure-not-done = 未完成
batch-figure-not-reached = 未处理到
batch-files-not-done = 未完成的文件
batch-table-file = 文件
batch-table-why = 原因

batch-run-move-comments = 移动 { $count ->
   *[other] { $count } 条注释
}
batch-run-move-in-out = 移动 { $count ->
   *[other] { $count } 个文件
}的入点/出点
batch-run-convert = 转换 { $count ->
   *[other] { $count } 个文件
}
batch-run-rotate = 旋转 { $count ->
   *[other] { $count } 个视频
}
batch-run-tag = 为 { $count ->
   *[other] { $count } 个文件
}打标签
batch-run-fix-tags = 重新排列 { $count ->
   *[other] { $count } 个名称
}中的标签
batch-run-rename = 重命名 { $count ->
   *[other] { $count } 个文件
}
batch-run-reload = 重新载入 { $count ->
   *[other] { $count } 个文件
}
batch-run-export-markers = 导出 { $count ->
   *[other] { $count } 个文件
}的标记
batch-service-soniox = Soniox
batch-change-renames = 重命名文件
batch-change-videos = 写入视频
batch-change-text-files = 在视频旁写入文本文件
batch-change-comments = 写入注释
batch-change-subtitles = 在视频旁写入字幕文件
batch-change-records = 只更改 frename 自己的记录
batch-option-direction = 方向
batch-option-language = 语言
batch-option-tag-suggestions = 标签推荐
batch-option-described = 已描述的视频
batch-option-subtitled = 已有字幕的视频
batch-option-subtitles-write = 要写入的文件
batch-option-spacing = 间距
batch-option-tag = 标签
batch-option-turn = 旋转
batch-plan-videos = 视频
batch-plan-length = 时长
batch-plan-cost = 费用
batch-plan-time = 时间
batch-reason-estimate = 等待估算
batch-reason-subtitles-no-format = 请至少勾选一种要写入的文件
batch-set-key = 设置密钥…
batch-check-key = 检查密钥…
batch-subtitles-languages-auto = 每个视频自动检测
batch-action-describe-ai-run-waiting = 描述视频
batch-action-fix-tags-order = 顺序：与标签列表一致，未知标签在前。
batch-action-markers-comment-hint-short = 将带时间的注释行转为标记，或将标记复制到注释。
batch-action-markers-to-comment-hint = 标记保留；再次运行不会重复添加。
batch-action-rotate-hint-short = 按旋转标志旋转 MP4 和 MOV 素材，无需重新编码。
batch-action-tag-commented-off = 已关闭给有注释的视频打标签
batch-action-tag-commented-choose = 选择标签…

batch-title = 批量操作
batch-done-label-changed = 已更改
batch-done-label-subtitled = 已加字幕
batch-ai-at-least = 至少
batch-cancel = 取消
batch-stopping = 正在停止…
batch-stop-over-budget = 已停止：下一个素材可能使本次运行超过 { $cap } 的上限（已花费 { $spent }）。提高上限即可完成其余部分。
batch-stopped = 已停止，已处理 { $finished }/{ $total ->
   *[other] { $total }
} 个文件。
batch-close = 关闭
batch-failed-subtitles = 未加字幕：
batch-written-subtitles = 已写入的文件：
batch-table-written = 已写入
batch-merged-markers = 已合并的重复标记：
batch-table-merged = 已合并
batch-markers-merged = { $count ->
   *[other] 已合并 { $count } 个重复标记
}
batch-add-credit = 充值
batch-open-log = 打开日志

batch-action-move-comments = 移动注释
batch-action-move-comments-hint = 将每个已勾选文件的注释连同其中的入点/出点移到所选位置。保存在视频中的标签和入点/出点保持不变。
batch-action-move-comments-into-videos = 从文本文件移入视频（XMP）
batch-action-move-comments-into-text-files = 从视频（XMP）移入文本文件

batch-action-move-in-out = 入点/出点：注释 ⇄ 视频（XMP）
batch-action-move-in-out-hint = 将每个已勾选文件的入点/出点移到所选位置。注释的其余部分保持原处。
batch-action-move-in-out-into-videos = 从注释移入视频（Adobe XMP 标记）
batch-action-move-in-out-into-comments = 从视频（XMP 标记）移入注释

batch-action-markers-comment = 标记 ⇄ 注释
batch-action-markers-comment-to-markers = 带时间的注释行转为标记
batch-action-markers-to-comment = 标记转入注释（复制：标记保留）
batch-action-markers-comment-hint = 像 "03:24 — Take 3 — nice light" 这样的一行，是位于 3:24、名称为 "Take 3"、注释为 "nice light" 的标记；"0:41-0:47 — Lion" 是从 0:41 到 0:47 的标记。名称和注释在第一个 " — " 或 " -- " 处分开，而不是普通的 " - "。AI 描述中的片段（"0:00–0:14 Street."）会成为白色标记，并保留在描述中。无论哪个方向，再次运行都不会重复添加。

batch-action-tag-commented = 给有注释的视频打标签
batch-action-tag-commented-hint = 为每个已勾选且有你自己注释的视频添加标签“{ $tag }”（AI 描述不算），并从没有注释的视频中移除。标签有变化的文件会被重命名。
batch-action-tag-commented-hint-off = 为每个已勾选且有你自己注释的视频添加“有注释视频”标签，并从没有注释的视频中移除。该标签已在设置中关闭。
batch-action-fix-tags = 按优先级整理标签
batch-action-fix-tags-hint = 将每个已勾选文件名中的标签按标签面板的顺序排列：标签在面板中越靠前，在文件名中也越靠前。文件夹尚不认识的标签像在标签面板中一样排在最前。顺序有变化的文件会被重命名。

batch-action-respace-tags = 应用标签间距
batch-action-respace-tags-hint-space = 按设置中的选择，重命名每个已勾选文件，使每个标签后有一个空格：Food. Goat. clip.mp4。
batch-action-respace-tags-hint-no-space = 按设置中的选择，重命名每个已勾选文件，使标签后没有空格：Food.Goat.clip.mp4。
batch-action-respace-tags-status-space = 间距：每个标签后加空格
batch-action-respace-tags-status-no-space = 间距：标签后不加空格
batch-action-export-markers = 将标记导出为文本
batch-action-export-markers-hint = 将每个已勾选文件的标记写成其旁边的文本文件 clip.markers.txt：每个标记一行，按时间排序，如 "0:41–0:47 [red] — Lion crossing — walks left to right"。绿色以外的颜色写在方括号中；注释跟在名称之后。“标记 ⇄ 注释”可将这样的行读回为标记。没有标记的文件不会生成文本文件，之前运行生成的文本文件会被替换。
batch-export-markers-cannot-hold = 此文件无法保存标记
batch-action-reload-files = 重置缓存并重新载入
batch-action-reload-files-hint = 从文件本身重新读取每个已勾选文件的注释和入点/出点，并替换文件夹记住的内容。在其他程序修改过文件之后使用。记住的值缺失或已过期的文件算作已更改。

batch-action-describe-ai = 用 AI 描述
batch-action-describe-ai-run = 描述 { $videos } · 约 { $dollars }
batch-action-describe-ai-estimating = 正在估算… { $known } / { $total }
batch-action-describe-ai-none = 没有需要描述的视频。
batch-action-describe-ai-hint = { batch-runs-in-background }。取消后已描述的视频会保留；再次运行会跳过它们。
batch-action-describe-ai-no-subtitles = 没有字幕（只描述画面）：{ $videos }。
batch-action-describe-ai-redo = 重新处理已有 AI 描述的视频
batch-action-describe-ai-hint-panel = 描述每个已勾选视频中发生了什么以及何时发生：摘要和带时间范围的片段会写入其注释的 AI 描述中；你自己的文字会保留。画面帧和字幕会发送给 { $provider }，即所选模型所属的公司。
batch-ai-change = 更改
batch-ai-tag-suggestions-off = 关
batch-ai-tag-suggestions-no-tags = 已开启，但此文件夹还没有标签
batch-ai-tag-suggestions-on = { $tags ->
   *[other] 来自文件夹的 { $tags } 个标签
}
batch-ai-key-missing = 请在设置中填写 { $provider } API 密钥
batch-ai-key-unavailable = 无法打开系统密钥环：它可能已锁定，或者不存在（如 GNOME Keyring 或 KWallet）。
## AI 描述的语言名称：设置中的选择器以及上面的 { $language }。

ai-language-same-as-subtitles = 与字幕相同
ai-language-english = 英语
ai-language-russian = 俄语
ai-language-ukrainian = 乌克兰语
ai-language-german = 德语
ai-language-spanish = 西班牙语
ai-language-french = 法语
batch-videos-count = { $n ->
   *[other] { $n } 个视频
}
batch-minutes = { $n } 分钟
batch-ai-minutes-under = < 1 分钟
batch-ai-duration-under-minute = 不到一分钟
batch-ai-duration-hours = { $h } 小时
batch-ai-duration-hours-minutes = { $h } 小时 { $m } 分钟
batch-ai-dollars-under = 低于 $0.01
batch-ai-skip-described = 已有描述
batch-ai-skip-too-long = 超过 30 分钟
batch-ai-skip-unreadable = 无法读取
batch-ai-skipped = 已跳过：{ $parts }。
batch-ai-progress-frame = 第 { $done } / { $total } 帧
batch-ai-progress-waiting = 等待 { $provider }
batch-ai-progress-saving = 正在保存
batch-ai-progress-retry-rate-limit = { $seconds } 秒后重试（触发速率限制）
batch-ai-progress-retry-temporary = { $seconds } 秒后重试（临时故障）
batch-ai-stop-no-key = 已停止：没有 { $provider } API 密钥。请在设置中填写。
batch-ai-fail-no-key = 没有 API 密钥
batch-ai-fail-unreadable = 无法读取视频
batch-ai-fail-too-long = 素材太长，AI 无法处理（超过 30 分钟）
batch-ai-fail-not-saved = 无法保存描述
batch-ai-stop-offline = 已停止：无法连接 { $provider }。再次运行以描述其余部分。

batch-action-generate-subtitles = 生成字幕
batch-action-generate-subtitles-install-ffmpeg = 要读取 .mkv、.m2ts、.avi …，请从 ffmpeg.org 安装 ffmpeg，将其加入 PATH，然后重启 frename
batch-action-generate-subtitles-replace = 替换现有字幕
batch-action-generate-subtitles-replace-hint = 重新转录；按所示费用计费。
batch-action-generate-subtitles-srt = SRT 字幕（clip.srt）
batch-action-generate-subtitles-premiere = Premiere Pro 转录文本（clip.premiere.json）
batch-action-generate-subtitles-premiere-hint = 在 Premiere Pro 中：文本面板 → 转录文本 → 导入静态转录文本。
batch-action-generate-subtitles-privacy = 这些视频的音频会发送给 Soniox，之后在那里删除。
batch-action-generate-subtitles-duration-hint = 每小时音频需要几分钟。{ batch-runs-in-background }。关闭 frename 会停止运行；已完成的字幕会保留。
batch-action-generate-subtitles-hint = 用 Soniox 转录每个已勾选视频中的语音，并将结果保存在视频旁：字幕（clip.srt，frename 在此显示），和/或 Premiere Pro 转录文本。
batch-subtitles-transcribe = 转录
batch-subtitles-transcribe-count = 转录 { $videos }
batch-subtitles-build-free = 生成 { $count } 个（免费）
batch-subtitles-nothing = 没有需要转录的内容
batch-subtitles-estimating = 正在估算…
batch-subtitles-key-missing = 请在设置中填写 Soniox API 密钥
batch-subtitles-key-rejected = Soniox 拒绝了该密钥
batch-subtitles-cost-unknown = 费用未知
batch-subtitles-typical-price = { $usd }（典型价格）
batch-subtitles-unknown-length = + { $n } 个时长未知
batch-subtitles-plan-line = 待转录 { $videos }，音频共 { $duration }{ $unknown } · { $cost }
batch-subtitles-already = { $n ->
   *[other] { $n } 个已有字幕
}
batch-subtitles-rebuilt-free = { $n } 个由已保存的转录文本免费重建
batch-subtitles-no-speech-before = { $n } 个上次没有检测到语音
batch-subtitles-no-audio = { $n ->
   *[other] { $n } 个没有音频
}
batch-subtitles-shared-name = { $n ->
   *[other] { $n } 个与其他视频的字幕同名
}
batch-subtitles-unreadable = { $n } 个无法读取（{ $how_to }）
batch-subtitles-shared-name-reason = 与其他视频的字幕同名
batch-subtitles-unsupported-reason = 不支持该音频格式
batch-subtitles-progress-transcribing = 正在转录…
batch-subtitles-progress-extracting = 正在提取音频 { $percent }%
batch-subtitles-progress-extracting-unknown = 正在提取音频
batch-subtitles-progress-uploading = 正在上传 { $percent }%
batch-subtitles-progress-uploading-unknown = 正在上传
batch-subtitles-progress-transcribing-left = 正在转录… 约剩 { $time }
batch-subtitles-progress-saving = 正在保存
batch-subtitles-reason-already = 已有字幕
batch-subtitles-reason-no-audio = 没有音频
batch-subtitles-reason-no-speech = 没有语音
batch-subtitles-reason-soniox = Soniox：{ $message }
batch-subtitles-reason-unreachable = 无法连接 Soniox
batch-subtitles-stop-key-rejected = 已停止：Soniox 拒绝了该密钥。请在 设置 → 字幕 中检查。
batch-subtitles-stop-balance-empty = 已停止：Soniox 余额为空。请前往 console.soniox.com 充值。
batch-subtitles-stop-budget-used = 已停止：Soniox 的月度预算已用完。请前往 console.soniox.com 提高预算。
batch-subtitles-stop-other = 已停止：Soniox 拒绝继续（{ $error_type }）。
batch-subtitles-stop-unreachable = 已停止：无法连接 Soniox。请检查网络连接。
batch-subtitles-report-spend = Soniox：至少 { $duration } · { $usd }
batch-subtitles-report-failed-deletes = 有 { $n ->
   *[other] { $n } 个上传文件
}无法在 Soniox 上删除（见日志）
batch-subtitles-languages-need-key = 保存 Soniox 密钥后，可从其所有语言中选择。
batch-subtitles-languages-rejected = Soniox 拒绝了该密钥。
batch-subtitles-languages-failed = 无法从 Soniox 获取语言。
batch-subtitles-count = { $n ->
   *[other] { $n } 个字幕
}
batch-subtitles-duration-seconds = { $s } 秒
batch-subtitles-usd-under = 不到 $0.01
batch-subtitles-usd-about = 约 ${ $amount }

## 文件列表
folder-has-subtitles = 有字幕
folder-closed-by-job = 批量操作更改文件夹期间已关闭
folder-filter-tip = 只显示…

folder-search-placeholder = 查找文件或注释
folder-search-comments-loading = 正在搜索注释… 剩余 { $n }
folder-search-clear = 清除
folder-all = 全部
folder-invert = 反选
folder-checked = 已勾选 { $count }
folder-outcome-changed = 已更改
folder-outcome-unchanged = 无需更改
folder-outcome-failed = 失败，见日志
folder-rename-error-empty = 名称为空
folder-rename-error-bad-character = 不允许：\ / : * ? " < > |
folder-rename-error-trailing = 不能以点或空格结尾
folder-rename-error-exists = 已存在同名文件
folder-markers-not-saved = 标记未保存：文件为只读或正被占用（请在 Premiere 中关闭它，然后打开该文件再离开）
drag-out-not-saved = 未拖出：无法保存文件（只读，或在 Premiere 中打开）
folder-opening = 正在打开文件夹…
folder-empty-title = 此文件夹中没有视频
folder-empty-line = frename 显示 MP4、MOV、MKV 及其他视频文件。
folder-open-another = 打开其他文件夹…
folder-no-match = 没有匹配的文件
folder-show-all = 显示全部
folder-locked = { $action }运行期间无法勾选
folder-checked-hidden = 已勾选 { $count } · 隐藏 { $hidden }
folder-checked-hidden-tip = { $hidden ->
   *[other] 有 { $hidden } 个已勾选的文件被搜索或筛选隐藏；操作也会作用于它们
}
folder-outcome-working = 处理中
folder-outcome-not-reached = 未处理到
folder-window-empty-title = 打开素材文件夹
folder-window-empty-line = 或将文件夹拖到窗口上。右键点击该按钮可打开单个文件。
folder-window-open = 打开文件夹…

## 最近的文件夹（打开按钮旁的箭头，以及空白界面）

recent-folders-tip = 最近的文件夹
recent-folders-title = 最近的文件夹
recent-folders-none = 尚未打开过文件夹
recent-folders-not-found = 未找到
recent-folders-remove-tip = 从列表中移除
recent-folders-clear = 清空列表
recent-folders-remove = 移除
recent-folders-keep = 保留
recent-folders-missing-line = 未找到。要移除吗？
recent-folders-just-now = 刚刚
recent-folders-minutes = { $count } 分钟前
recent-folders-hours = { $count } 小时前
recent-folders-days = { $count ->
   *[other] { $count } 天前
}
recent-folders-weeks = { $count ->
   *[other] { $count } 周前
}
recent-folders-months = { $count ->
   *[other] { $count } 个月前
}
recent-folders-years = { $count ->
   *[other] { $count } 年前
}

## 文件菜单（右键点击文件，或 F11 / Shift+F11 / Ctrl+F11）

file-menu-show-in-explorer = 在资源管理器中显示
file-menu-show-in-file-manager = 在文件管理器中显示
file-menu-copy-path = 复制完整路径
file-menu-copy-name = 复制文件名
file-menu-copied = 已复制
file-menu-not-copied = 未复制：剪贴板不可用
file-menu-not-shown = 无法打开文件管理器

## 文件列表下方的控制栏

folder-controls-filter-untagged = 无标签
folder-controls-filter-subtitles = 字幕
folder-controls-filter-comments = 注释
folder-controls-filter-markers = 标记
folder-controls-scroll = 在列表中显示已打开的文件
folder-controls-open = 打开文件夹
folder-controls-batch = 对已勾选的文件执行批量操作
folder-controls-batch-back = 返回已打开的文件
app-bar-single = 单个文件
app-bar-batch = 批量
folder-controls-update-available = 有可用更新：{ $version }
folder-controls-whats-new = { $version } 的新功能
folder-controls-previous = 上一个文件
folder-controls-next = 下一个文件
folder-controls-open-file = 右键点击：打开单个文件

## 视频

video-controls-back = 后退 10 秒
video-controls-play = 播放
video-controls-pause = 暂停
video-controls-forward = 前进 10 秒
video-controls-frame-back = 后退一帧
video-controls-frame-forward = 前进一帧
video-controls-set-in = 设置入点
video-controls-set-out = 设置出点
video-controls-screenshot = 保存此帧
video-controls-add-marker = 添加标记
video-controls-add-marker-hold = 按住以标出范围；再按一次为其命名
video-controls-cannot-hold-markers = 此文件无法保存标记
video-controls-add-a-name = 添加名称
video-controls-rotate-left = 向左旋转
video-controls-rotate-right = 向右旋转
video-controls-more = 更多
media-viewer-loading-slow = 正在等待文件…（云端文件可能需要一些时间）
media-viewer-cannot-play = 无法播放此素材
media-viewer-no-picture = 此文件没有视频画面
video-timecode-copied = 已复制时间码：{ $time }
video-timecode-unreadable = “{ $text }”不是此素材中的时间
video-controls-volume-scroll = 音量：滚动调节
rotate-cannot = 无法旋转：{ $reason }
rotate-reason-missing = 文件已不存在
rotate-flag-right = 右 90°
rotate-flag-left = 左 90°
rotate-flag-half = 180°
rotate-flag-none = 无
rotate-now = 旋转：{ $flag }
rotate-turned = 已旋转 { $turn } · 当前旋转 { $flag }
rotate-failed = 未旋转：{ $reason }
undo-name-taken = 未更改：已有另一个文件名为 { $name }
undo-back-on-clip = 回到此素材（已撤销移动）。再按 Ctrl+Z 撤销它的上一次更改。
redo-on-clip = 再次回到此素材（已重做移动）。
rotate-reason-in-use = 文件为只读或正被占用
rotate-reason-format = 此格式没有旋转标志
rotate-reason-damaged = 文件已损坏
rotate-reason-no-video = 文件没有视频轨道
rotate-reason-matrix = 视频的显示矩阵不常见
batch-action-rotate = 旋转视频
batch-action-rotate-hint = 更改每个已勾选 MP4 和 MOV 文件的旋转标志；画面不会重新编码，注释、入点/出点和标记保持不变。Premiere Pro 导入时应显示旋转后的素材。其他格式没有旋转标志，会失败。再次运行会再次旋转文件；“重置”会去除所有旋转，包括手机录制时产生的旋转，因此手机拍的竖屏素材随后会横着播放。
batch-action-rotate-right = 右 90°（顺时针）
batch-action-rotate-left = 左 90°（逆时针）
batch-action-rotate-half = 180°
batch-action-rotate-reset = 重置：无旋转（0°）
media-viewer-video-subtitle-list = 字幕列表
media-viewer-video-marker-list = 标记列表
media-viewer-video-markers-hint = Shift+F1 / Shift+F3 在标记间跳转；Shift+拖动 吸附
media-viewer-video-tab-markers = 标记
media-viewer-video-fullscreen = 全屏
media-viewer-video-close-list = 关闭列表
media-viewer-video-subtitle-file-unreadable = 无法读取此文件
# 素材在上次播放停止的位置打开时，显示在画面上；按 Home 或点击回到开头。
media-viewer-video-resumed = 已从 { $time } 继续 · Home：从头开始

## 标记列表
markers-in-out = 入点/出点

markers-empty = 还没有标记
markers-add = 添加标记
markers-ai-hint = AI 标记：AI 再次描述此素材时会被替换
markers-keep-color = 保留颜色
markers-done = 完成
markers-delete = 删除标记
markers-cannot-hold-hint = Premiere 从 MP4 和 MOV 文件中读取标记。
markers-color-green = 绿色
markers-color-red = 红色
markers-color-orange = 橙色
markers-color-yellow = 黄色
markers-color-white = 白色
markers-color-blue = 蓝色
markers-color-cyan = 青色
markers-color-lavender = 淡紫色
markers-color-magenta = 洋红色
markers-color-other = 其他颜色
markers-read-only = 只读
markers-name-placeholder = 名称
markers-ai-describe = 用 AI 描述：为标记命名并添加发生的内容
markers-ai-stop = 停止描述
markers-ai-describe-unnamed = 描述 { $count } 个未命名标记
markers-ai-describe-unnamed-hint = 用 AI 为每个没有名称的标记命名并描述，一次处理几个。每个标记一次请求；其间你输入的名称会保留。
markers-ai-price-about = 约 { $dollars }
markers-ai-describe-unnamed-cost = 每个标记一次请求：每个 { $each }，全部 { $total }。
markers-ai-confirm = { $count ->
   *[other] 描述 { $count } 个标记？每个标记一次请求：共 { $total }。
}
markers-ai-confirm-unpriced = { $count ->
   *[other] 描述 { $count } 个标记？每个标记一次付费请求。
}
markers-ai-confirm-yes = 描述
markers-ai-confirm-no = 取消
markers-ai-waiting = 等待轮到它…
markers-ai-run = 正在描述 { $describing } · { $waiting } 个等待中
markers-ai-run-last = 正在描述 { $describing }
markers-ai-stop-all = 全部停止
markers-ai-describing = 正在描述…
markers-ai-done = 标记已描述
markers-ai-nothing-new = 标记已有这条描述
markers-no-marker-notice = 此处没有标记
markers-ai-no-key = 没有 { $provider } API 密钥：请在设置中填写
markers-ai-failed = 未描述：{ $reason }
markers-ai-failed-unknown = 出了点问题
markers-read-only-notice = 该标记为只读
markers-ai-gone = 标记已不存在：未添加其描述
markers-ai-stopping-for-batch = 正在先停止标记描述… 通常需要几秒钟。
folder-not-saved-marker-ai = 未保存：标记描述仍在读取素材。请稍后再试。

## 文件工作区

file-workspace-search-placeholder = 查找标签，或直接输入
file-workspace-search-clear = 清除
file-workspace-comment-placeholder = 注释...
file-workspace-comment-collapse = 返回标签
file-workspace-comment-expand = 展开注释

## 标签网格

tag-grid-star = 星标：置顶
tag-grid-unstar = 取消星标
tag-grid-save = 添加到文件夹的标签
tag-grid-delete = 从文件夹的标签中删除“{ $tag }”
tag-grid-create = 创建“{ $tag }”
tag-grid-no-file = 打开素材后才能打标签
tag-grid-no-tags = 还没有标签
tag-grid-no-tags-hint = 输入名称并按 Enter 创建第一个。
tag-grid-group-unsaved = 不在文件夹的标签中
tag-grid-group-folder = 文件夹标签
tag-grid-more = 另有 { $count } 个

## 排序栏

sync-panel-locked = 在下方调整顺序会重排整个文件夹
sync-panel-unlocked = 在下方调整顺序只改变此素材
sync-panel-unlock = 解锁
sync-panel-lock = 锁定
sync-panel-differs = 顺序与文件夹不同
sync-panel-use-for-folder = 用于文件夹
sync-panel-sort-like-folder = 与文件夹排序一致

## 文件名卡片

file-name-panel-no-tags = 此素材没有标签
file-name-panel-untag = 移除标签
file-name-panel-clear-in = 清除入点
file-name-panel-clear-out = 清除出点
file-name-panel-suggested-in-out = AI
file-name-panel-apply-suggested-in-out = 将入点和出点设为 AI 的建议（取整到秒）
file-name-panel-suggested-tags = AI 标签
file-name-panel-add-suggested-tag = 添加标签“{ $tag }”（把握 { $percent }%）
file-name-panel-add-suggested-tag-unsure = 添加标签“{ $tag }”
file-name-panel-add-all-suggested-tags = 全部添加
file-name-panel-add-all-suggested-tags-tip = 添加 AI 推荐的所有标签
file-name-panel-tag-ideas = 想法：{ $ideas }
file-name-panel-tag-ideas-tip = AI 注意到、但此文件夹没有的标签。它们不会被添加。
file-name-panel-saved = 所有更改已保存
file-name-panel-saved-to-recovery = 已保存到恢复文件，离开此素材时应用

## 更新（设置）

updates-check = 检查更新
updates-not-installed = 更新仅在已安装的版本中可用
updates-from-store = 更新来自 Microsoft Store
updates-from-releases = 新版本：请从发布页面下载
updates-open-releases = 打开发布页面
updates-checking = 正在检查…
updates-downloading-named = 正在下载 { $version }… { $percent }%
updates-downloading = 正在下载… { $percent }%
updates-restarting = 正在重启…
updates-check-failed = 无法检查更新：{ $reason }
updates-update-failed = 无法更新：{ $reason }
updates-close-other-window = 请先关闭另一个 frename 窗口
updates-reason-github = GitHub 没有响应，请稍后再试
updates-reason-no-connection = 无网络连接
updates-reason-read-only = 文件夹为只读
updates-reason-not-installed = 更新仅在已安装的版本中可用
updates-reason-download-stopped = 下载意外中断
updates-version-available = 版本 { $version } 可用
updates-up-to-date = frename 已是最新版本
updates-update-and-restart = 更新并重启
updates-wait-for-batch = 请等待批量操作完成
updates-current-version = frename { $version }
updates-check-on-start = frename 启动时检查更新
updates-notes-show = 发行说明
updates-notes-hide = 隐藏发行说明
updates-notes-all-releases = GitHub 上的所有发行版本
updates-notes-history = 所有版本，最新的在前
updates-notes-update = { $version } 带来的内容
updates-notes-whats-new = { $version } 的新功能
updates-notes-loading = 正在加载发行说明…
updates-notes-unavailable = 无法加载最新版本的说明。
updates-notes-more = 显示更多

recovery-restored = frename 意外关闭。已恢复你在 { $clip } 上未保存的工作（{ $what }）。
recovery-kept = frename 意外关闭。你在 { $clip } 上未保存的工作（{ $what }）未被应用：{ $why }。可供重新输入的可读副本在 { $folder } 中。
recovery-unreadable = frename 意外关闭，且无法读取恢复文件。它保存在 { $folder } 中。
recovery-why-gone = 素材已不存在或已被重命名
recovery-why-changed = 素材在你编辑后发生了变化
recovery-why-not-written = 已存在同名文件，或素材为只读或正被占用
recovery-tags = { $count ->
   *[other] { $count } 个标签
}
recovery-markers = { $count ->
   *[other] { $count } 个标记
}
recovery-comment = 注释
recovery-in-out = 入点/出点
recovery-edits = 编辑
folder-frame-saved = 帧已保存
folder-not-saved = 未保存：已存在同名文件，或文件为只读或正被占用
markers-deleted-notice = 标记已删除
markers-not-saved-notice = 标记未保存：文件为只读或正被占用
markers-comment-not-saved-notice = 标记未保存：注释文件为只读或正被占用
markers-copy = 复制
markers-copy-tip = 将标记复制为文本，每个一行
markers-export = 导出…
markers-export-tip = 将标记保存为文本文件
markers-exported = 标记已保存到 { $name }
markers-not-exported = 标记未保存：无法写入文件

batch-option-notes = 给 AI 的备注
batch-ai-notes-info = 你关于此文件夹素材的备注，会随此处的每次 AI 请求一起发送：描述、标签推荐和标记描述。保存在文件夹的 .frename 文件中。
batch-ai-notes-placeholder = 例如：最常出现的男人是 Eugene。标出广告牌。看到茶的地方就推荐标签 tea。
batch-ai-notes-hint = 与画面帧一起发送给 { $provider }。名字只来自这些备注；frename 不识别人脸。已描述的视频会被跳过：勾选“重新处理”可用新的备注重新描述。
batch-ai-notes-too-long = 备注有 { $chars } 个字符，上限为 { $max }。请缩短后再运行。
batch-ai-notes-too-long-reason = 给 AI 的备注太长
batch-option-notes-once = 仅本次运行的附加说明
batch-ai-notes-once-placeholder = 例如：这段素材中穿红衣服的女人是 Ana
file-workspace-ai-notes-on = 备注已开启
file-workspace-ai-notes-tip = 给 AI 的备注会随此文件夹中的每次 AI 请求一起发送。请在“用 AI 描述”（批量操作）中编辑：{ $notes }
batch-ai-notes-no-subtitles = 你的备注似乎与说话内容有关，而 { $videos } 没有字幕：请先生成字幕（生成字幕），这样 AI 才能读到台词。
