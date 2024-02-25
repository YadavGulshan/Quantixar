use hnsw_rs::dist::DistL1;
use hnsw_rs::prelude::*;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

fn main() {
    // Create the HNSW graph
    let nb_elem = 1000; // number of possible words in the dictionary
    let max_nb_connection = 15;
    let nb_layer = 16.min((nb_elem as f32).ln().trunc() as usize);
    let ef_c = 200;
    let hns = Hnsw::<f32, DistL1>::new(max_nb_connection, nb_elem, nb_layer, ef_c, DistL1 {});

    // Define five sets of poems
    let poem1 = "Roses are red, violets are blue, I love programming, and so do you.";
    let poem2 =
        "The sun shines bright, the moon shines bright, I love to code at night, and you do too.";
    let poem3 = "In the forest, the trees stand tall, With the wind whispering through the fall.";
    let poem4 = "Over the mountains, the rivers flow, Carrying life and hope, in every show.";
    let poem5 = "In the quiet of the night, stars twinkle bright, Guiding sailors through the dark, with their soft light.";

    // Create the sentence embeddings model
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
        .create_model()
        .unwrap();
    let poems = vec![poem1, poem2, poem3, poem4, poem5];
    // Convert poems to embeddings
    let embeddings = model.encode(&poems).unwrap();

    // Insert embeddings into the HNSW graph
    for (i, embedding) in embeddings.iter().enumerate() {
        hns.insert((embedding, i));
    }

    // Define a search query
    let query = &model.encode(&["I love"]).unwrap()[0];
    let query: &[f32] = &query;

    // Perform a search
    let ef_search: usize = 30;
    let res = hns.search(query, 10, ef_search);

    // Display the search results
    println!("Search results:");
    for r in res {
        println!("Score: {}\tPoem: {}", r.distance, poems[r.p_id.1 as usize],);
    }
}
