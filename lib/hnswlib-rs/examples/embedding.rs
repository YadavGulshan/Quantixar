use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

fn main() {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
        .create_model()
        .unwrap();

    let sentences = vec!["This is an example sentence", "Each sentence is converted"];
    let embeddings = model.encode(&sentences).unwrap();
    println!("{:?}", embeddings);
}
