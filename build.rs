// build.rs — 编译 C 硬件驱动
//
// cc crate 编译 drivers/*.c → 静态链接进 Rust 二进制
// Rust 上层通过 extern "C" fn 调用 C 驱动

fn main() {
    cc::Build::new()
        .files(&[
            "drivers/bus.c",
            "drivers/ear.c",
            "drivers/wrist.c",
            "drivers/neck.c",
            "drivers/temple.c",
        ])
        .compile("feelings-drivers");
}
