pub mod distribution;
pub mod quick;

#[cfg(test)]
mod test {
    use test::Bencher;

    const LEN: usize = 1024*1024;

    pub fn generate_array() -> Vec<usize> {
        use util::random_array;
        random_array(LEN)
    }

    #[bench]
    fn bench_generate_array(b: &mut Bencher) {
        b.iter(|| generate_array())
    }
}
