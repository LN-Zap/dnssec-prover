.PHONY: build-uniffi-bindings release-uniffi-bindings

build-uniffi-bindings:
	cd uniffi \
	&& uniffi-bindgen-cs --config ./uniffi.toml ./src/interface.udl \
	&& cargo build --release

release-uniffi-bindings: build-uniffi-bindings
	mkdir -p dist/dnssec-prover
	cp uniffi/src/dnssec_prover.cs dist/dnssec-prover
	if [ "$(shell uname)" = "Darwin" ]; then \
		cp uniffi/target/release/libdnssec_prover_uniffi.dylib dist/dnssec-prover; \
	elif [ "$(shell uname -o)" = "Msys" ]; then \
		cp uniffi/target/release/libdnssec_prover_uniffi.dll dist/dnssec-prover; \
	else \
		cp uniffi/target/release/libdnssec_prover_uniffi.so dist/dnssec-prover; \
	fi
	cd dist && tar czvf dnssec-prover-$(VERSION)-$(shell uname -m).tar.gz dnssec-prover