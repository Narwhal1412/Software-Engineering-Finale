# ---- Giai đoạn 1: build giao diện (Presentation layer) ----
FROM node:20-slim AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .

RUN npm run build

# ---- Giai đoạn 2: nginx phục vụ file tĩnh và chuyển /api sang backend ----
FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
