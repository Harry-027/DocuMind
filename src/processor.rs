use tokio::task;
use uuid::Uuid;

use crate::{
    utils::{chunk_text, extract_file_content, get_embeddings, send_request, ConfigVar},
    vector_db::VectorStore,
};

pub struct Processor {
    pub settings: ConfigVar,
}

impl Processor {
    pub fn new(settings: ConfigVar) -> Self {
        Self { settings }
    }

    pub async fn process_file(
        &self,
        file_name: &str,
        vec_store: &VectorStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = self.process_chunks(file_name)?;
        let embeddings = self.process_embeddings(chunks.to_owned()).await?;
        self.store_embeddings(file_name, embeddings.to_owned(), &vec_store)
            .await?;
        Ok(())
    }

    pub async fn process_prompt(
        &self,
        user_query: &str,
        coll_name: &str,
        vec_store: &VectorStore,
        processor: &Processor,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // split user query into chunks
        let chunk_size = self
            .settings
            .embedding_model_chunk_size
            .as_ref()
            .expect("required chunk size");
        let chunks = chunk_text(&user_query, *chunk_size);
        let embeddings = self.process_embeddings(chunks.to_owned()).await?;
        println!("embeddings:: {:?}", embeddings);
        let mut all_payloads = vec![];
        for embedding in embeddings {
            let payloads = vec_store.search_result(coll_name, embedding.1).await?;
            println!("Payloads:: {:?}", payloads);
            all_payloads.extend(payloads);
        }
        let context = all_payloads.join(",");
        let generate_model_url = processor
            .settings
            .generate_model_url
            .as_ref()
            .expect("generate model string is expected");
        let generate_model_name = processor
            .settings
            .generate_model_name
            .as_ref()
            .expect("model name is expected");
        let prompt = format!(
            "You are an expert providing factually accurate answers.
            Use only the information from the context to generate your answer.
            If the context doesn't contain relevant information say I don't know.
            Context: {context} Question: {user_query} Answer(only use the context for your answer)"
        );
        let res = send_request(
            generate_model_url.as_str(),
            generate_model_name.as_str(),
            prompt.as_str(),
        )
        .await?;
        let res_json: serde_json::Value =
            serde_json::from_str(res.as_str()).expect("expected parsing");
        let response: String = serde_json::from_value(res_json["response"].clone()).unwrap();
        Ok(response)
    }

    pub fn process_chunks(
        &self,
        file_name: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let chunk_size = self
            .settings
            .embedding_model_chunk_size
            .as_ref()
            .expect("required chunk size");
        let content = extract_file_content(file_name)?;
        let chunks = chunk_text(&content, *chunk_size);
        Ok(chunks)
    }

    pub async fn process_embeddings(
        &self,
        chunks: Vec<String>,
    ) -> Result<Vec<(String, Vec<f32>, String)>, Box<dyn std::error::Error>> {
        let mut tasks = vec![];
        for (_i, chunk) in chunks.into_iter().enumerate() {
            let settings_clone = self.settings.clone();
            tasks.push(task::spawn(async move {
                let chunk_clone = chunk.clone();
                let embedding = get_embeddings(&settings_clone, chunk_clone.as_str())
                    .await
                    .unwrap();
                (Uuid::new_v4().to_string(), embedding, chunk_clone)
            }))
        }

        let results = futures::future::join_all(tasks).await;
        let embeddings: Vec<(String, Vec<f32>, String)> =
            results.into_iter().map(|res| res.unwrap()).collect();
        Ok(embeddings)
    }

    pub async fn store_embeddings(
        &self,
        coll_name: &str,
        embeddings: Vec<(String, Vec<f32>, String)>,
        vec_store: &VectorStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        vec_store.store_embeddings(coll_name, embeddings).await?;
        Ok(())
    }
}
