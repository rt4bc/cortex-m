//! Low level access to Cortex-M processors
//!
//! This crate provides:
//!
//! - Access to core peripherals like NVIC, SCB and SysTick.
//! - Access to core registers like CONTROL, MSP and PSR.
//! - Interrupt manipulation mechanisms
//! - Safe wrappers around Cortex-M specific instructions like `bkpt`
//!
//! # Optional features
//!
//! ## `critical-section-single-core`
//!
//! This feature enables a [`critical-section`](https://github.com/rust-embedded/critical-section)
//! implementation suitable for single-core targets, based on disabling interrupts globally.
//!
//! It is **unsound** to enable it on multi-core targets or for code running in unprivileged mode,
//! and may cause functional problems in systems where some interrupts must not be disabled
//! or critical sections are managed as part of an RTOS. In these cases, you should use
//! a target-specific implementation instead, typically provided by a HAL or RTOS crate.
//!
//! ## `cm7-r0p1`
//!
//! This feature enables workarounds for errata found on Cortex-M7 chips with revision r0p1. Some
//! functions in this crate only work correctly on those chips if this Cargo feature is enabled
//! (the functions are documented accordingly).
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.61 and up. It *might*
//! compile with older versions but that may change in any new patch release.

#![deny(missing_docs)]
//!这条指令告诉编译器，如果有缺少文档注释的公共项（如公有函数、结构体、模块等），就会触发编译错误。
//!这有助于确保代码的公共 API 有详细的文档注释，从而提高代码的可维护性和可读性。
#![no_std]
//!这条指令禁用标准库，使得代码只能使用 core 库中的功能
#![allow(clippy::identity_op)]
//!这条指令告诉 clippy 静态分析工具不要对“恒等操作”发出警告。例如，不要警告类似于 x + 0 或 y * 1 
//!这样的操作,尽管这些操作在数学上是多余的，但有时候它们可能用于保持代码的一致性或提高可读性。
#![allow(clippy::missing_safety_doc)]
//!这条指令告诉 clippy 不要对缺少安全性注释的 unsafe 代码块发出警告。在 Rust 中，unsafe 代码块
//!需要特别小心处理，并且通常应该有详细的注释解释为什么这个代码块是安全的。
//!这条指令允许在缺少这些注释的情况下通过静态分析。
// Prevent clippy from complaining about empty match expression that are used for cfg gating.
#![allow(clippy::match_single_binding)]
//!这条指令告诉 clippy 不要对只有一个分支的 match 表达式发出警告。通常，只有一个分支的 match 
//!表达式可以被简化为其他更简洁的表达方式，但有时候为了未来的扩展或条件编译（cfg gating），
//!可能需要保留这样的 match 表达式。
// This makes clippy warn about public functions which are not #[inline].
//
// Almost all functions in this crate result in trivial or even no assembly.
// These functions should be #[inline].
//
// If you do add a function that's not supposed to be #[inline], you can add
// #[allow(clippy::missing_inline_in_public_items)] in front of it to add an
// exception to clippy's rules.
//
// This should be done in case of:
//  - A function containing non-trivial logic (such as itm::write_all); or
//  - A generated #[derive(Debug)] function (in which case the attribute needs
//    to be applied to the struct).
#![deny(clippy::missing_inline_in_public_items)]
// Don't warn about feature(asm) being stable on Rust >= 1.59.0
#![allow(stable_features)]

#[macro_use]
mod macros;
//!#[macro_use] 是一个属性，用于将一个模块中的宏定义导入到当前作用域。通常，它在模块声明之前使用。
//!这允许你在使用宏时不必显式地使用模块路径，从而简化宏的调用。
//!而不需要使用 macros::my_macro!() 这样的完全限定名

pub mod asm;
#[cfg(armv8m)]
pub mod cmse;
pub mod delay;
pub mod interrupt;
#[cfg(all(not(armv6m), not(armv8m_base)))]
pub mod itm;
pub mod peripheral;
pub mod register;

pub use crate::peripheral::Peripherals;

#[cfg(all(cortex_m, feature = "critical-section-single-core"))]
mod critical_section;

/// Used to reexport items for use in macros. Do not use directly.
/// Not covered by semver guarantees.
#[doc(hidden)]
pub mod _export {
    pub use critical_section;
}
