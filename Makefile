build:
	cargo build --release

test:
	./build.sh nhm9t3owr7ze7ij01uduaiop output/textbook

chevron:
	./build.sh vb1n097d5bcdes7qyeidww2q ../itell/apps/chevron/content/textbook

demo:
	./build.sh nhm9t3owr7ze7ij01uduaiop ../itell/apps/demo/content/textbook

middlesex:
	./build.sh k4szzxaraamln78crrfoauqd ../itell/apps/middlesex/content/textbook

introduction-to-computing:
	./build.sh bi049c8kjvr7ubolz69lnkfh ../itell/apps/introduction-to-computing/content/textbook
