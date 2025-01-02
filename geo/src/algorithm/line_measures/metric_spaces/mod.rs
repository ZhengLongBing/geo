// 开始选择
mod euclidean; // 导入欧几里得度量空间模块
pub use euclidean::Euclidean; // 公共使用欧几里得度量空间

mod geodesic; // 导入大地测度模块
pub use geodesic::Geodesic; // 公共使用大地测度

mod haversine; // 导入哈弗赛因模块
pub use haversine::Haversine; // 公共使用哈弗赛因

mod rhumb; // 导入罗盘航线模块
pub use rhumb::Rhumb; // 公共使用罗盘航线
                      // 结束选择
