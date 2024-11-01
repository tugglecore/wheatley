set -euxo pipefail
echo "Run tests."

for codec in br gzip snap zstd; do
	cargo test -F "$codec" "$codec"_compression -- --exact
done

cargo test
