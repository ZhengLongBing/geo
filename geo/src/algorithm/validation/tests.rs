// 运行JTS测试用例以验证所有带“Valid”字样的测试都成功
#[test]
fn jts_validation_tests() {
    jts_test_runner::assert_jts_tests_succeed("*Valid*");
}
