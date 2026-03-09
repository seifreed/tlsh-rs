pub fn find_quartiles(buckets: &[u32]) -> (u32, u32, u32) {
    let bucket_count = buckets.len();
    let mut bucket_copy = buckets.to_vec();
    let mut short_cut_left = vec![0usize; bucket_count];
    let mut short_cut_right = vec![0usize; bucket_count];
    let mut spl = 0usize;
    let mut spr = 0usize;

    let p1 = (bucket_count / 4) - 1;
    let p2 = (bucket_count / 2) - 1;
    let p3 = bucket_count - (bucket_count / 4) - 1;
    let end = bucket_count - 1;

    let q2 = {
        let mut l = 0usize;
        let mut r = end;
        loop {
            let ret = partition(&mut bucket_copy, l, r);
            if ret > p2 {
                r = ret - 1;
                short_cut_right[spr] = ret;
                spr += 1;
            } else if ret < p2 {
                l = ret + 1;
                short_cut_left[spl] = ret;
                spl += 1;
            } else {
                break bucket_copy[p2];
            }
        }
    };

    short_cut_left[spl] = p2 - 1;
    short_cut_right[spr] = p2 + 1;

    let q1 = {
        let mut found = bucket_copy[p1];
        let mut l = 0usize;
        for &r in &short_cut_left[..=spl] {
            if r > p1 {
                let mut r = r;
                loop {
                    let ret = partition(&mut bucket_copy, l, r);
                    if ret > p1 {
                        r = ret - 1;
                    } else if ret < p1 {
                        l = ret + 1;
                    } else {
                        found = bucket_copy[p1];
                        break;
                    }
                }
                break;
            }
            if r < p1 {
                l = r;
            } else {
                found = bucket_copy[p1];
                break;
            }
        }
        found
    };

    let q3 = {
        let mut found = bucket_copy[p3];
        let mut r = end;
        for &l in &short_cut_right[..=spr] {
            if l < p3 {
                let mut l = l;
                loop {
                    let ret = partition(&mut bucket_copy, l, r);
                    if ret > p3 {
                        r = ret - 1;
                    } else if ret < p3 {
                        l = ret + 1;
                    } else {
                        found = bucket_copy[p3];
                        break;
                    }
                }
                break;
            }
            if l > p3 {
                r = l;
            } else {
                found = bucket_copy[p3];
                break;
            }
        }
        found
    };

    (q1, q2, q3)
}

fn partition(buf: &mut [u32], left: usize, right: usize) -> usize {
    if left == right {
        return left;
    }

    if left + 1 == right {
        if buf[left] > buf[right] {
            buf.swap(left, right);
        }
        return left;
    }

    let mut ret = left;
    let pivot = (left + right) >> 1;
    let val = buf[pivot];
    buf.swap(pivot, right);

    for idx in left..right {
        if buf[idx] < val {
            buf.swap(ret, idx);
            ret += 1;
        }
    }

    buf[right] = buf[ret];
    buf[ret] = val;
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_covers_small_ranges() {
        let mut one = [7u32];
        assert_eq!(partition(&mut one, 0, 0), 0);

        let mut two = [9u32, 1];
        assert_eq!(partition(&mut two, 0, 1), 0);
        assert_eq!(two, [1, 9]);
    }

    #[test]
    fn quartiles_match_sorted_reference() {
        let values = [10, 0, 5, 1, 7, 2, 9, 4];
        let (q1, q2, q3) = find_quartiles(&values);
        assert_eq!((q1, q2, q3), (1, 4, 7));
    }

    #[test]
    fn quartiles_cover_q1_shortcut_and_partition_no_swap_branch() {
        let values = [50, 40, 30, 20, 10, 0, 60, 70];
        let (q1, q2, q3) = find_quartiles(&values);
        let mut sorted = values;
        sorted.sort_unstable();
        assert_eq!((q1, q2, q3), (sorted[1], sorted[3], sorted[5]));

        let mut two = [1u32, 9];
        assert_eq!(partition(&mut two, 0, 1), 0);
        assert_eq!(two, [1, 9]);
    }

    #[test]
    fn quartiles_cover_q1_ret_greater_than_target_branch() {
        let values = [0, 2, 1, 3, 4, 5, 6, 7];
        let (q1, q2, q3) = find_quartiles(&values);
        let mut sorted = values;
        sorted.sort_unstable();
        assert_eq!((q1, q2, q3), (sorted[1], sorted[3], sorted[5]));
    }

    #[test]
    fn quartiles_cover_q3_shortcut_right_branch() {
        let values = [0, 1, 2, 6, 3, 4, 5, 7];
        let (q1, q2, q3) = find_quartiles(&values);
        let mut sorted = values;
        sorted.sort_unstable();
        assert_eq!((q1, q2, q3), (sorted[1], sorted[3], sorted[5]));
    }
}
