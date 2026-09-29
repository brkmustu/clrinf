FROM oven/bun:1.3.14-alpine AS base
WORKDIR /usr/src/app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --production

COPY src ./src
COPY public ./public

EXPOSE 4200
ENV PORT=4200
ENV HOST=0.0.0.0
ENV NODE_ENV=development

CMD ["bun", "src/server.ts"]
