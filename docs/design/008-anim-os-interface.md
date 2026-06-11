# ADR 008: Anim ↔ Feelings-OS 硬实时帧缓冲区接口

> 日期：2026-06-10
> 状态：定稿
> 性质：Anim 九 Pass 管线与 Feelings-OS 五级调度域之间的唯一接口定义

---

## 决策

Anim 和 Feelings-OS 之间——只有帧缓冲区指针这一个接口。Anim 不调用 OS API。OS 不执行 Anim 函数。写完——指针切换。取帧——指针读取。

Anim 的两条流水线（ADR 004）各自落在 OS 的不同调度域上——编译不占用注入时间——注入在每次帧边缘精确完成。

---

## 一、双流水线 → OS 调度域映射

```
Anim Pipeline A — 离线编译             → P3 后台预交织
    .anim 源码 → Pass 0a → ... → ESIR 帧
    产出 ESIR 帧缓存 → 写入 L2 Slab 帧缓冲区
    只有 Idle 状态时跑。不抢占任何实时域。

Anim Pipeline B — 实时注入             → P1 感受域 + P1b 恢复域
    每 1ms tick → 从 ESIR 帧缓存取帧 → 写入设备寄存器
    帧内——分支预测器在 BRAM 端口 A 读上一帧反馈 → 选下一帧。
    预测错 → 安全插桩接管（P0 安域写标志 → 下一拍保护帧）。
    P1b 恢复帧 → 在 P1 剩余时隙注入——不增加延迟。
```

---

## 二、分支预测器——硬件位置

Anim ADR 007 §4.2 定义预测器逻辑。Feelings-OS 定义其物理位置：

```
位置：FPGA BRAM 双端口区（ADR 001 定义的 L0 缓存层）。
    端口 A = 感受域只读。端口 B = 安域只写（见 OS-README.md §六）。

分支预测器位于端口 A 侧：
    每 1ms tick → 读上一帧生理反馈（HRV/EDA/EEG band）→
    线性外推下一帧偏差方向 → 从 ESIR 帧缓存中选下一帧。
    纯 BRAM 寄存器操作——不经过 CPU。不经过 schedulerd。
    预测对 → 下一帧零延迟衔接。
    预测错 → 端口 B 安域写 SafetyHold 标志 → 下一拍安全插桩帧接管。

不是 CPU 指令流里的分支预测器（Smith 1981 → TAGE-SC-L）。
是感受信号注入端的"帧选器"——在 FPGA 硬件上——查寄存器——< 1μs。
```

---

## 三、NeuroEnergyTracker——漏桶→P0 熔断

Anim P0 #10 定义漏桶数学模型。Feelings-OS 定义熔断路径：

```
tracker 位置：在 personalize() 末尾——PSIR 生成后、return Ok 前——
    tracker.intake_and_verify(intensity, dimension, profile)。

漏桶累积：Energy_t = max(0, Energy_{t-1} + Intensity_t - LeakRate × Δt)

超标熔断：
    Energy_t > Critical_Threshold →
    写入 /dev/safety 熔断标志位 →
    P0 安域在下一个 1ms tick 锁定标志 → 触发保护帧。
    不是抢占——是预先置位——P1 下一帧自动从保护帧队列取帧（零额外延迟）。

保护帧：
    OiSmoothing 四帧衰减系数 × 当前帧 → 强制静默一帧 → 3-5s 内引导回安全基线。
```

---

## 四、脱敏看门狗——P0 软子层→P1b 恢复帧

Anim P0 #11 定义同一强度持续过久的脱敏检测。Feelings-OS 定义信号路径：

```
检测位置：P0 软子层（次子层）。
    同一强度/维度连续 N 帧变化幅度 < 阈值 → 触发"信号变异度过低"警告 →
    写入 /dev/safety 降级标志位。

响应时机：不抢占。不需要 100μs 响应。
    100ms 采样一次。降级标志置位后——
    P1b 恢复调度域在下一帧空闲时隙读取标志 →
    主动插入恢复帧（低强度 / 静默）。

恢复帧：
    由 Feelings-Core 强度调度器预编译 → Anim 管执行 → OS 管时序。
    P1b 在 P1 感受域注入当前帧后的剩余时隙内注入——
    不增加延迟。不抢占 P1。和超量恢复的节律同构。
```

---

## 五、接口边界一览

```
Anim 侧                    OS 侧                 物理位置
───────                    ──────                 ────────

ESIR 帧缓存写入              L2 Slab 帧缓冲区        mempoold 共享内存页
分支预测器                    BRAM 端口 A 读          FPGA 寄存器（L0）
安全插桩标志                  /dev/safety write       BRAM 端口 B 安域写
保护帧队列                    P0 安域队列             BRAM 安全帧预留页
恢复帧队列                    P1b 恢复域              L2 帧缓冲区（低优先级页）
漏桶累积能量                  /dev/safety 熔断标志     P0 软看门狗子层
脱敏标记                     /dev/safety 降级标志      P0 次子层
PLL 时钟                     timerd 主时钟脉冲        FPGA 晶振

唯一的接口——帧缓冲区的指针地址。
写完——指针切换。取帧——指针读取。
Anim 不调用 OS API。OS 不执行 Anim 函数。
双流水线的设计同构——编译和注入不共享同一个 CPU 核心——
通过硬件时钟和 BRAM 端口在物理层同步。
```

---

## 六、和已有 ADR 的咬合

```
ADR 001 — MPU/SAS          帧缓冲页——每块页只有两个 MPU 授权。
                            Anim 写入（RW）——busd/设备 读取（RO）。
                            Anim 和 OS 被 MPU 物理隔离——但共享帧缓冲页指针。

ADR 002 — busd Ping-Pong   Anim 写入 Page_A。从设备正从 Page_B 读取当前帧。
                            时钟脉冲到来 → 指针翻转。
                            没有锁。只有硬件时钟在无情地推着指针往前跳。
                            ValidatedFrame 生命周期绑定在指针翻转的窗口内。

ADR 003 — P0 熔断恢复        漏桶累积超标 → 写入 /dev/safety 标志 →
                            P0 安域锁定（100μs）→ 触发保护帧。
                            脱敏降级 → P1b 恢复帧（不抢占 P1）。

ADR 007 — Typestate 校验    Anim 写入的 ESIR 帧必须经过 UnvalidatedFrame::validate()。
                            未通过校验的帧 —— 瞬间打向 P0 熔断 —— 10ms 保底帧。
                            零拷贝。编译期 + 运行期双重闸门。

Anim 侧的                             Feelings-OS 侧的
ADR 004 §分支预测                      本文 §二（BRAM 端口 A）
ADR 004 §双流水线                      本文 §一（P3/P1/P1b）
ADR 007 §帧级预判器                    本文 §二
P0 #10 漏桶                           本文 §三
P0 #11 脱敏/超量恢复                    本文 §四
```

---

## 七、诚实的边界

```
当前所有定义——都在纸上。没有 FPGA 硬件验证。
    分支预测器在 BRAM 端口的延迟——设计 < 1μs——但没测过。
    NeuroEnergyTracker 的 LeakRate——无实证数据——第一纪元积累后标定。
    恢复帧节律模板的具体参数——需要设备上线后的真实生理数据校准。

不是"接口设计完整=系统就绪"。
是"接口设计完整=Anim 和 OS 之间——不需要再画新的线。
   剩下的——是验证——不是设计。"
```
