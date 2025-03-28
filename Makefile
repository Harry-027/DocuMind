build_server:
	@echo "Building the app..."
	cargo build --release
	@echo "Done!"

run_server:
	@echo "Running the server..."
	./target/release/DocuMindServer
	

tauri_client:
	@echo "Building the client..."
	cd client && yarn && yarn run dev
	

app_client:
	@echo "Running the client..."
	cp client/src-tauri/.env.example target/release/.env
	cd target/release/ && ./DocuMindClient

app_server: build_server run_server 

