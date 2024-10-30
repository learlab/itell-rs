

.PHONY: chevron

build:
	cargo build --release

chevron:
	./build.sh 8 ../itell/apps/chevron/content
