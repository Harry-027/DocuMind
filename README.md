# DocuMind (RAG based desktop app)

Turn your documents into dynamic knowledge sources with Documind! Simply upload a document (currently supports only pdf format), ask questions, and get instant, accurate responses. Powered by advanced Retrieval-Augmented Generation (RAG) technology, DocuMind understands the content and provides clear and insightful answers. Whether it’s contracts, research papers, reports, or technical manuals — Documind helps you access information in seconds.
👉 Your documents, your AI-powered mind.


## Features
Ask questions about your documents and get instant, accurate responses.
Provides relevant, fact-based answers using RAG.
Documents are processed securely with no data leakage

🛠️ Installation

## Pre-requisites
Make sure you have following installed
* Rust
* Ollama
* Docker
* Node (v18.19) & yarn

### setup the Qdrant vector database
```bash
docker volume create qdrant_data

docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -v qdrant_data:/qdrant/storage \
  qdrant/qdrant
```

### Pull the AI models to local via Ollama CLI

```bash
ollama pull nomic-embed-text:latest
ollama pull llama3.1:8b
```

### Clone the repo and build the application

```bash
# Clone the repo
git clone https://github.com/Harry-027/DocuMind
cd DocuMind
# Run the server
make app_server
# Run the client on another terminal
make app_client
```
