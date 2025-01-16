set -euxo pipefail
echo "Run tests."

for codec in br gzip snap zstd; do
	cargo nextest run -F "$codec" -- "$codec"_compression --exact
done

cargo nextest run --workspace
