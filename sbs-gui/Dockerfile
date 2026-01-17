# Stage 1: Build Web Artifacts
FROM gradle:8.5-jdk17 AS builder

# Install Node.js (Required for Webpack bundling)
RUN apt-get update && apt-get install -y nodejs npm

WORKDIR /app

# Copy configuration first for caching
COPY build.gradle.kts settings.gradle.kts gradle.properties ./
COPY composeApp/build.gradle.kts composeApp/

# Copy source code
COPY composeApp/src composeApp/src

# Build JS distribution
# -Xmx2g: Prevents OutOfMemoryError
# --stacktrace: Shows errors clearly
RUN gradle :composeApp:jsBrowserDistribution --no-daemon --stacktrace -Dorg.gradle.jvmargs="-Xmx2g"

# Stage 2: Serve with Nginx
FROM nginx:alpine

# Copy static assets
# The output path for JS is different from Wasm
COPY --from=builder /app/composeApp/build/dist/js/productionExecutable /usr/share/nginx/html

# Expose port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
