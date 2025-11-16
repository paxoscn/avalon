FROM node:22-alpine3.21 AS builder

# if you located in China, you can use aliyun mirror to speed up
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# if you located in China, you can use taobao registry to speed up
RUN npm config set registry https://registry.npmmirror.com

WORKDIR /app
COPY . .
RUN npm install
RUN npm run build

FROM nginx:stable-alpine
COPY --from=builder /app/build /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
