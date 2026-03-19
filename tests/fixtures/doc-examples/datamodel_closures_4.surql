LET $my_func = |$num| <number>$num * 1000;

"Two"
    .replace("Two", "2")
    .chain($my_func);
