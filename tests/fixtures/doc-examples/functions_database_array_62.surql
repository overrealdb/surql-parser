/**[test]
[[test.results]]
value = "[{ is_even: false, value: 1 }, { is_even: true, value: 2 }, { is_even: false, value: 3 }]"

*/

["1", "2", "3"].map(|$val| {
  LET $num = <number>$val;
  LET $is_even = IF $num % 2 = 0 { true } ELSE { false };
  {
    value: $num,
    is_even: $is_even
  }
});
