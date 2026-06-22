// Feelings-OS — animi v2.0 从 01 裸奔的底座
//
// 六个守护进程、一切皆文件、Unix 哲学 x 硬实时双调度
// 双层语言: Rust 上层守护进程 / C 底层硬件驱动
// 单一地址空间——无内核态用户态切换，物理连续内存池
//
// 启动序列: mempoold → cached → busd → timerd → schedulerd → Idle
// 从加电到可服务 < 2s
//
// License: MIT

pub mod mempoold;
pub mod cached;
pub mod busd;
pub mod timerd;
pub mod schedulerd;
pub mod logd;
pub mod dev;
