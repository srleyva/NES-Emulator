from rust:1.52-alpine

COPY . .

RUN apk add sdl2-dev httpd

RUN cargo build