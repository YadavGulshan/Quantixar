use std::io::Error;

use hdf5::{File, Result};

use crate::actix::routes::dataset_api::HDF5FileConfig;

pub fn read_hdf5(file_path: &str, config: HDF5FileConfig) -> Result<()> {
  let file = File::open(file_path)?;
  let dataset = file.dataset(config.target_data_path)?;
  println!("{:?}", dataset.read_raw::<f32>()?);
  // let attr = dataset.attr("/test")?;
  // println!("{:?}", attr.read_1d::<f32>()?.as_slice());
  Ok(())
}
