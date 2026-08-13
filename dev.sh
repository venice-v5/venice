export RT=/Users/aadish/Developer/venice/venice/target/armv7a-vex-v5/release/venice.bin
cargo v5 build -r &&
   du -sh "$RT" &&
   gzip -c "$RT" > "$RT.gz" &&
   mv "$RT.gz" "$RT" &&
   du -sh "$RT"
