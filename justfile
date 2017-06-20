
pwd = `echo $PWD`
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

build_folder = pwd + "/build"
sdl_folder = build_folder + "/sdl"

sdl2_url = "https://www.libsdl.org/release/SDL2-devel-2.0.5-mingw.tar.gz"
sdl2_ttf_url = "https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-devel-2.0.14-mingw.tar.gz"

download-sdl:
    mkdir -p {{sdl_folder}}
    curl -L {{sdl2_url}}     | tar xzf - -C {{sdl_folder}}
    curl -L {{sdl2_ttf_url}} | tar xzf - -C {{sdl_folder}}

windows-cross:
    # If you get an error, make sure you have run: "just download-sdl"
    cargo build --release --target "x86_64-pc-windows-gnu"

windows_dist_folder = build_folder + "/windows_dist"
sdl2_path = sdl_folder + "/SDL2-2.0.5"
sdl2_ttf_path = sdl_folder + "/SDL2_ttf-2.0.14"
assets_path = "./assets"
executable = "./target/x86_64-pc-windows-gnu/release/rov-interface.exe"
windows_dist_zip_prefix = build_folder + "/rov-interface"
windows_dist_zip_suffix = "windows.zip" 

dist-windows VERSION: windows-cross
    mkdir -p {{windows_dist_folder}}
    cp {{sdl2_path}}/x86_64-w64-mingw32/bin/* {{windows_dist_folder}}
    cp {{sdl2_ttf_path}}/x86_64-w64-mingw32/bin/* {{windows_dist_folder}}
    cp -r {{assets_path}} {{windows_dist_folder}}
    cp {{executable}} {{windows_dist_folder}}
    # TODO: Copy driver code
    # driver/
    ## driver/BNO055/      <- I could vendor the library instead
    #                           Yeah. Let's do that.
    ## driver/driver/      <- Driver
    ## README.txt          <- Installation instructions, pins
    # Zip up directory
    cd {{windows_dist_folder}} && zip -FSr "{{windows_dist_zip_prefix}}_{{VERSION}}_{{windows_dist_zip_suffix}}" *

new-release VERSION:
    git checkout -b release-{{VERSION}} develop
    cargo bump --no-git-tag-version {{VERSION}}
    git add Cargo.toml Cargo.lock
    git commit -m "Update version number"
    just dist-windows {{VERSION}}

commit-release VERSION MASTER_MESSAGE:
    git checkout master
    git merge --no-ff release-{{VERSION}}
    git tag -a {{VERSION}} -m "MASTER_MESSAGE" master
    git push --tags
    git checkout develop
    git merge --no-ff release-{{VERSION}}
    git push
    git branch -d release-{{VERSION}}

kak:
    # Start kakoune daemon
    kak -d -s rov-interface

