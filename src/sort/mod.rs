pub mod distribution;
pub mod quick;

#[cfg(test)]
mod test {
    use test::Bencher;
    use util::random_array;

    pub fn generate_array_small() -> Vec<usize> {
        random_array(1024)
    }

    pub fn generate_array_medium() -> Vec<usize> {
        random_array(64 * 1024)
    }

    pub fn generate_array_large() -> Vec<usize> {
        random_array(1024 * 1024)
    }

    macro_rules! bench {
        ($name:ident, $array:expr) => {
            #[bench] fn $name (b: &mut Bencher) {
                b.iter(|| $array);
            }
        }
    }

    bench!(bench_generate_array_s, generate_array_small());
    bench!(bench_generate_array_m, generate_array_medium());
    bench!(bench_generate_array_l, generate_array_large());
    bench!(bench_stdlib_sort_s, generate_array_small().sort());
    bench!(bench_stdlib_sort_m, generate_array_medium().sort());
    bench!(bench_stdlib_sort_l, generate_array_large().sort());
    bench!(bench_stdlib_sort_unstable_s, generate_array_small().sort_unstable());
    bench!(bench_stdlib_sort_unstable_m, generate_array_medium().sort_unstable());
    bench!(bench_stdlib_sort_unstable_l, generate_array_large().sort_unstable());
}
