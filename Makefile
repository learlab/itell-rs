

.PHONY: chevron

build:
	cargo build --release

chevron:
	./build.sh 8 ../itell/apps/chevron/content

rmp:
	./build.sh 9 ../itell/apps/research-methods-in-psychology/content
