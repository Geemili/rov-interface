
pio-dir = "./driver"
out = "target/rov-interface"
target = "x86_64-pc-windows-gnu"
binaries = '
    binaries/libfreetype-6.dll
    binaries/SDL2.dll
    binaries/SDL2_ttf.dll
    binaries/zlib1.dll
    binaries/LICENSE.freetype.txt
    binaries/LICENSE.zlib.txt
    target/{{target}}/rov-interface.exe
    assets/
'

build-app:
    cargo build

build-driver:
    pio run --project-dir {{pio-dir}} --environment promini

build-release:
    LIBRARY_PATH=lib/ cargo build --release --target={{target}}
    pio run --project-dir {{pio-dir}}

run: build-app
    env RUST_BACKTRACE=1 cargo run

upload: build-driver
    pio run --project-dir {{pio-dir}} --target upload --environment promini

publish: build-release
    mkdir -pf {{out}}/dist
    cp binaries {{out}}/dist/
    mkdir -pf {{out}}/driver
    cp driver/src/commands.h driver/src/main.h {{out}}/driver/
    cp driver/src/src.ino {{out}}/driver/driver.ino

