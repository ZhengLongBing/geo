/// 几何体的逻辑维度。
///
/// 这个枚举定义了不同类型的几何维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimensions {
    /// 具有 X 和 Y 值的二维几何体
    Xy,

    /// 具有 X、Y 和 Z 值的三维几何体
    Xyz,

    /// 具有 X、Y 和 M 值的三维几何体
    Xym,

    /// 具有 X、Y、Z 和 M 值的四维几何体
    Xyzm,

    /// 逻辑类型未知的几何体。包含的 `usize` 值表示物理维度的数量。
    Unknown(usize),
}

impl Dimensions {
    /// 返回此几何体的物理维度数量。
    pub fn size(&self) -> usize {
        match self {
            Self::Xy => 2,
            Self::Xyz | Self::Xym => 3,
            Self::Xyzm => 4,
            Self::Unknown(val) => *val,
        }
    }
}
