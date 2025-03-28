# DocuMind (RAG based desktop app)

Turn your documents into dynamic knowledge sources with Documind! Simply upload a document (currently supports only pdf format), ask questions, and get instant, accurate responses. Powered by advanced Retrieval-Augmented Generation (RAG) technology, DocuMind understands the content and provides clear and insightful answers. Whether it’s contracts, research papers, reports, or technical manuals — Documind helps you access information in seconds.

---

## Features
* Ask questions about your documents and get instant, accurate responses.
* Provides relevant, fact-based answers using RAG.
* Documents are processed securely on your local machine with no data leakage.

---

## TechStack
* Using Axum rust server at backend to expose REST Apis and connect with Ollama server for inference & Qdrant vector database for storage.
* Tauri app with React UI for frontend.

---

## 🛠️Installation

### Pre-requisites
Make sure you have following installed
* Rust
* Ollama
* Docker
* Node (v18.19) & yarn

### Setup the Qdrant vector database
```bash
docker volume create qdrant_data

docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -v qdrant_data:/qdrant/storage \
  qdrant/qdrant
```

### Pull the AI models on your machine via Ollama CLI

```bash
ollama pull nomic-embed-text:latest
ollama pull llama3.1:8b
```

### Clone the repository and build the application

```bash
# Clone the repo
git clone https://github.com/Harry-027/DocuMind
# Change the directory
cd DocuMind
# Rename the .env.example file to .env
mv .env.example .env
# Run the server
make app_server
# Run the client on another terminal
make app_client
```
---
## ⚙️ Configuration

In the `env.yaml`, you can configure the following -
- **Chunking Strategy:** You can configure the chunk size (`embedding_model_chunk_size`).
- **Embedding Model:** Customize the embedding model (`embedding_model_name`) for better document understanding.
- **LLM Model:** Customize the LLM model (`generate_model_name`) for better document understanding.

---

## 📜 License

Documind is licensed under the **MIT License**. See the LICENSE file for details.

---

## 🧑‍💻 Contributing

Feel free to open issues or submit pull requests.

---

## 📧 Contact

For support or inquiries, reach out at [harishmmp@gmail.com](mailto:harishmmp@gmail.com).

---
👉 **Documind**: Your documents, your AI-powered mind. 🌿
