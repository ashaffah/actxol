run	:
	@cargo run
watch	:
	@cargo watch -x run
compose	:
	@docker compose down --volumes && docker compose up --build
exec	:
	@docker exec -it actxol-app-1 bash