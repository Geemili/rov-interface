
$OUT = "target/rov-interface"
$OUT_ZIP = "target/rov-interface.zip"

mkdir -f -o $OUT
cp README.txt $OUT

mkdir -f -p $OUT/dist
cp libfreetype-6.dll    $OUT/dist
cp LICENSE.freetype.txt $OUT/dist
cp LICENSE.zlib.txt     $OUT/dist
cp target/release/rov-interface.exe $OUT/dist
cp SDL2.dll             $OUT/dist
cp SDL2_ttf.dll         $OUT/dist
cp zlib1.dll            $OUT/dist
cp -Force -r assets            $OUT/dist

mkdir -f -p $OUT/driver
cp driver/src/commands.h $OUT/driver/
cp driver/src/main.h     $OUT/driver/
cp driver/src/src.ino    $OUT/driver/driver.ino

Compress-Archive -Force -Path $OUT/* -DestinationPath $OUT_ZIP
