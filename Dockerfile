FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

COPY api-switch /app/api-switch

RUN mkdir -p /app/data && chmod 755 /app/api-switch

COPY static /app/static

EXPOSE 9091

ENV RUST_LOG=info
ENV DATABASE_PATH=/app/data/api-switch.db
ENV PORT=9091

CMD ["/app/api-switch"]
