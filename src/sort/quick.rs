//! Hoare's Quick Sort

pub fn quick_sort<T: Copy+Ord>(array: &mut [T]) {
    let len = array.len();
    quick_sort_range(array, 0, len - 1)
}

pub fn quick_sort_range<T: Copy+Ord>(array: &mut [T], lo: usize, hi: usize) {
    if lo < hi {
        let p = partition(array, lo   , hi);
        quick_sort_range (array, lo   , p );
        quick_sort_range (array, p + 1, hi);
    }
}

fn partition<T: Copy+Ord>(array: &mut [T], lo: usize, hi: usize) -> usize {
    let pivot = array[lo];
    let mut i = lo.wrapping_sub(1);
    let mut j = hi.wrapping_add(1);
    loop {
        i = i.wrapping_add(1);
        j = j.wrapping_sub(1);

        while array[i] < pivot { i = i.wrapping_add(1) }
        while array[j] > pivot { j = j.wrapping_sub(1) }

        if i >= j { return j }
        array.swap(i, j);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::Bencher;
    use sort::test::*;

    #[test]
    fn correct() {
        use util::random_array;
        let mut l = random_array(1024);

        let mut qsorted = l.clone();
        quick_sort(&mut *qsorted);

        l.sort_unstable();
        assert!(l == qsorted);
    }

    macro_rules! bench {
        ($name:ident, $array:expr) => {
            #[bench] fn $name (b: &mut Bencher) {
                b.iter(|| quick_sort(&mut *$array));
            }
        }
    }

    bench!(bench_s, generate_array_small());
    bench!(bench_m, generate_array_medium());
    bench!(bench_l, generate_array_large());
}
