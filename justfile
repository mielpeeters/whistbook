watch-command: tailwind
  cargo run

watch-command-release: tailwind
  cargo run --release

bundle:
  pnpm webpack

tailwind:
  pnpm tailwindcss -i styles/tailwind.css -o public/css/main.css

watch:
  cargo watch -w templates -w src -w styles -- just watch-command

watch-release:
  cargo watch -w templates -w src -w styles -- just watch-command-release
