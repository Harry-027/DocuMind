use std::collections::HashMap;

use anyhow::{anyhow, Context, Ok, Result};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointId, PointStruct, SearchPoints, UpsertPointsBuilder,
        Value, VectorParamsBuilder, Vectors,
    },
    Qdrant,
};
use tracing::info;

pub struct VectorStore {
    client: Qdrant,
}

// initialize the db client
fn db_init(url: &str) -> Qdrant {
    let qdrant_client = match Qdrant::from_url(url).build() {
        std::result::Result::Ok(client) => client,
        Err(err) => panic!("unable to connect to DB! {}", err.to_string()),
    };
    return qdrant_client;
}

// impl the vector store
impl VectorStore {
    pub fn new(db_url: &str) -> Self {
        let db_client = db_init(db_url);
        Self { client: db_client }
    }

    // create_collection to be a private method. Collection to be created based on doc name
    async fn create_collection(&self, collection_name: &str) -> Result<()> {
        let collection_exists = self
            .client
            .collection_exists(collection_name)
            .await
            .context("collection_exists operation failed!")?;
        if !collection_exists {
            let new_collection = self
                .client
                .create_collection(
                    CreateCollectionBuilder::new(collection_name)
                        .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine)),
                )
                .await
                .context("create new collection failed")?;
            if new_collection.result {
                return Ok(());
            } else {
                return Err(anyhow!("unable to create the new collection"));
            }
        }
        Err(anyhow!("collection already exists"))
    }

    // saves the vector embeddings to database
    pub async fn store_embeddings(
        &self,
        collection_name: &str,
        embeddings: Vec<(String, Vec<f32>, String)>,
    ) -> Result<()> {
        self.create_collection(collection_name).await?;
        let points: Vec<PointStruct> = embeddings
            .into_iter()
            .map(|(id, vec, content)| PointStruct {
                id: Some(PointId::from(id)),
                vectors: Some(Vectors::from(vec)),
                payload: HashMap::from([("text".to_string(), Value::from(content))]),
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points).wait(true))
            .await
            .context("upsert_points operation failed")?;
        info!("embeddings saved successfully!");
        Ok(())
    }

    // search for the similar points along with payload
    // payload to be sent to LLM as context.
    pub async fn search_result(
        &self,
        collection_name: &str,
        query: Vec<f32>,
    ) -> Result<Vec<String>> {
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: collection_name.to_string(),
                vector: query,
                limit: 6,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .context("unable to fetch the results")?;

        let payloads: Vec<String> = search_result
            .result
            .iter()
            .filter_map(|p| {
                p.payload
                    .get("text")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .collect();
        Ok(payloads)
    }

    //list out the collection names
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let collections = self
            .client
            .list_collections()
            .await
            .context("list_collections operation failed")?;
        let collection_names: Vec<String> = collections
            .collections
            .into_iter()
            .map(|coll| coll.name)
            .collect();
        Ok(collection_names)
    }
}
