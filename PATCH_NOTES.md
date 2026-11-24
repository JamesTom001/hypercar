# HyperCar 本地补丁说明（八轮 · 基于上游 v0.1.0.618）

基线：`fw867/hypercar` 标签 `v0.1.0.618`（versionName 0.1.0.618，上游 versionCode 10714）。
本包 `versionCode=10716`，仅含以下针对现场问题的改动。

## 1. 会话抖动保活（修「连上就闪退 / 无法保持」）
现场日志 `hypercar-log-20261124-065545` 的直接触发链：车机 Wi-Fi（network #1811）
掉线 → 事件通道 socket 被中止（os error 103）→ 会话判结束 → 首页虚显/编码器释放 →
`CarHomeActivity` 被 finish（手机端表现为退出，车机黑屏约 8s 才恢复画面）。

改动（`session/HyperCarRepo.kt` + `carui/CarUiCaster.kt`）：
- 新增 `lastWifiFlapElapsed`：`onLost` 记录掉线时刻，6s 窗口内的断开判定为「抖动」。
- 抖动时调用 `runDisconnect(reason, flap=true)`：仍结束传输会话，但
  **不拆除首页虚显与编码器**（`CarUiCaster.holdForReconnect`），并强制 `keepListening`
  保持前台服务与监听存活。
- 保活窗口 20s；期间 `stopAsync()` 自动跳过拆除，超时才真正释放（`stopAsync(force=true)`）。
- 重连成功后 `CarUiCaster.start()` 命中既有「已经在跑」分支，直接复用同一块虚显与编码器，
  只重新上报画面尺寸（触摸坐标不再按 2560x744 缩放），车机无需重新建屏。
- 重连退避：抖动场景首个重试延迟压到 ≤800ms（原 2000ms）。

## 2. 音频采集缓冲与诊断（沿用七轮）
`spike/ShellPlaybackAudio.kt`：REMOTE_SUBMIX 采集缓冲 38400 → 153600（约 800ms），
新增 `shortReads` / `totalKB` 统计并打入 CARUI-AUDIO 五行日志。

## 3. 音频节奏恢复放宽 + 统计回绕修正
- `rust-core/src/audio_pacing.rs`：`recoverable_source_frames` 由「1 块、queued≥704」
  放宽为「最多 2 块、晚 <3 slot、queued≥1056」，容忍 worker 一次短停顿。
- `rust-core/src/engine.rs`：`skipped = missed - recovered` 改
  `missed.saturating_sub(recovered)`，修复日志里
  `skipped_source_frames=18446744073709551264` 的 u64 下溢（放大恢复窗口后必然触发）。

## 4. 未采用的方案
- 自定义 App 软件横屏：HarmonyOS user build 拒绝方向兼容覆盖，收益为零（用户已放弃）。
- FFmpeg 软编整体替换：Surface 无法直接喂 FFmpeg，需 ImageReader/EGL + x264(GPL)，
  前台抢占下更易卡，不推荐。
