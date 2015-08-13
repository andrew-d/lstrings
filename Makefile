SHELL := /bin/bash

all: src/english-bigram-map.bin

# NOTE: this data from: http://corpora2.informatik.uni-leipzig.de/download.html
data/eng_news_2013_100K-sentences.txt: data/eng_news_2013_100K-sentences.txt.xz
	xz -dc $< > $@

src/english-bigram-map.bin: data/eng_news_2013_100K-sentences.txt examples/make_bigram_map.rs
	cargo run \
		--release \
		--example make_bigram_map \
		-- \
		$< \
		$@
