//! Order maintenance problems

pub mod linked_list;
pub mod ofm;

#[cfg(test)]
mod test {
    use test::Bencher;
    pub const N: usize = 64*1024;

    #[bench]
    fn bench_vec_push_back(b: &mut Bencher) {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..N { v.push(i) }
        });
    }

    #[bench]
    fn bench_vec_push_front(b: &mut Bencher) {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..N { v.insert(0, i) }
        });
    }

    #[bench]
    fn bench_vec_iter(b: &mut Bencher) {
        let mut v = Vec::new();
        for i in 0..N { v.push(i) }
        b.iter(|| v.iter().sum::<usize>());
    }
}
