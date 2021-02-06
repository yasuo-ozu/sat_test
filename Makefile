%.svg:	8-queens.sat src/bin/%.rs
	cargo run --bin "$(basename $@)" -- $< $@

%.pdf:	%.svg
	inkscape -f $< -A $@

%.jpg:	%.svg
	convert -units PixelsPerInch -density 72x72 $< $@

clean:
	rm -rf *.jpg *.svg *.pdf
