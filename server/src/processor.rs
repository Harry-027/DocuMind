use tokio::task;
use tracing::debug;
use uuid::Uuid;

use crate::{
    utils::{
        chunk_text, extract_file_content, get_content_embeddings, send_request, ConfigVar,
        ModelKind,
    },
    vector_db::VectorStore,
};

use anyhow::{anyhow, Context, Ok, Result};

pub struct Processor {
    pub settings: ConfigVar,
    pub vec_store: VectorStore,
}

impl Processor {
    pub fn new(settings: ConfigVar, vec_store: VectorStore) -> Self {
        Self {
            settings,
            vec_store,
        }
    }

    // process_file splits the text into chunks so to generate the embeddings
    // for proper context length and saves them to the db
    pub async fn process_file(&self, file_name: &str) -> Result<()> {
        let chunks = self.process_chunks(file_name)?;
        let embeddings = self.process_embeddings(chunks.to_owned()).await.unwrap();
        let coll_name = file_name.split_once(".pdf").unwrap().0;
        self.save_embeddings(coll_name, embeddings.to_owned())
            .await?;
        Ok(())
    }

    // process_prompt gets the similar cosine embeddings for the user prompt
    // and sets the context for LLM to get the result generated as per the context
    pub async fn process_prompt(&self, user_query: &str, doc_name: &str) -> Result<String> {
        // split user query into chunks
        let chunk_size = self
            .settings
            .embedding_model_chunk_size
            .as_ref()
            .expect("required chunk size");
        let chunks = chunk_text(&user_query, *chunk_size);
        let embeddings = self
            .process_embeddings(chunks.to_owned())
            .await
            .context("unable to process the embeddings")?;

        // get all the payloads similar to prompt embedding
        let mut all_payloads = vec![];
        if let Some(split_name) = doc_name.split_once(".pdf") {
            let coll_name = split_name.0;
            for embedding in embeddings {
                let payloads = self
                    .vec_store
                    .search_result(coll_name, embedding.1)
                    .await
                    .with_context(|| format!("unable to fetch the result for {}", coll_name))?;
                debug!("Payloads:: {:?}", payloads);
                all_payloads.extend(payloads);
            }
        } else {
            debug!("error handling fileName ...");
            return Err(anyhow!("bad request - doc type is incorrect..."));
        }

        // set the LLM context
        let context = all_payloads.join(",");

        let (generate_model_url, generate_model_name) = self
            .settings
            .get_model_details(ModelKind::Generate)
            .context("unable to fetch the model details")?;

        // final prompt to the LLM
        let prompt = format!(
            "You are an expert providing factually accurate answers.
            Use only the information from the context to generate your answer.
            If the context doesn't contain relevant information say I don't know as context doesn't have much info.
            Context: {context} Question: {user_query} Answer(only use the context for your answer)"
        );
        let res = send_request(
            generate_model_url.as_str(),
            generate_model_name.as_str(),
            prompt.as_str(),
        )
        .await
        .context("send request to LLM operation failed")?;
        let res_json: serde_json::Value = serde_json::from_str(res.as_str())
            .context("parsing response into value type failed")?;
        let response: String = serde_json::from_value(res_json["response"].clone())
            .context("parsing string from value type failed")?;
        Ok(response)
    }

    // process_chunks splits the large text into chunks
    pub fn process_chunks(&self, file_name: &str) -> Result<Vec<String>> {
        let chunk_size = self
            .settings
            .embedding_model_chunk_size
            .as_ref()
            .expect("required chunk size");
        let content =
            extract_file_content(file_name).context("failed to extract the file content")?;
        let chunks = chunk_text(&content, *chunk_size);
        Ok(chunks)
    }

    // process_embedding generates the embeddings for different chunk texts parallely
    pub async fn process_embeddings(
        &self,
        chunks: Vec<String>,
    ) -> Result<Vec<(String, Vec<f32>, String)>> {
        let mut tasks = vec![];
        for (_i, chunk) in chunks.into_iter().enumerate() {
            let settings = self.settings.clone();
            tasks.push(task::spawn(async move {
                let chunk_clone = chunk.clone();
                let embedding = get_content_embeddings(settings, chunk_clone.as_str())
                    .await
                    .unwrap_or_else(|e| {
                        debug!("Error: {}", e);
                        vec![]
                    });
                (Uuid::new_v4().to_string(), embedding.clone(), chunk_clone)
            }))
        }

        let results = futures::future::join_all(tasks).await;
        let embeddings: Vec<(String, Vec<f32>, String)> =
            results.into_iter().map(|res| res.unwrap()).collect();
        Ok(embeddings)
    }

    // save_embeddings saves the embeddings to the vector DB
    pub async fn save_embeddings(
        &self,
        coll_name: &str,
        embeddings: Vec<(String, Vec<f32>, String)>,
    ) -> Result<()> {
        self.vec_store
            .store_embeddings(coll_name, embeddings)
            .await?;
        Ok(())
    }
}
