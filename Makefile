build_server:
	@echo "Building the app..."
	cargo build --release
	@echo "Done!"

run_server:
	@echo "Running the server..."
	./target/release/DocuMindServer
	

build_client:
	@echo "Building the client..."
	cd client && yarn

run_client:
	@echo "Running the client..."
	cp client/src-tauri/.env target/release/.env
	cd target/release/ && ./DocuMindClient
