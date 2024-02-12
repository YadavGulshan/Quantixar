use faiss::MetricType;

// represents the common functionality of all index types
pub trait Index
{
    fn new(d: usize, metric: MetricType) -> Self
    where
        Self: Sized;
    fn init(&mut self, index_param: &str) -> i32;
    fn train(&mut self, n: i64, x: &[f32]);
    fn add(&mut self, n: i64, x: &[f32]);
    fn search(&self, n: i64, x: &[f32], k: i64) -> (Vec<f32>, Vec<i64>);
    fn dump(&self, dir: &str) -> i32;
    fn load(&mut self, dir: &str) -> i32;
}
