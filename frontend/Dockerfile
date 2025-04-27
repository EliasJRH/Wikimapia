# Build stage (install all deps)
FROM oven/bun:latest AS builder
COPY . .
RUN bun install
RUN bun run build

# Runtime stage (only static files + production deps)
FROM oven/bun:latest
COPY --from=builder /home/bun/app/dist ./dist
CMD ["bunx", "serve", "dist"]
