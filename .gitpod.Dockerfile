FROM gitpod/workspace-full

USER gitpod

RUN .cargo/bin/rustup toolchain install nightly --allow-downgrade -c rustfmt
