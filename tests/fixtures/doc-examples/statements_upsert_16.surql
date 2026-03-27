IF (SELECT * FROM event_for:[time::now().format("%Y-%m-%d")]).is_empty() {
    CREATE event_for:[time::now().format("%Y-%m-%d")] SET number = 1;
} ELSE {
    UPDATE event_for:[time::now().format("%Y-%m-%d")] SET number += 1;
};
