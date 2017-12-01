use util::random_samples;
const M: usize = 4096;
const B: usize = 64;
const sMB: usize = 8; // sqrt(M/B)

fn external_distribution_sort<T: Clone+Ord>(array: &[T]) -> Vec<T> {
    if array.len() <= M {
        let mut array = array.to_vec();
        array.sort_unstable();
        return array
    }

    let mut pivots = random_samples(array, sMB);
    pivots.sort_unstable();

    let mut partitions = (0..pivots.len() + 1).map(|_| Vec::new()).collect::<Vec<_>>();
    for ele in array {
        let pnum = pivots.iter().filter(|&x| x < ele).count();
        partitions[pnum].push(ele.clone());
    }

    let mut output = Vec::with_capacity(array.len());
    for p in partitions {
        output.extend(external_distribution_sort(&*p))
    }
    output
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

        let dsorted = external_distribution_sort(&*l);
        l.sort_unstable();

        assert!(l == dsorted);
    }

    macro_rules! bench {
        ($name:ident, $array:expr) => {
            #[bench] fn $name (b: &mut Bencher) {
                b.iter(|| external_distribution_sort(&*$array));
            }
        }
    }

    bench!(bench_s, generate_array_small());
    bench!(bench_m, generate_array_medium());
    bench!(bench_l, generate_array_large());
}
