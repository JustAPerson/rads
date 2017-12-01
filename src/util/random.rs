use rand;
use rand::Rng;
use rand::distributions::range::SampleRange;

pub fn random_range<T: SampleRange>(lo: T, hi: T) -> T {
    let range = T::construct_range(lo, hi);
    T::sample_range(&range, &mut rand::thread_rng())
}

// TODO Return Vec<T> where T: Clone or return Vec<&T>
pub fn random_samples<T: Clone>(array: &[T], k: usize) -> Vec<T> {
    let mut rng = rand::thread_rng();
    let range = usize::construct_range(0, array.len());

    let mut v = Vec::with_capacity(k);
    for _ in 0..k {
        let index = usize::sample_range(&range, &mut rng);
        v.push(array[index].clone())
    }

    v
}

pub fn random_array(l: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut out = Vec::with_capacity(l);
    for _ in 0..l {
        out.push(rng.gen())
    }
    out
}
