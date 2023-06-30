FROM rustlang/rust:nightly

RUN apt-get update \
    && apt-get install -y pkg-config libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/multithread_raycaster_game
COPY . .

RUN cargo build --release
