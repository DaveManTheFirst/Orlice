FROM rust:latest as build

WORKDIR /usr/src/orlice

COPY Cargo.toml Cargo.toml
COPY ./src ./src
COPY LICENSE LICENSE
COPY README.md README.md

RUN rustup update
RUN cargo install wasm-pack
RUN wasm-pack build --target web


FROM node:alpine

LABEL author="David Gerhardinger"
LABEL version="1.0"
LABEL description="Orlice on wasm - Dockerfile"

WORKDIR /usr/src/orlice

COPY --from=build /usr/src/orlice/pkg /usr/src/orlice/pkg
COPY ./www ./www

WORKDIR www

EXPOSE 8080

RUN npm install

ENTRYPOINT ["npm", "start"]

