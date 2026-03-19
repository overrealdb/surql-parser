array::len(
    array::clump(
        array::distinct(
            array::flatten([[1,2,3],[1,4,6],[4,2,4]])
        )
    , 2)
);
