use etcd::Error;
use faiss::Index;

pub trait RetrievalModel
{
    fn init(&mut self, model_parameters: &str, indexing_size: i32) -> i32;
    fn indexing(&mut self) -> i32;

    fn add(&mut self, n: i64, vec: &[u8]) -> bool;
    fn update(&mut self, ids: &[i64], vecs: &[&[u8]]) -> i32;
    fn delete(&mut self, ids: &[i64]) -> i32;
    fn search(&self, n: i64, x: &[u8], k: i32) -> (Vec<f32>, Vec<i64>);
    fn search_preassigned(
        &self,
        n: i64,
        x: &[f32],
        k: i32,
        keys: &[i64],
        coarse_dis: &[f32],
        store_pairs: bool,
    ) -> (Vec<f32>, Vec<i64>);
    fn get_total_mem_bytes(&self) -> i64;
    fn dump(&self, dir: &str) -> i32;
    fn load(&mut self, dir: &str) -> i32;
    fn train(&mut self, x: &[f32]) -> Result<(), Error>;
}
