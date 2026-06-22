// 四层缓存守护进程
//
// L0 BRAM (FPGA on-chip) → L1 线程本地 → L2 节点级 → L3 分布式
// 和 mempoold 协作——缓存命中时零拷贝直通帧 buffer
//
// 接口: /dev/cache → 读命中率/驱逐率
