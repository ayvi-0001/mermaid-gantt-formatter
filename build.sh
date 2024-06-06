# windows
cargo build --release --target x86_64-pc-windows-msvc
chmod +x target/x86_64-pc-windows-msvc/release/fmt-mmd-gantt.exe
mv target/x86_64-pc-windows-msvc/release/fmt-mmd-gantt.exe .

# linux
# cargo build --release --target x86_64-unknown-linux-gnu
# chmod +x target/x86_64-unknown-linux-gnu/release/fmt-mmd-gantt
# mv target/x86_64-unknown-linux-gnu/release/fmt-mmd-gantt .
