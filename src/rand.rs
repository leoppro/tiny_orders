pub fn rand_u32(start: u32, end: u32) -> u32 {
    let mut i: u32 = rand::random();
    i = start + (i % (end - start + 1));
    return i;
}
