FROM rust:1.78-bookworm

WORKDIR /app

COPY . .

EXPOSE 8080

CMD ["cargo", "run", "--release", "-p", "frontend"]
