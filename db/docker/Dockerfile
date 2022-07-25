FROM alpine:3.16

WORKDIR /app

## copies the debug version into the
COPY ../../server/target/debug/text-graph_rust /app/main

CMD ./main
