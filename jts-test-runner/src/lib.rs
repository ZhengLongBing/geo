mod input;
use input::Operation;

mod runner;
pub use runner::TestRunner;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// ```
/// use jts_test_runner::assert_jts_tests_succeed;
/// assert_jts_tests_succeed("*Relate*.xml");
/// ```
pub fn assert_jts_tests_succeed(pattern: &str) {
    let mut runner = TestRunner::new().matching_filename_glob(pattern);
    runner.run().expect("测试用例运行失败");

    // 健全检查 - 确保至少运行了一些测试
    assert!(
        runner.failures().len() + runner.successes().len() > 0,
        "没有运行任何测试。"
    );

    if !runner.failures().is_empty() {
        let failure_text = runner
            .failures()
            .iter()
            .map(|failure| format!("{}", failure))
            .collect::<Vec<String>>()
            .join("\n");
        panic!(
            "JTS测试套件中有{}个失败 / {}个成功：\n{}",
            runner.failures().len(),
            runner.successes().len(),
            failure_text
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    // 目前有几个ConvexHull测试失败
    fn test_all_general() {
        init_logging();
        let mut runner = TestRunner::new();
        runner.run().expect("测试用例运行失败");

        if !runner.failures().is_empty() {
            let failure_text = runner
                .failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "JTS测试套件中有{}个失败 / {}个成功：\n{}",
                runner.failures().len(),
                runner.successes().len(),
                failure_text
            );
        }

        // 健全检查 - 确保运行了预期数量的测试。
        //
        // 随着更多测试的添加，我们需要增加这个数字，但它不应该减少。
        let expected_test_count: usize = 3775;
        let actual_test_count = runner.failures().len() + runner.successes().len();
        match actual_test_count.cmp(&expected_test_count) {
            Ordering::Less => {
                panic!(
                    "我们现在运行的测试用例比之前少了{}个。它们发生了什么？",
                    expected_test_count - actual_test_count
                );
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                panic!(
                    "很好，看起来我们正在运行新的测试。只需将`expected_test_count`增加到{}",
                    actual_test_count
                );
            }
        }
    }
}
