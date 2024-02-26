// #[cfg(test)]
// mod test {
//     use faiss::{index_factory, Idx, Index, MetricType};

//     #[test]
//     fn index_factory_flat() {
//         let index = index_factory(64, "Flat", MetricType::L2).unwrap();
//         assert_eq!(index.is_trained(), true); // Flat index does not need training
//         assert_eq!(index.ntotal(), 0);
//     }

//     #[test]
//     fn flat_index_range_search() {
//         let mut index = index_factory(8, "Flat", MetricType::L2).unwrap();
//         let some_data = &[
//             7.5_f32, -7.5, 7.5, -7.5, 7.5, 7.5, 7.5, 7.5, -1., 1., 1., 1., 1., 1., 1., -1., 0., 0.,
//             0., 1., 1., 0., 0., -1., 100., 100., 100., 100., -100., 100., 100., 100., 120., 100.,
//             100., 105., -100., 100., 100., 105.,
//         ];
//         index.add(some_data).unwrap();
//         assert_eq!(index.ntotal(), 5);

//         let my_query = [0.; 8];
//         let result = index.range_search(&my_query, 8.125).unwrap();
//         let (distances, labels) = result.distance_and_labels();
//         assert!(labels == &[Idx::new(1), Idx::new(2)] || labels == &[Idx::new(2), Idx::new(1)]);
//         assert!(distances.iter().all(|x| *x > 0.));
//     }
// }
