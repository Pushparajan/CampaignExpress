# =============================================================================
# Campaign Express UI — Multi-stage Docker build
# =============================================================================
# Next.js 14 / React 18 / TanStack Query 5 / Tailwind CSS
# =============================================================================

# -- Dependencies stage --
FROM node:20-alpine AS deps
WORKDIR /app
COPY ui/package.json ui/package-lock.json* ./
RUN npm ci --ignore-scripts

# -- Build stage --
FROM node:20-alpine AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY ui/ .

# API URL injected at build time — defaults to in-cluster service
ENV NEXT_PUBLIC_API_URL=http://campaign-express.campaign-express.svc.cluster.local:8080

RUN npm run build

# -- Runtime stage --
FROM node:20-alpine AS runtime
WORKDIR /app

RUN addgroup -S campaign && adduser -S campaign -G campaign

COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

USER campaign

EXPOSE 3000

ENV NODE_ENV=production
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=3 \
    CMD wget -q --spider http://localhost:3000/ || exit 1

CMD ["node", "server.js"]
