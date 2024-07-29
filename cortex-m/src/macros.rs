/// Macro for sending a formatted string through an ITM channel
#[macro_export]
macro_rules! iprint {
    ($channel:expr, $s:expr) => {
        $crate::itm::write_str($channel, $s);
    };
    ($channel:expr, $($arg:tt)*) => {
        $crate::itm::write_fmt($channel, format_args!($($arg)*));
    };
}
//!#[macro_export] 属性使得宏可以被其他模块使用。
//!没有 #[macro_export] 的宏将是私有的，不能在模块外部使用。

/// Macro for sending a formatted string through an ITM channel, with a newline.
#[macro_export]
macro_rules! iprintln {
    ($channel:expr) => {
        $crate::itm::write_str($channel, "\n");
    };
    ($channel:expr, $fmt:expr) => {
        $crate::itm::write_str($channel, concat!($fmt, "\n"));
    };
    ($channel:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::itm::write_fmt($channel, format_args!(concat!($fmt, "\n"), $($arg)*));
    };
}
//!这个宏有三种不同的匹配规则，用于处理不同数量的参数。
//!这条规则匹配一个表达式参数 $channel。
//!这条规则匹配两个表达式参数 $channel 和 $fmt。
//!这条规则匹配两个以上的参数：$channel、$fmt 和一个可变数量的参数 $($arg)*。

/// Macro to create a mutable reference to a statically allocated value
///
/// This macro returns a value with type `Option<&'static mut $ty>`. `Some($expr)` will be returned
/// the first time the macro is executed; further calls will return `None`. To avoid `unwrap`ping a
/// `None` variant the caller must ensure that the macro is called from a function that's executed
/// at most once in the whole lifetime of the program.
///
/// # Notes
///
/// This macro requires a `critical-section` implementation to be set. For most single core systems,
/// you can enable the `critical-section-single-core` feature for this crate. For other systems, you
/// have to provide one from elsewhere, typically your chip's HAL crate.
///
/// For debuggability, you can set an explicit name for a singleton. This name only shows up the
/// debugger and is not referenceable from other code. See example below.
///
/// # Example
///
/// ``` no_run
/// use cortex_m::singleton;
///
/// fn main() {
///     // OK if `main` is executed only once
///     let x: &'static mut bool = singleton!(: bool = false).unwrap();
///
///     let y = alias();
///     // BAD this second call to `alias` will definitively `panic!`
///     let y_alias = alias();
/// }
///
/// fn alias() -> &'static mut bool {
///     singleton!(: bool = false).unwrap()
/// }
///
/// fn singleton_with_name() {
///     // A name only for debugging purposes
///     singleton!(FOO_BUFFER: [u8; 1024] = [0u8; 1024]);
/// }
/// ```
#[macro_export]
macro_rules! singleton {
    ($(#[$meta:meta])* $name:ident: $ty:ty = $expr:expr) => {
        $crate::_export::critical_section::with(|_| {
            // this is a tuple of a MaybeUninit and a bool because using an Option here is
            // problematic:  Due to niche-optimization, an Option could end up producing a non-zero
            // initializer value which would move the entire static from `.bss` into `.data`...
            $(#[$meta])*
            static mut $name: (::core::mem::MaybeUninit<$ty>, bool) =
                (::core::mem::MaybeUninit::uninit(), false);

            #[allow(unsafe_code)]
            let used = unsafe { $name.1 };
            if used {
                None
            } else {
                let expr = $expr;

                #[allow(unsafe_code)]
                unsafe {
                    $name.1 = true;
                    Some($name.0.write(expr))
                }
            }
        })
    };
    ($(#[$meta:meta])* : $ty:ty = $expr:expr) => {
        $crate::singleton!($(#[$meta])* VAR: $ty = $expr)
    };
}

/// ``` compile_fail
/// use cortex_m::singleton;
///
/// fn foo() {
///     // check that the call to `uninitialized` requires unsafe
///     singleton!(: u8 = std::mem::uninitialized());
/// }
/// ```
#[allow(dead_code)]
const CFAIL: () = ();

/// ```
/// #![deny(unsafe_code)]
/// use cortex_m::singleton;
///
/// fn foo() {
///     // check that calls to `singleton!` don't trip the `unsafe_code` lint
///     singleton!(: u8 = 0);
/// }
/// ```
#[allow(dead_code)]
const CPASS: () = ();

/// ```
/// use cortex_m::singleton;
///
/// fn foo() {
///     // check that attributes are forwarded
///     singleton!(#[link_section = ".bss"] FOO: u8 = 0);
///     singleton!(#[link_section = ".bss"]: u8 = 1);
/// }
/// ```
#[allow(dead_code)]
const CPASS_ATTR: () = ();
