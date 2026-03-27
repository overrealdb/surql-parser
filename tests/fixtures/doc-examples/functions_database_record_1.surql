/**[test]

[[test.results]]
value = "false"

*/

FOR $person IN ["Haakon_VII", "Ferdinand_I", "Manuel_II", "Wilhelm_II", "George_I", "Albert_I", "Alfonso_XIII", "George_V", "Frederick_VIII"] {
    LET $record_name = type::record("person", $person.lowercase());
    IF !$record_name.exists() {
        CREATE $record_name;
    }
}
