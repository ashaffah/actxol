FROM rust:1.81.0-alpine AS build
WORKDIR /usr/src/actxol
RUN apk add --no-cache clang lld musl-dev git bash
COPY ./static /usr/src/actxol/static
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/actxol /bin/server 
# ENTRYPOINT ["tail", "-f", "/dev/null"]

FROM build AS run
# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/
COPY --from=build /usr/src/actxol/static /bin/static
# ENTRYPOINT ["tail", "-f", "/dev/null"]
# What the container should run when it is started.
CMD ["/bin/server"]