# ─── Stage 1: Dependencies ──────────────────────────────────────────────────
FROM node:18-alpine AS deps
WORKDIR /app

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies with cache optimization
RUN npm ci

# ─── Stage 2: Builder ────────────────────────────────────────────────────────
FROM node:18-alpine AS builder
WORKDIR /app

# Set to production for optimized build
ENV NODE_ENV=production

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Build the app
RUN npm run build

# ─── Stage 3: Runner ─────────────────────────────────────────────────────────
FROM node:18-alpine AS runner
WORKDIR /app

# Set to production for runtime
ENV NODE_ENV=production

# Create a non-root user for security
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy only necessary files from builder
# COPY --from=builder /app/next.config.js ./
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json

# Set proper ownership
RUN chown -R nextjs:nodejs /app

# Switch to non-root user
USER nextjs

# Expose the port Next.js runs on
EXPOSE 3000

# Start the app - environment variables will be provided by docker-compose
CMD ["npm", "start"]