/// 计算两个几何体之间的最小距离。
pub trait Distance<F, Origin, Destination> {
    /// 注意，并非所有的实现都支持所有几何体组合，但至少支持 `Point` 到 `Point` 的计算。
    /// 有关详细信息，请参见[具体实现](#implementors)。
    ///
    /// # 单位
    ///
    /// - `origin`, `destination`: 几何体，其中x/y的单位取决于特征实现。
    /// - 返回值: 依赖于特征实现。
    fn distance(origin: Origin, destination: Destination) -> F;
}
