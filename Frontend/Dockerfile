# ---- Giai đoạn 1: build giao diện (Presentation layer) ----
FROM node:20-alpine AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
# false = gọi backend thật qua /api, không dùng dữ liệu giả
#ARG VITE_USE_MOCK=false
#ENV VITE_USE_MOCK=$VITE_USE_MOCK
RUN npm run build

# ---- Giai đoạn 2: nginx phục vụ file tĩnh và chuyển /api sang backend ----
FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
