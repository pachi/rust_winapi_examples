release: linux win32
	$(info [INFO]: Compilando versión de producción)

build:
	$(info [INFO]: Compilando ejecutable (versión de depuración))
	cargo build

linux:
	$(info [INFO]: Versión de producción para linux)
	cargo build --release

win32:
	$(info [INFO]: Versión de producción para i686-pc-windows-gnu)
	cargo build --release --target=i686-pc-windows-gnu

clippy:
	$(info [INFO]: Comprobaciones con clippy)
	cargo +nightly clippy

bloat:
	$(info [INFO]: Calculando consumo de espacio en archivo ejecutable)
	cargo bloat --release -n 10
	cargo bloat --release --crates -n 10
