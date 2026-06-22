// 内存池守护进程
//
// Page/Chunk/Slab/Zone 四级预分配。零动态分配，零碎片，零 GC。
// 物理连续内存，MPU 区域隔离替代 MMU 页表。
//
// 接口: /dev/mempool → 读当前分配状态
