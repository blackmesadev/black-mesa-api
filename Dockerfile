FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /usr/src/black-mesa-api

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /usr/src/black-mesa-api/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust AS runtime
COPY --from=build /usr/src/black-mesa-api/target/release/black-mesa-api .
EXPOSE 8080
CMD ["./black-mesa-api"]