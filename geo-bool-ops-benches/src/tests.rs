use anyhow::{bail, Context, Result};
use geo::prelude::*;
use geo_booleanop::boolean::BooleanOp as OtherBOp;
use geo_types::*;
use geojson::{Feature, GeoJson};
use glob::glob;
use log::{error, info};

use serde_derive::Serialize;
use std::{
    convert::TryInto,
    error::Error,
    fs::{read_to_string, File},
    io::BufWriter,
    panic::{catch_unwind, resume_unwind},
    path::Path,
};
use wkt::ToWkt;

#[cfg(test)]
#[path = "../benches/utils/bops.rs"]
pub mod bops_utils;

use bops_utils::*;

// 初始化日志
pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .try_init();
}

// 测试函数生成数据集
#[test]
fn generate_ds() -> Result<(), Box<dyn Error>> {
    init_log();

    // 获取项目路径
    let proj_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut cases = vec![];
    for fix in glob(&format!(
        "{proj_path}/fixtures/rust-geo-booleanop-fixtures/**/*.geojson"
    ))? {
        let fix = fix?;
        info!("运行 fixture {fix}...", fix = fix.display()); // 运行 fixture
        match try_run_fixture(&fix) {
            Ok(c) => {
                info!("\tfixture 成功，得到 {n} 个用例", n = c.len());
                cases.extend(c)
            }
            Err(e) => {
                error!("运行 fixture 出错: {fix:?}");
                error!("\t{e}");
            }
        }
    }
    #[derive(Serialize)]
    struct TestCase {
        p1: String,
        p2: String,
        op: String,
        expected: String,
        ours: String,
        theirs: String,
        comment: String,
    }

    // 获取 panic 信息
    fn panic_message(e: Box<dyn std::any::Any + Send + 'static>) -> String {
        e.downcast::<String>().map(|b| *b).unwrap_or_else(|e| {
            e.downcast::<&str>()
                .map(|b| b.to_string())
                .unwrap_or_else(|e| resume_unwind(e))
        })
    }

    // 运行指定的 fixture
    fn try_run_fixture(fix: &Path) -> Result<Vec<TestCase>> {
        let data = read_to_string(fix)?;
        let gjson: GeoJson = data.parse()?;
        let cases = if let GeoJson::FeatureCollection(fc) = gjson {
            if fc.features.len() <= 2 {
                return Ok(vec![]);
            }
            let p1 = feature_as_geom(&fc.features[0]).context("几何对象1不可读")?;
            let p2 = feature_as_geom(&fc.features[1]).context("几何对象2不可读")?;

            let prev_p1 = convert_mpoly(&p1);
            let prev_p2 = convert_mpoly(&p2);

            info!("p1: {wkt}", wkt = p1.to_wkt()); // 打印 p1 的 WKT
            info!("p2: {wkt}", wkt = p2.to_wkt()); // 打印 p2 的 WKT
            fc.features
                .into_iter()
                .skip(2)
                .map(|feat| -> Result<_> {
                    let p = feature_as_geom(&feat)?;
                    let props = feat.properties.unwrap();
                    let ty = props["operation"].as_str().context("操作不是字符串")?;
                    info!("操作: {ty} {wkt}", wkt = p.to_wkt(),);

                    let result = catch_unwind(|| {
                        let geoms = if ty == "intersection" {
                            p1.intersection(&p2)
                        } else if ty == "union" {
                            p1.union(&p2)
                        } else if ty == "xor" {
                            p1.xor(&p2)
                        } else if ty == "diff" {
                            p1.difference(&p2)
                        } else if ty == "diff_ba" {
                            p2.difference(&p1)
                        } else {
                            error!("未预期的操作: {ty}");
                            unreachable!()
                        };
                        info!("我们的结果: {wkt}", wkt = geoms.to_wkt());
                        geoms
                    });

                    let their_result = catch_unwind(|| {
                        let geoms = if ty == "intersection" {
                            prev_p1.intersection(&prev_p2)
                        } else if ty == "union" {
                            prev_p1.union(&prev_p2)
                        } else if ty == "xor" {
                            prev_p1.xor(&prev_p2)
                        } else if ty == "diff" {
                            prev_p1.difference(&prev_p2)
                        } else if ty == "diff_ba" {
                            prev_p2.difference(&prev_p1)
                        } else {
                            error!("未预期的操作: {ty}");
                            unreachable!()
                        };
                        let geoms = convert_back_mpoly(&geoms);
                        let wkt = geoms.wkt_string();
                        info!("他们的结果: {wkt}");
                        wkt
                    });
                    let theirs = their_result.unwrap_or_else(|_e| {
                        error!("他们的结果造成错误");
                        "pannik".to_string()
                    });

                    let (comment, our_geom) = match result {
                        Ok(our_geom) => {
                            let diff = catch_unwind(|| p.difference(&our_geom));
                            let comment = match diff {
                                Ok(diff) => {
                                    info!("差异: {wkt}", wkt = diff.to_wkt());
                                    if !diff.is_empty() {
                                        info!("输出不相同:");
                                        info!("\t我们的: {wkt}", wkt = our_geom.wkt_string());
                                        info!("操作: {ty} {wkt}", wkt = p.to_wkt(),);
                                        let area = diff.unsigned_area();
                                        let err = area / p.unsigned_area();
                                        info!("\t相对误差 = {err}");
                                        format!("相对误差: {err:.2}")
                                    } else {
                                        "相同".to_string()
                                    }
                                }
                                Err(e) => {
                                    let msg = panic_message(e);
                                    error!("差异计算出错: {msg}!");
                                    format!("差异错误: {msg}")
                                }
                            };
                            (comment, Some(our_geom))
                        }
                        Err(e) => {
                            let msg = panic_message(e);
                            error!("计算出错: {msg}!");
                            (format!("错误: {msg}"), None)
                        }
                    };

                    Ok(TestCase {
                        p1: p1.wkt_string(),
                        p2: p2.wkt_string(),
                        op: ty.to_string(),
                        ours: our_geom.map(|g| g.wkt_string()).unwrap_or_default(),
                        expected: p.wkt_string(),
                        comment,
                        theirs,
                    })
                })
                .collect::<Result<_>>()?
        } else {
            unreachable!()
        };
        Ok(cases)
    }

    // 创建 JSON 文件并写入测试用例数据
    let file = File::create("rust-geo-booleanop-fixtures.json")?;
    serde_json::to_writer(BufWriter::new(file), &cases)?;
    Ok(())
}

// 将 Feature 转换为 MultiPolygon
fn feature_as_geom(feat: &Feature) -> Result<MultiPolygon<f64>> {
    let p: Geometry<f64> = feat
        .geometry
        .clone()
        .context("特征中缺少几何")?
        .try_into()
        .context("无法将特征解析为几何")?;
    Ok(match p {
        Geometry::Polygon(p) => p.into(),
        Geometry::MultiPolygon(p) => p,
        _ => bail!("几何类型出乎意料"),
    })
}
