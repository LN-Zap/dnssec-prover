.PHONY: build-c-bindings release-c-bindings

build-c-bindings:
	cd uniffi \
	&& uniffi-bindgen-cs --config ./uniffi.toml ./src/interface.udl \
	&& cargo build --release

release-c-bindings: build-c-bindings
	mkdir -p dist/dnssec-prover
	cp uniffi/src/dnssec_prover.cs dist/dnssec-prover
	if [[ "$(shell uname)" == "Darwin" ]]; then \
		cp uniffi/target/release/libdnssec_prover_uniffi.dylib dist/dnssec-prover; \
	elif [[ "$(shell uname -o)" == "Msys" ]]; then \
		cp uniffi/target/release/libdnssec_prover_uniffi.dll dist/dnssec-prover; \
	else \
		cp uniffi/target/release/libdnssec_prover_uniffi.so dist/dnssec-prover; \
	fi
	cd dist && tar czvf dnssec-prover-$(VERSION)-$(shell uname -m).tar.gz dnssec-prover