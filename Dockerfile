FROM rust:1.81.0-alpine AS build
WORKDIR /actxol
RUN apk add --no-cache clang lld musl-dev
COPY ./static ./static
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo build --locked --release  
# ENTRYPOINT ["tail", "-f", "/dev/null"]

FROM alpine:3.18 AS deploy
WORKDIR /app
# Copy the executable from the "build" stage.
COPY --from=build /actxol/target/release/actxol /app/src/server
COPY --from=build /actxol/static .
# ENTRYPOINT ["tail", "-f", "/dev/null"]
# What the container should run when it is started.
CMD ["/app/src/server"]