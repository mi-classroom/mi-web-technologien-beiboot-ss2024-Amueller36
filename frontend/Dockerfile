# Use a Node.js image for the build stage
FROM node:alpine3.20 AS build-stage

WORKDIR /app

# Copy package.json and yarn.lock and install dependencies
COPY package.json yarn.lock ./
RUN yarn install --frozen-lockfile --network-concurrency 10 && yarn cache clean

# Copy the rest of the application code and build the application
COPY . .

ARG VITE_BACKEND_URL
ENV VITE_BACKEND_URL=${VITE_BACKEND_URL}

RUN yarn build

# Use a new stage to install only production dependencies
FROM node:20-bullseye AS prod-deps

WORKDIR /app

# Copy package.json and yarn.lock and install production dependencies
COPY package.json yarn.lock ./
RUN yarn install --frozen-lockfile --production --network-concurrency 10 && yarn cache clean

# Use the latest Nginx image for the production stage
FROM nginx:alpine AS prod


COPY nginx.conf /etc/nginx/conf.d/default.conf
# Copy production node_modules and built assets from the build stage
COPY --from=prod-deps /app/node_modules /app/node_modules
COPY --from=build-stage /app/dist /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start Nginx server
CMD ["nginx", "-g", "daemon off;"]
