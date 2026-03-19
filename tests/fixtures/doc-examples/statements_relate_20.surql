/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: road:ohl5o501eh7ylfzcqwm3, length: 10.5f, name: 'Dalhurst Way' }]"
skip-record-id-key = true

*/

-- Returns true if $this->has_road path is not empty
DEFINE FIELD has_road_access ON TABLE house VALUE !!$this->has_road->road;
CREATE road SET name = "Dalhurst Way", length = 10.5;
