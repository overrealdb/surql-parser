-- true: has `type` and `coordinates` field with valid data
{ type: "Point", coordinates: [-0.118092, 51.509865] }.is_geometry();

-- false: lacks 'type' field
{ coordinates: [-0.118092, 51.509865] }.is_geometry();

-- false: has an extra field
{ type: "Point", coordinates: [-0.118092, 51.509865], unnecessary: "data" }.is_geometry();
