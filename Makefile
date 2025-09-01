build:
	cargo build --release

test:
	./build.sh nhm9t3owr7ze7ij01uduaiop output/textbook

testing:
	./build.sh r45rrxumaunejh5kwh6o7gr3 ../itell/apps/testing/content/textbook

nlp:
	./build.sh t8gcaq5m82inj19xsu8qe3ml ../itell/apps/testing/content/textbook

chevron:
	./build.sh vb1n097d5bcdes7qyeidww2q ../itell/apps/chevron/content/textbook

demo:
	./build.sh nhm9t3owr7ze7ij01uduaiop ../itell/apps/demo/content/textbook

middlesex:
	./build.sh k4szzxaraamln78crrfoauqd ../itell/apps/middlesex/content/textbook

rmp:
	./build.sh nhm9t3owr7ze7ij01uduaiop ../itell/apps/rmp/content/textbook

introduction-to-computing:
	./build.sh bi049c8kjvr7ubolz69lnkfh ../itell/apps/introduction-to-computing/content/textbook

civic:
	./build.sh r45rrxumaunejh5kwh6o7gr3 output
