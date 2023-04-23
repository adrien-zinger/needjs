all:
	@echo "Redirection to \`make init\`, you may run \`make build\` then"

init: submodules

./runtime/target/debug/runtime:
	$(MAKE) build

.SILENT:
build:
	@echo "Build runtime"
	cd runtime; cargo build

.SILENT:
install: ./runtime/target/debug/runtime
	@echo "Put binaries into bin/ folder"
	if [ ! -d "./bin" ]; then mkdir bin; fi
	cp runtime/target/debug/runtime bin/runtime

submodules:
	git submodule update --init --remote