use std::collections::HashMap;

use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointId, PointStruct, SearchPoints, UpsertPointsBuilder,
        Value, VectorParamsBuilder, Vectors,
    },
    Qdrant,
};

pub struct VectorStore {
    client: Qdrant,
}

// initialize the db client
fn db_init(url: &str) -> Qdrant {
    let qdrant_client = match Qdrant::from_url(url).build() {
        Ok(client) => client,
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

    // create_collection to be private by default. To be created based on doc name
    async fn create_collection(
        &self,
        collection_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.client.collection_exists(collection_name).await {
            Ok(exists) => {
                if exists {
                    Ok(())
                } else {
                    let coll_created = self
                        .client
                        .create_collection(
                            CreateCollectionBuilder::new(collection_name)
                                .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine)),
                        )
                        .await?;
                    if coll_created.result {
                        Ok(())
                    } else {
                        Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "internal error",
                        )))
                    }
                }
            }
            Err(err) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.to_string(),
            ))),
        }
    }

    // saves the vector embeddings to database
    pub async fn store_embeddings(
        &self,
        collection_name: &str,
        embeddings: Vec<(String, Vec<f32>, String)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            .unwrap();
        println!("embeddings saved successfully!");
        Ok(())
    }

    // search the similar points
    pub async fn search_result(
        &self,
        collection_name: &str,
        query: Vec<f32>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: collection_name.to_string(),
                vector: query,
                limit: 6,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

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
}
