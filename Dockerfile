FROM --platform=linux/amd64 python:3.12-slim

# Установите Rust
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libclang-dev \
    llvm-dev \
    clang \
    gcc \
    g++ \
    make \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Установите Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Установите maturin
RUN pip install maturin

WORKDIR /app
COPY . .

# Соберите проект
RUN maturin build --release --target x86_64-unknown-linux-gnu -i python3.12