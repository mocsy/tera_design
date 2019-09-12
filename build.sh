cargo install cross

zip -r tera_design_0-0-0.zip examples README.md LICENSE config.ron
cross build --release --target x86_64-pc-windows-gnu
zip -r tera_design_0-0-0.zip target/x86_64-pc-windows-gnu/release/tera_design.exe

cargo build --release --target x86_64-unknown-linux-gnu
zip -r tera_design_0-0-0.zip target/x86_64-unknown-linux-gnu/release/tera_design
