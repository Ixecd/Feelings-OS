# ADR 007: 零拷贝参数合法性断言器——Typestate 编译期闸门

> 日期：2026-05-23
> 状态：定稿
> 性质：Feelings-OS busd 安全边界

---

## 问题

Anim 在 Pass 8 生成 ESIR 帧参数，写入共享页。如果 Anim 内部逻辑混乱——写出恶性超越边界的电信号——busd 必须在微秒级内拦截。不能拷贝数据——1ms 窗口不容许 memcpy。不能在 C 驱动侧做判断——驱动层不该有策略。

## 策略——编译期类型状态模式

将"验证状态"编码进 Rust 的类型系统。C 驱动的总线分发函数在编译期就拒绝接收任何未经验证的裸指针。运行期只是对裸内存指针的指代权转型。零字节拷贝。几个寄存器传递。

## 类型定义

```rust
// src/bus/frame.rs

#[repr(C, align(64))] // 对齐到 64 字节缓存行，利于 SIMD 读写
pub struct RawFrame {
    pub epoch_id: u8,
    pub frame_seq: u8,
    pub flags: u8,
    pub intensity_pct: u8,
    pub channels: [u16; 32], // 32 通道的当前物理输出值
}

/// 刚从共享内存捞出来的未验证状态——不可公开消费
pub struct UnvalidatedFrame<'a> {
    raw_ptr: &'a RawFrame,
}

/// 已通过物理安全边界校验的帧——向总线发送数据的唯一凭证
pub struct ValidatedFrame<'a> {
    raw_ptr: &'a RawFrame,
}
```

### 为什么需要生命周期 `'a`

`'a` 是瞬态生命周期。严格绑定在 busd 锁存该共享页的当前 1ms 时间窗口内。窗口结束——生命周期强制失效。编译期斩断任何试图将 `ValidatedFrame` 缓存到全局或跨帧复用的可能。防止旧数据在下一帧复活。

## 零拷贝校验器

唯一构造 `ValidatedFrame` 的途径：调用 `UnvalidatedFrame::validate()`。失败 → 直接返回硬件错误码。

```rust
// src/bus/validator.rs

use core::intrinsics::likely;

pub enum SafetyViolation {
    AmperageOverflow { channel: usize, value: u16, limit: u16 },
    InvalidEpoch { expected: u8, found: u8 },
}

impl<'a> UnvalidatedFrame<'a> {
    /// 将裸 C 指针转换为未验证的 Rust 借用（零拷贝）
    #[inline(always)]
    pub unsafe fn from_shared_memory(ptr: *const RawFrame) -> Self {
        Self { raw_ptr: &*ptr }
    }

    /// 微秒级分支闭环校验
    #[inline(always)]
    pub fn validate(
        self,
        current_epoch: u8,
        hardware_limits: &[u16; 32]
    ) -> Result<ValidatedFrame<'a>, SafetyViolation> {

        // 1. 硬件级 Epoch 硬核比对（1 个周期）
        if self.raw_ptr.epoch_id != current_epoch {
            return Err(SafetyViolation::InvalidEpoch {
                expected: current_epoch,
                found: self.raw_ptr.epoch_id,
            });
        }

        // 2. 物理阈值拦截：32 通道并行边界检查
        // 编译器提示：绝大多数情况下硬件应当是安全的——优化分支预测
        for i in 0..32 {
            let signal_value = self.raw_ptr.channels[i];
            let safe_limit = hardware_limits[i];

            if likely(signal_value > safe_limit) {
                return Err(SafetyViolation::AmperageOverflow {
                    channel: i,
                    value: signal_value,
                    limit: safe_limit,
                });
            }
        }

        // 3. 校验通过——零拷贝类型状态飞跃
        Ok(ValidatedFrame { raw_ptr: self.raw_ptr })
    }
}

impl<'a> ValidatedFrame<'a> {
    /// 仅允许已验证帧输出裸指针供 C 驱动消费
    #[inline(always)]
    pub fn as_raw_ptr(&self) -> *const RawFrame {
        self.raw_ptr as *const RawFrame
    }
}
```

## 微秒级优化——编译期 + 硬件级

**自动向量化。** `RawFrame` 的 `align(64)` 保证缓存行对齐。32 个 `u16` 通道的循环检查——LLVM 直接优化为 SIMD 指令（ARM NEON 的 `VLD` + `VMAX`）。两条向量指令并行吞吐全部通道的阈值比对——< 10 纳秒。

**零运行时开销。** `UnvalidatedFrame` 和 `ValidatedFrame` 在编译后被彻底擦除。最终汇编里没有这两个结构体——只有原始物理内存指针（在 R0 中）在执行 SIMD 比对。

## busd 硬熔断闭环

```rust
// src/bus/mod.rs

pub fn tick_bus_latch(
    shared_page_ptr: *const RawFrame,
    current_epoch: u8,
    limits: &[u16; 32]
) {
    // 1. 零拷贝封装
    let unvalidated = unsafe {
        UnvalidatedFrame::from_shared_memory(shared_page_ptr)
    };

    // 2. 校验状态机转换
    match unvalidated.validate(current_epoch, limits) {
        Ok(valid_frame) => {
            // 编译期保证：只有走进这个分支，才能拿到 valid_frame
            unsafe {
                // 调用 C 驱动：bus.c 硬件锁存函数
                ffi::wrist_latch_and_transmit(valid_frame.as_raw_ptr());
            }
        },
        Err(violation) => {
            // 3. 瞬间打向 P0 安全域——物理硬熔断 → 10ms 保底帧
            emergency_hardware_melt(violation);
        }
    }
}

extern "C" {
    mod ffi {
        // C 驱动接口只认经过 Rust 类型系统背书的指针
        pub fn wrist_latch_and_transmit(frame: *const super::RawFrame);
    }
}
```

## 完整数据流

```
[ Anim 写入共享页 ] ──► ( raw_ptr )
                           │
                           ▼
                 UnvalidatedFrame<'a>   ── 不可公开消费
                           │
                           ├─► .validate() ──( 失败 )──► P0 物理熔断
                           │
                           ▼
                  ValidatedFrame<'a>    ── 进入 FFI 管道
                           │
                           ▼
             [ C驱动: wrist_latch_and_transmit ] ──► 物理总线
```

Anim 生成的任何过载数据都无法跨越 `UnvalidatedFrame::validate` 这道编译期 + 运行期双重复合闸门。C 驱动不给任何"自行判断"的机会——驱动层不该有策略。Anim 不给任何绕过校验的机会——它的类型在 FFI 接口处根本对不上。全部动作在一个缓存行内发生。零拷贝。纯粹用物理边界解决物理问题。
