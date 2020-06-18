FROM rust:latest as rust-dependency-builder

# Start a placeholder project for dependency caching
RUN USER=root cargo new --lib wasm-game-of-life
WORKDIR /wasm-game-of-life
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

FROM rust-dependency-builder as rust-pkg-builder
RUN rm -rf src
COPY src/ ./src/

RUN cargo install wasm-pack
RUN wasm-pack build

FROM node:latest as web-dependency-builder

# Install npm dependecies
WORKDIR /www
COPY /www/package.json ./
COPY --from=rust-pkg-builder \
  /wasm-game-of-life/pkg/ \
  /pkg/
RUN npm install

COPY /www/bootstrap.js ./
COPY /www/index.html ./
COPY /www/index.js ./
COPY /www/webpack.config.js ./

RUN npm run build

FROM nginx:alpine
COPY ./nginx.conf /etc/nginx/
COPY --from=web-dependency-builder \
  /www/dist/ \
  /usr/share/nginx/html/