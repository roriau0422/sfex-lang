// JIT Compilation Module using Cranelift
pub mod compiler;
pub mod profiler;
pub use compiler::JitCompiler;
pub use profiler::Profiler;
/// Takes a pointer to interpreter state, returns a Value
pub type JitFunction = unsafe extern "C" fn() -> i64;
