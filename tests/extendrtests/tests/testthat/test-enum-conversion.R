test_that("non-factor integer vector", {
  expect_error(
    tst_enum_wrapper(NULL, c(1L))
  )
  expect_error(
    tst_enum_wrapper(NULL, 1L)
  )
})
test_that("non-scalar factor vector", {
  expect_error(
    tst_enum_wrapper(NULL, factor(c("A", "B"), levels = c("A", "B", "C")))
  )
})
test_that("invalid levels", {
  expect_error(
    tst_enum_wrapper(NULL, factor(c("B"), levels = c("a", "b", "c")))
  )
})
test_that("scalar conversion test", {
  expect_equal(
    my_enum("B"),
    factor(c("B"), levels = c("A", "B", "C"))
  )
})
