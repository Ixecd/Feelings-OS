// drivers/ear.c — 耳后设备驱动
//
// 迷走神经耳支 (VnsEarBranch) 微电流刺激
// PPG 心率血氧采集 (MAX30102)
// 骨传导音频通路 (Cochlear)
//
// FFI: Rust → extern "C" → ear_read() / ear_stimulate()
