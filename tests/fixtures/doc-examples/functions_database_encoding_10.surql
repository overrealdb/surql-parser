/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ cbor: 'encoding', is: 'pretty neat' }"

*/
LET $some_bytes = encoding::base64::decode("omRjYm9yaGVuY29kaW5nYmlza3ByZXR0eSBuZWF0");
encoding::cbor::decode($some_bytes);
