use geo::bool_ops::OpType as BoolOp;
use geo::relate::IntersectionMatrix;
use geo::{Geometry, Point};
use serde::{Deserialize, Deserializer};

use super::Result;

/// 这些结构体所代表的XML示例
///
/// ```xml
/// <run>
/// <precisionModel scale="1.0" offsetx="0.0" offsety="0.0"/>
///
/// <case>
/// <desc>AA 不相交</desc>
/// <a>
/// POLYGON(
/// (0 0, 80 0, 80 80, 0 80, 0 0))
/// </a>
/// <b>
/// POLYGON(
/// (100 200, 100 140, 180 140, 180 200, 100 200))
/// </b>
/// <test><op name="relate" arg3="FF2FF1212" arg1="A" arg2="B"> true </op>
/// </test>
/// <test>  <op name="intersects" arg1="A" arg2="B">   false   </op></test>
/// <test>  <op name="contains" arg1="A" arg2="B">   false   </op></test>
/// </case>
/// </run>
/// ```
#[derive(Debug, Deserialize)]
pub(crate) struct Run {
    #[serde(rename = "precisionModel", default)]
    pub precision_model: Option<PrecisionModel>, // 可选的精度模型

    #[serde(rename = "case")]
    pub cases: Vec<Case>, // 测试用例集合
}

#[derive(Debug, Deserialize)]
pub(crate) struct PrecisionModel {
    #[serde(rename = "type", default)]
    pub ty: String, // 类型
}

#[derive(Debug, Deserialize)]
pub(crate) struct Case {
    #[serde(default)]
    pub(crate) desc: String, // 描述信息

    #[serde(deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) a: Geometry, // 几何对象A

    #[serde(deserialize_with = "deserialize_opt_geometry", default)]
    pub(crate) b: Option<Geometry>, // 可选的几何对象B

    #[serde(rename = "test", default)]
    pub(crate) tests: Vec<Test>, // 测试集合
}

#[derive(Debug, Deserialize)]
pub(crate) struct Test {
    #[serde(rename = "op")]
    pub(crate) operation_input: OperationInput, // 操作输入信息
}

#[derive(Debug, Deserialize)]
pub struct CentroidInput {
    pub(crate) arg1: String, // 参数1

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_point")]
    pub(crate) expected: Option<geo::Point>, // 期望的结果（可选）
}

#[derive(Debug, Deserialize)]
pub struct ConvexHullInput {
    pub(crate) arg1: String, // 参数1

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) expected: geo::Geometry, // 期望的结果
}

#[derive(Debug, Deserialize)]
pub struct EqualsTopoInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool, // 期望的布尔值结果
}

#[derive(Debug, Deserialize)]
pub struct IntersectsInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool, // 期望的布尔值结果
}

#[derive(Debug, Deserialize)]
pub struct IsValidInput {
    pub(crate) arg1: String, // 参数1

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool, // 期望的布尔值结果
}

#[derive(Debug, Deserialize)]
pub struct RelateInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "arg3", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: IntersectionMatrix, // 期望的交集矩阵
}

#[derive(Debug, Deserialize)]
pub struct ContainsInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool, // 期望的布尔值结果
}

#[derive(Debug, Deserialize)]
pub struct WithinInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool, // 期望的布尔值结果
}

#[derive(Debug, Deserialize)]
pub struct OverlayInput {
    pub(crate) arg1: String, // 参数1
    pub(crate) arg2: String, // 参数2

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) expected: geo::Geometry<f64>, // 期望的几何结果
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name")]
pub(crate) enum OperationInput {
    #[serde(rename = "contains")]
    ContainsInput(ContainsInput), // 包含操作输入

    #[serde(rename = "getCentroid")]
    CentroidInput(CentroidInput), // 中心点操作输入

    #[serde(rename = "convexhull")]
    ConvexHullInput(ConvexHullInput), // 凸包操作输入

    #[serde(rename = "equalsTopo")]
    EqualsTopoInput(EqualsTopoInput), // 拓扑相等操作输入

    #[serde(rename = "intersects")]
    IntersectsInput(IntersectsInput), // 相交操作输入

    #[serde(rename = "isValid")]
    IsValidInput(IsValidInput), // 验证操作输入

    #[serde(rename = "relate")]
    RelateInput(RelateInput), // 关系操作输入

    #[serde(rename = "union")]
    UnionInput(OverlayInput), // 合并（并集）操作输入

    #[serde(rename = "intersection")]
    IntersectionInput(OverlayInput), // 交集操作输入

    #[serde(rename = "difference")]
    DifferenceInput(OverlayInput), // 差分操作输入

    #[serde(rename = "symdifference")]
    SymDifferenceInput(OverlayInput), // 对称差操作输入

    #[serde(rename = "within")]
    WithinInput(WithinInput), // 包含于操作输入

    #[serde(other)]
    Unsupported, // 不支持的操作输入
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Centroid {
        subject: Geometry,       // 主体几何
        expected: Option<Point>, // 期望的结果（可选点）
    },
    Contains {
        subject: Geometry, // 主体几何
        target: Geometry,  // 目标几何
        expected: bool,    // 期望的布尔值结果
    },
    IsValidOp {
        subject: Geometry, // 主体几何
        expected: bool,    // 期望的布尔值结果
    },
    Within {
        subject: Geometry, // 主体几何
        target: Geometry,  // 目标几何
        expected: bool,    // 期望的布尔值结果
    },
    ConvexHull {
        subject: Geometry,  // 主体几何
        expected: Geometry, // 期望的几何结果
    },
    EqualsTopo {
        a: Geometry,    // 几何对象A
        b: Geometry,    // 几何对象B
        expected: bool, // 期望的布尔值结果
    },
    Intersects {
        subject: Geometry, // 主体几何
        clip: Geometry,    // 剪切几何
        expected: bool,    // 期望的布尔值结果
    },
    Relate {
        a: Geometry,                  // 几何对象A
        b: Geometry,                  // 几何对象B
        expected: IntersectionMatrix, // 期望的交集矩阵
    },
    BooleanOp {
        a: Geometry<f64>,        // 几何对象A
        b: Geometry<f64>,        // 几何对象B
        op: BoolOp,              // 布尔操作类型
        expected: Geometry<f64>, // 期望的几何结果
    },
    ClipOp {
        a: Geometry<f64>,        // 几何对象A
        b: Geometry<f64>,        // 几何对象B
        invert: bool,            // 反转标识
        expected: Geometry<f64>, // 期望的几何结果
    },
    Unsupported {
        #[allow(dead_code)]
        reason: String, // 不支持的原因
    },
}

impl OperationInput {
    pub(crate) fn into_operation(self, case: &Case) -> Result<Operation> {
        let geometry = &case.a;
        match self {
            Self::CentroidInput(centroid_input) => {
                assert_eq!("A", centroid_input.arg1);
                Ok(Operation::Centroid {
                    subject: geometry.clone(),
                    expected: centroid_input.expected,
                })
            }
            Self::ConvexHullInput(convex_hull_input) => {
                assert_eq!("A", convex_hull_input.arg1);
                Ok(Operation::ConvexHull {
                    subject: geometry.clone(),
                    expected: convex_hull_input.expected,
                })
            }
            Self::EqualsTopoInput(equals_topo_input) => {
                assert_eq!("A", equals_topo_input.arg1);
                assert_eq!("B", equals_topo_input.arg2);
                assert!(case.b.is_some(), "equalsTopo测试用例必须包含几何对象B");
                Ok(Operation::EqualsTopo {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    expected: equals_topo_input.expected,
                })
            }
            Self::IntersectsInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                assert!(case.b.is_some(), "intersects测试用例必须包含几何对象B");
                Ok(Operation::Intersects {
                    subject: geometry.clone(),
                    clip: case.b.clone().expect("没有几何对象B在测试用例中"),
                    expected: input.expected,
                })
            }
            Self::RelateInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                assert!(case.b.is_some(), "relate测试用例必须包含几何对象B");
                Ok(Operation::Relate {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    expected: input.expected,
                })
            }
            Self::ContainsInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                Ok(Operation::Contains {
                    subject: geometry.clone(),
                    target: case.b.clone().expect("没有几何对象B在测试用例中"),
                    expected: input.expected,
                })
            }
            Self::WithinInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                Ok(Operation::Within {
                    subject: geometry.clone(),
                    target: case.b.clone().expect("没有几何对象B在测试用例中"),
                    expected: input.expected,
                })
            }
            Self::UnionInput(input) => {
                validate_boolean_op(
                    &input.arg1,
                    &input.arg2,
                    geometry,
                    case.b.as_ref().expect("没有几何对象B在测试用例中"),
                )?;
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    op: BoolOp::Union,
                    expected: input.expected,
                })
            }
            Self::IntersectionInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);

                // 在geo中裁剪线字符串就像是在JTS中进行线与多边形的交集
                match (
                    geometry,
                    case.b.as_ref().expect("没有几何对象B在测试用例中"),
                ) {
                    (
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                    )
                    | (
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                    ) => {
                        return Ok(Operation::ClipOp {
                            a: geometry.clone(),
                            b: case.b.clone().expect("没有几何对象B在测试用例中"),
                            invert: false,
                            expected: input.expected,
                        });
                    }
                    _ => {
                        validate_boolean_op(
                            &input.arg1,
                            &input.arg2,
                            geometry,
                            case.b.as_ref().expect("没有几何对象B在测试用例中"),
                        )?;
                    }
                };

                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    op: BoolOp::Intersection,
                    expected: input.expected,
                })
            }
            Self::DifferenceInput(input) => {
                // 在geo中裁剪线字符串就像是在JTS中进行线与多边形的交集
                match (
                    geometry,
                    case.b.as_ref().expect("没有几何对象B在测试用例中"),
                ) {
                    (
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                    )
                    | (
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                    ) => {
                        return Ok(Operation::ClipOp {
                            a: geometry.clone(),
                            b: case.b.clone().expect("没有几何对象B在测试用例中"),
                            invert: true,
                            expected: input.expected,
                        });
                    }
                    _ => {
                        validate_boolean_op(
                            &input.arg1,
                            &input.arg2,
                            geometry,
                            case.b.as_ref().expect("没有几何对象B在测试用例中"),
                        )?;
                    }
                };
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    op: BoolOp::Difference,
                    expected: input.expected,
                })
            }
            Self::SymDifferenceInput(input) => {
                validate_boolean_op(
                    &input.arg1,
                    &input.arg2,
                    geometry,
                    case.b.as_ref().expect("没有几何对象B在测试用例中"),
                )?;
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().expect("没有几何对象B在测试用例中"),
                    op: BoolOp::Xor,
                    expected: input.expected,
                })
            }
            Self::Unsupported => Err("不支持的OperationInput".into()),
            OperationInput::IsValidInput(input) => match input.arg1.as_str() {
                "A" => Ok(Operation::IsValidOp {
                    subject: geometry.clone(),
                    expected: input.expected,
                }),
                _ => todo!("处理 {}", input.arg1),
            },
        }
    }
}

fn validate_boolean_op(arg1: &str, arg2: &str, a: &Geometry<f64>, b: &Geometry<f64>) -> Result<()> {
    assert_eq!("A", arg1);
    assert_eq!("B", arg2);
    for arg in &[a, b] {
        if matches!(arg, Geometry::LineString(_)) {
            log::warn!("跳过不支持的`line_string.union`");
            return Err("不支持`line_string.union`".into());
        }
        if matches!(arg, Geometry::MultiPoint(_)) {
            log::warn!("跳过不支持的`line_string.union`");
            return Err("不支持`line_string.union`".into());
        }
    }
    Ok(())
}

pub fn deserialize_opt_geometry<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Geometry>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "wkt::deserialize_wkt")] Geometry);

    Option::<Wrapper>::deserialize(deserializer).map(|opt_wrapped| opt_wrapped.map(|w| w.0))
}

pub fn deserialize_from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: std::str::FromStr,
    D: Deserializer<'de>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    String::deserialize(deserializer)
        .and_then(|str| T::from_str(&str).map_err(serde::de::Error::custom))
}
