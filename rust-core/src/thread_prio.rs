/// 原生线程优先级提升。
///
/// Android 的 `Process.setThreadPriority` 只覆盖 JVM 线程；Rust/tokio 工作线程、
/// AAC 编码线程仍继承默认 nice。联网 App 到前台后，后台调度会把这些线程延后，
/// 表现为音视频周期性迟到/丢帧。这里直接用 libc `setpriority` 对齐 Android 的 nice
/// 档位：核心工作线程 -4（DISPLAY），实时音频线程 -16（URGENT_AUDIO）。
pub(crate) fn apply(label: &str, nice: i32) {
    let nice = nice.clamp(-20, 19);
    let tid = unsafe { libc::gettid() };
    let rc = unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, nice) };
    let err = std::io::Error::last_os_error();
    if rc != 0 {
        crate::emit_log(&format!("THREAD-PRIO: {label} tid={tid} setpriority({nice}) 失败：{err}"));
    } else {
        crate::emit_log(&format!("THREAD-PRIO: {label} tid={tid} nice={nice}"));
    }
}
