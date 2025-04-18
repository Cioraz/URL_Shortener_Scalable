# ─── Stage 1: Build the Next.js app ──────────────────────────────────────────
FROM node:18-alpine AS builder

# Create app directory
WORKDIR /app

# Install all deps (prod + dev) for build
COPY package.json package-lock.json ./
RUN npm ci

# Copy source & build
COPY . .
RUN npm run build


# ─── Stage 2: Runtime image ─────────────────────────────────────────────────
FROM node:18-alpine AS production

WORKDIR /app

# Install only production deps
COPY package.json package-lock.json ./
RUN npm ci --omit=dev

# Copy build artifacts
COPY --from=builder /app/.next .next
COPY --from=builder /app/public public

# If you have a next.config.js, uncomment the line below:
# COPY --from=builder /app/next.config.js ./

# Copy env (optional—consider injecting at runtime instead)
COPY .env .env

# Expose & run
EXPOSE 3000
CMD ["npm", "start"]
