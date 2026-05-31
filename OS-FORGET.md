# OS-FORGET.md — 待修复项（P0 + P1）

> 扫描日期：2026-05-21
> 范围：架构规范（`Feelings-OS.md`）+ 代码（零行）
> 原则：只列 P0（生产命门）和 P1（功能受限）

---

## P0 — 生产命门（OS 不存在就无法工作）— 0/6

1. **mempoold 零代码** — 内存池四级预分配 + 空闲链表未实现。FIXME: v0.2。

2. **schedulerd 零代码** — P0 安域硬实时抢占调度器未实现。FIXME: v0.3。

3. **busd 零代码** — 总线驱动 + 主时钟分发未实现。FIXME: v0.2（总线骨架）+ v0.4（设备驱动）。

4. **timerd 零代码** — PLL 锁相时钟源未实现。FIXME: v0.2。

5. **/dev 文件接口零代码** — 设备注册 + open/read/write/close 未实现。FIXME: v0.2。

6. **C 驱动层零代码** — ear.c / wrist.c / neck.c / temple.c / bus.c 全部未实现。FIXME: v0.4。

---

## P1 — 功能受限（规模化前必做）— 0/7

7. **cached 零代码** — L0 BRAM 分区 + L1/L2 映射未实现。FIXME: v0.5。

8. **帧缓冲区零拷贝零设计** — mmap 页切换机制未落地。FIXME: v0.4。

9. **中断向量表零配置** — P0 安域最高优先级未在硬件层配置。FIXME: v0.3。

10. **animi 集成零验证** — animi 作为独立进程在 Feelings-OS 上运行未测试。FIXME: v0.5。

11. **storaged 零设计** — 当前六个守护进程无一负责非易失持久化。PBM 几万维超维向量每次 session 后微调——不能丢。Session 生理记录不能丢。logd 审计日志需落盘。需新增 `storaged` 或扩 `mempoold`。FIXME: v0.4。

12. **FPGA 选型未定** — Artix-7 vs Zynq vs iCE40 vs Cyclone V。选型决定 DSP Block 数量、BRAM 大小、功耗上限、开发工具链。1ms 帧周期 + <100μs 安全域中断响应需硬件验证。ESIR 帧格式序列化（JSON→字节流）需在选型后与 FPGA 固件一起验证。FIXME: 阶段一硬件研究。

13. **设备驱动层与 Anim 边界已明确** — Anim 只输出 ESIR 字节流。驱动在 Feelings-OS：busd 总管，ear.c/wrist.c/neck.c/temple.c 各自负责。Anim v0.x 阶段不需要驱动——只做编译器前端。驱动是阶段三之后的事。✅ 架构已定，代码零行。

14. **进程/线程/协程在 Feelings 生态中的位置已明确** — 三层抽象，各守各的边界。FPGA 层 0-4：没有进程/线程/协程——只有门级并行——所有计算空间上同时跑，延迟物理确定。Feelings-OS 层 5-10：六个守护进程 + schedulerd 四级抢占线程——P0 安域永远优先——不是 pthread_create——是硬件线程绑定核心——无内核态切换。Anim/Feelings-Server/KubePivot：async/goroutine——在非实时路径上——网络 IO 密集——离线编译——不在硬实时路径上。硬实时路径上永远没有协程 yield——没有等待——不需要。✅ 边界清晰，每一层用最合适的。

---

## 编辑记录

```
2026-05-30  v0.1.2 进程/线程/协程边界
            - P1 新增 #14：进程/线程/协程在Feelings生态中的位置已明确
            - FPGA无进程/线程/协程——门级并行
            - Feelings-OS硬实时抢占线程——无CFS
            - Anim/Server/KubePivot用async/goroutine——不在硬实时路径

2026-05-30  v0.1.1 架构审计
            - P1 新增 3 项（4→7）：
              11. storaged 零设计——PBM/session/logd 需持久化
              12. FPGA 选型未定——硬件验证阻塞项
              13. 驱动与Anim边界已明确——Anim不管驱动，Feelings-OS管
            - P0 不变（6/6 代码零行）

2026-05-21  v0.1 初始扫描
            - 架构规范 100%
            - 代码 0%
            - P0 6 项：mempoold / schedulerd / busd / timerd / dev接口 / C驱动
            - P1 4 项：cached / fbuf / 中断向量表 / animi集成
```
