# HyperCar 0.1.0.614 六轮补丁说明（debug 出包）

基线：上游 tag `v0.1.0.614`（versionCode 10710），保留 614 全部新功能（短视频 App
投影、投影选择器、投影时暂停本地音乐等）。本机无工具链，由 Actions `debug-build.yml`
云端编译，编译错误以 CI 日志为准。

## 相对上游 614 的改动

### 锁屏无感化（消除车机“跟着锁一下再恢复”）

五轮日志显示：收到 `SCREEN_OFF` 后，代码立即 `stopLocked()` + `startLocked()`，
把旧源虚拟屏 2615 和编码器释放后新建 2617，车机可见的瞬间闪断正来自这次整源重建。

- `carui/CarUiCaster.kt`：`SCREEN_OFF` / 锁屏瞬间**不再主动重建源屏**，保留现有
  虚拟屏、编码器和首页任务；只有 display 被移除、shell 服务断开或看门狗判定真的
  帧停滞/断线时才恢复。
- `spike/SpikeShellService.kt`：建屏每个候选 flag 的 null/异常都写入 `SHELL` 报告，
  便于确认 HarmonyOS 拒绝 `OWN_DISPLAY_GROUP` / `ALWAYS_UNLOCKED` 的具体原因。

### 自定义 App 横屏适配与诊断

五轮已把展开态画布改为横屏；日志确认虚显本身已是 `2495x794 dpi=211`，但任务窗口
bounds / freeform 模式未成功，且 Android 12 不认识五轮传入的兼容 ChangeId。

- `spike/MapCompat.kt`：方向兼容命令同时尝试 ChangeId 与名称别名
  （`OVERRIDE_UNDEFINED_ORIENTATION_TO_NOSENSOR`、`OVERRIDE_ANY_ORIENTATION`），
  两条命令的 exit / out / err 都完整写入日志。
- `carui/MediaTaskWindow.kt`：expand 过程逐段记录 freeform、resize、get-mode 结果；
  任务找不到或 bounds 未匹配时导出 `am stack list` 相关 RootTask / 任务行，便于判断
  是 freeform 不支持、任务仍被锁竖屏，还是 resize 未生效。

### 线程提级（操作其他 App 时音视频卡顿）

三轮只覆盖了 JVM 采集/编码器线程；四轮把 Rust/native 与剩余转码线程也提级。

- `rust-core/src/thread_prio.rs`：新增 native nice 档位工具，失败只记日志不崩。
- `rust-core/src/lib.rs`：tokio 4 个核心工作线程启动时提至 nice=-4（DISPLAY）。
- `rust-core/src/engine.rs`：独立混音线程 `hypercar-audio` 提至 nice=-16（URGENT_AUDIO）。
- `rust-core/src/audio_out/aac_worker.rs`：AAC 编码线程提至 nice=-16。
- `core/CarWithHevcTranscoder.kt`：初始化、解码、编码线程提至 `THREAD_PRIORITY_DISPLAY`。
- `core/CodecSurfaceBridge.kt`：EGL/渲染 HandlerThread 提至 `THREAD_PRIORITY_DISPLAY`。
- 三轮保留：`CarAudioCapture.kt`、`ShellPlaybackAudio.kt`、`SplitPlaybackAudio.kt`
  读取线程紧急音频优先级；`SpikeCaster.kt` 编码器 HandlerThread 显示优先级。

### 音频采集稳健性
- `ShellPlaybackAudio.kt`：采集缓冲 38400 → 76800 字节（200ms）；短读块补零后按完整帧写出。
- `SplitPlaybackAudio.kt`：短读块补零、按完整缓冲写出。

### 弱信号宽限
- `session/HyperCarRepo.kt`：`onLost` 增加 4000ms 宽限期；期内网络重绑则保留会话，
  超时未恢复再断开。

## 出包方式

- `.github/workflows/debug-build.yml`：JDK17 + Rust aarch64-linux-android + NDK r27c +
  cargo-ndk 编译核心层，`:app:assembleDebug`，产物 `HyperCar-0.1.0.614-debug.apk`。
- 移除官方 `android-ci.yml`（缺签名 Secret 会直接失败）。
- `versionCode` 10710 → 10714；`versionName` 保持 0.1.0.614。
- debug 证书签名：安装前必须先卸载官方签名版本。

## 日志判断要点

- 六轮 `CARUI-LIFECYCLE` 息屏行应是“保留现有源屏与编码器，不主动重建”，不应再出现
  锁屏瞬间 `recovery rebuilding source`。
- `VIDEO-CARD` 新增 `expand freeform-mode` / `expand resize` / `expand final` /
  `stack:` 行；回传日志时重点看任务 bounds 是否仍是竖屏、freeform 模式是否可用。
- 新增 `THREAD-PRIO:` 行：记录各 native/JVM 线程 tid 与 nice；若 `setpriority` 失败，
  会带错误原因，现场回传时重点看。
- 仍看 `AUDIO-PACING` 的 `max_late_us` / `expired_media_frames`、`SPIKE` 的 fps 与 gap。

## 未覆盖 / 待观察

- 若目标 App 在 Manifest 中硬锁竖屏、且 Android 12/HarmonyOS 不提供方向兼容覆盖，
  系统会按 letterbox 保留竖条；App 侧无法绕过系统窗口策略。需用六轮 stack 日志确认。
- ownGroup/alwaysUnlocked 被系统拒绝的具体权限原因仍需 SHELL 段日志确认。
- 地库/隧道断连需实地复测并回传日志。
- 手机侧设置（代码不可强制）：电池优化白名单、最近任务锁卡片、允许后台活动/自启动。
