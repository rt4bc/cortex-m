//! Instrumentation Trace Macrocell
//!
//! **NOTE** This module is only available on ARMv7-M and newer.

use core::{fmt, ptr, slice};

use crate::peripheral::itm::Stim;

//这里的bytes类型是u32，就是保证了32bit对齐
// NOTE assumes that `bytes` is 32-bit aligned
unsafe fn write_words(stim: &mut Stim, bytes: &[u32]) {
    let mut p = bytes.as_ptr();
    for _ in 0..bytes.len() {
        while !stim.is_fifo_ready() {}
        stim.write_u32(ptr::read(p));
        p = p.offset(1);
    }
}
//安全性：由于 offset 和 read 是不安全的操作，必须放在 unsafe 块中使用。
//使用 offset 时，需要确保指针不会越界；使用 read 时，需要确保指针指向的是有效的内存区域。
//offset 用于将指针移动到指定位置（正向或反向），适用于内存遍历或指针运算。
//read 用于从指针指向的内存中读取值，适用于读取内存中的数据而不改变所有权。

/// Writes an aligned byte slice to the ITM.
///
/// `buffer` must be 4-byte aligned.
/// 注意这里的buffer 是u8 类型
unsafe fn write_aligned_impl(port: &mut Stim, buffer: &[u8]) {
    let len = buffer.len();

    if len == 0 {
        return;
    }
    
    //Clippy: Clippy 是一个 Rust 静态分析工具，提供了一组 lint（静态分析检查）
    //来帮助开发者识别和纠正可能存在的错误、非最佳实践或潜在的问题。
    //Clippy 的检查范围从性能优化建议到可能导致未定义行为的代码模式。
    
    //Lint: Lint 是对代码的静态检查，可以识别潜在的错误或非最佳实践。
    //Rust 的 Clippy 工具提供了大量的 lint 来帮助保持代码的质量

    // 当 Clippy 发现代码中存在从一个对齐要求较高的类型的指针强制转换为对齐要求较低的类型时，
    // 它可能会触发 cast_ptr_alignment lint。例如，将一个 *const u64 类型的指针转换为 
    //*const u8 类型的指针可能会触发这个 lint，因为 u64 通常需要 8 字节对齐，而 u8 只需要 1 字节对齐。
    
    //以下lint告诉 Clippy 忽略某段代码中的 cast_ptr_alignment lint
    let split = len & !0b11;
    #[allow(clippy::cast_ptr_alignment)]
    write_words(
        port,
        slice::from_raw_parts(buffer.as_ptr() as *const u32, split >> 2),
    );

    // 3 bytes or less left
    let mut left = len & 0b11;
    let mut ptr = buffer.as_ptr().add(split);

    // at least 2 bytes left
    if left > 1 {
        while !port.is_fifo_ready() {}

        #[allow(clippy::cast_ptr_alignment)]
        port.write_u16(ptr::read(ptr as *const u16));

        ptr = ptr.offset(2);
        left -= 2;
    }

    // final byte
    if left == 1 {
        while !port.is_fifo_ready() {}
        port.write_u8(*ptr);
    }
}

// 这里首先声明一个单变量的结构体元组， 元组的成员是一个stim变量的引用。用的是括号，不是花括号！
// 所以后面引用的时候，直接是self.0
// 这个元组结构体初始化的时候， 标记的stim生命周期是'p, 那么也决定了Port的生命周期也是'p
// 然后是生命的关联函数Write
//如果关联函数的签名中包含引用，并且这些引用与某个结构体或泛型参数的生命周期相关，那么就需要标注生命周期。
struct Port<'p>(&'p mut Stim);

impl<'p> fmt::Write for Port<'p> {
    #[inline]
    //write_str 方法: 这是实现 fmt::Write 的核心方法，它接收一个字符串切片 &str，然后执行写操作。
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_all(self.0, s.as_bytes());
        //Ok(()) 表示写操作成功，符合 fmt::Result 类型的要求。
        Ok(())
    }
}

/// A wrapper type that aligns its contents on a 4-Byte boundary.
///
/// ITM transfers are most efficient when the data is 4-Byte-aligned. This type provides an easy
/// way to accomplish and enforce such an alignment.
#[repr(align(4))]
pub struct Aligned<T: ?Sized>(pub T);
//#[repr(align(4))] 是一个编译器属性，用于指定结构体或联合体的对齐要求。
//在这个例子中，它强制 Aligned 结构体的内存对齐为 4 字节。

//T: ?Sized：T 是一个泛型参数，类型为 T 的大小可以是已知的或未知的（即可以是 Sized 或 ?Sized）。
//?Sized 是 Rust 中的一个特殊约束，表示类型 T 可能是不定大小的（比如动态大小类型，如 [T]、str 或某些 trait 对象）。

/// Writes `buffer` to an ITM port.
#[allow(clippy::missing_inline_in_public_items)]
// Clippy 会在检测到公共 API 项（如 pub fn 或 pub const）未标注 #[inline] 时触发这个 lint
pub fn write_all(port: &mut Stim, buffer: &[u8]) {
    unsafe {
        let mut len = buffer.len();
        let mut ptr = buffer.as_ptr();

        if len == 0 {
            return;
        }

        // 0x01 OR 0x03
        if ptr as usize % 2 == 1 {
            while !port.is_fifo_ready() {}
            port.write_u8(*ptr);

            // 0x02 OR 0x04
            ptr = ptr.offset(1);
            len -= 1;
        }

        // 0x02
        if ptr as usize % 4 == 2 {
            if len > 1 {
                // at least 2 bytes
                while !port.is_fifo_ready() {}

                // We checked the alignment above, so this is safe
                #[allow(clippy::cast_ptr_alignment)]
                port.write_u16(ptr::read(ptr as *const u16));

                // 0x04
                ptr = ptr.offset(2);
                len -= 2;
            } else {
                if len == 1 {
                    // last byte
                    while !port.is_fifo_ready() {}
                    port.write_u8(*ptr);
                }

                return;
            }
        }

        // The remaining data is 4-byte aligned, but might not be a multiple of 4 bytes
        write_aligned_impl(port, slice::from_raw_parts(ptr, len));
        //slice::from_raw_parts 是 Rust 标准库中用于创建切片（slice）的一个非常重要的函数。
        //它允许你从一个指针和一个长度构造一个切片，切片是 Rust 中常用的用于引用连续内存区域的类型。
    }
}

/// Writes a 4-byte aligned `buffer` to an ITM port.
///
/// # Examples
///
/// ```no_run
/// # use cortex_m::{itm::{self, Aligned}, peripheral::ITM};
/// # let port = unsafe { &mut (*ITM::PTR).stim[0] };
/// let mut buffer = Aligned([0; 14]);
///
/// buffer.0.copy_from_slice(b"Hello, world!\n");
///
/// itm::write_aligned(port, &buffer);
///
/// // Or equivalently
/// itm::write_aligned(port, &Aligned(*b"Hello, world!\n"));
/// ```
#[allow(clippy::missing_inline_in_public_items)]
pub fn write_aligned(port: &mut Stim, buffer: &Aligned<[u8]>) {
    unsafe { write_aligned_impl(port, &buffer.0) }
}

/// Writes `fmt::Arguments` to the ITM `port`
#[inline]
pub fn write_fmt(port: &mut Stim, args: fmt::Arguments) {
    use core::fmt::Write;

    Port(port).write_fmt(args).ok();
}

/// Writes a string to the ITM `port`
#[inline]
pub fn write_str(port: &mut Stim, string: &str) {
    write_all(port, string.as_bytes())
}
