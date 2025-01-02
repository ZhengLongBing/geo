// 模块：point（点模块）
mod point;
pub use point::SweepPoint;

// 模块：events（事件模块）
mod events;
pub(crate) use events::{Event, EventType};

// 模块：line_or_point（线或点模块）
mod line_or_point;
pub use line_or_point::LineOrPoint;

// 模块：cross（交叉模块）
mod cross;
pub use cross::Cross;

// 模块：segment（线段模块）
mod segment;
use segment::{Segment, SplitSegments};

// 模块：active（活性模块）
mod active;
pub(super) use active::{Active, ActiveSet};

// 模块：im_segment（共享线段模块）
mod im_segment;
use im_segment::IMSegment;

// 模块：vec_set（向量集合模块）
mod vec_set;
pub(crate) use vec_set::VecSet;

// 模块：proc（处理模块）
mod proc;
use proc::Sweep;

// 模块：iter（迭代模块）
mod iter;
pub use iter::Intersections;
