use super::Kernel;
use crate::CoordNum;

/// 简单内核提供谓词的直接实现。
/// 这些用于精确算术有符号类型（例如 i32, i64）。
#[derive(Default, Debug)]
pub struct SimpleKernel;

// 为SimpleKernel实现Kernel特性，适用于实现CoordNum特性的类型T。
impl<T: CoordNum> Kernel<T> for SimpleKernel {}
