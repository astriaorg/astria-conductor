FROM rust:1.65

COPY . .

# Setup buf
RUN BIN="/usr/local/bin" && \
VERSION="1.15.1" && \
curl -sSL \
"https://github.com/bufbuild/buf/releases/download/v${VERSION}/buf-$(uname -s)-$(uname -m)" \
-o "${BIN}/buf" && \
chmod +x "${BIN}/buf"

RUN cargo build --release

CMD ["./target/build/release/astria-conductor"]