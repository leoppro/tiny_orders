pub fn rand_i64(start: i64, end: i64) -> i64 {
    let i: u32 = rand::random();
    let mut i = i as i64;
    i = start + (i % (end - start + 1));
    return i;
}
