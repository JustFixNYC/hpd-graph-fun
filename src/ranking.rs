pub fn rank_tuples<T>(v: &mut Vec<(T, usize)>) {
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    v.reverse();
}
