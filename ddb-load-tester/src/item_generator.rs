use rand::{
    Rng, SeedableRng,
    distr::{Alphabetic, SampleString},
};

#[derive(Clone)]
pub struct ItemGenerator {
    items: Vec<String>,
    random: rand::rngs::SmallRng,
}
impl ItemGenerator {
    pub fn new(seed: u64, item_count: u64, length: usize) -> Self {
        let mut random = rand::rngs::SmallRng::seed_from_u64(seed);
        let mut items = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            items.push(Alphabetic.sample_string(&mut random, length));
        }
        Self { items, random }
    }

    pub fn next(&mut self) -> String {
        let index = self.random.random_range(0..self.items.len());
        self.items[index].clone()
    }
}
