# Build stage
FROM node:18 AS build

# Set the working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy the rest of the application code
COPY . .

# Build the application
RUN npm run build

# Production stage
FROM node:18-slim

# Set the working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install only production dependencies
RUN npm install --only=production

# Copy built application from build stage
COPY --from=build /app/.next ./.next
COPY --from=build /app/public ./public
# Add any other necessary files for your Next.js app

# Copy .env file
COPY .env .env

# Expose the port
EXPOSE 3000

# Start the application
CMD ["npm", "start"]
