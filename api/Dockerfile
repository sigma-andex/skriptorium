# syntax=docker/dockerfile:1
FROM node:17 AS builder

COPY . /app

WORKDIR /app 

RUN npm run setup && npm run bundle

FROM node:17-alpine 

COPY --from=builder /app/target /app

WORKDIR /app

EXPOSE 8080

ENTRYPOINT [ "node" , "index.js"]
