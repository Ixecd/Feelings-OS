// /dev 文件接口——一切皆文件
//
// /dev/ear       迷走神经耳支刺激 + 心率采集          (Rust ← C driver)
// /dev/wrist     皮肤电导采集 + 温度控制               (Rust ← C driver)
// /dev/neck      本体感受采集 + 低频振动               (Rust ← C driver)
// /dev/temple    EEG 采集 + 认知状态信号               (Rust ← C driver)
// /dev/safety    安域状态 (只读)
// /dev/mempool   内存池状态 (只读)
// /dev/cache     缓存命中率/驱逐率 (只读)
//
// Unix 哲学: open/read/write/close。小组件 + 管道组合。
// FFI 边界: Rust → extern "C" fn → C 驱动。C 驱动 → 共享帧缓冲区 → Rust。
