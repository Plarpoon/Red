FROM mcr.microsoft.com/dotnet/sdk:7.0 AS build-env
WORKDIR /app
# Copy csproj and restore
COPY *.csproj ./
RUN dotnet restore
# Copy everything else and build
COPY . ./
RUN dotnet publish -o out

# Build runtime image
FROM mcr.microsoft.com/dotnet/runtime:7.0
WORKDIR /app
COPY --from=build-env /app/out .

# Use MySQL Docker image as a build environment
FROM mysql:8.0 AS mysql-env

# Set environment variables for MySQL
ENV MYSQL_USER=evilbunny
ENV MYSQL_DATABASE=evil_db

# Read MYSQL_PASSWORD from config.yaml file using yq
RUN export MYSQL_PASSWORD=$(yq e '.DB_PASS' config.yaml)

# Install yq to parse YAML files
RUN apt-get update && apt-get install -y jq && \
    wget https://github.com/mikefarah/yq/releases/download/v4.13.5/yq_linux_amd64 -O /usr/bin/yq && \
    chmod +x /usr/bin/yq

# Copy config.yaml file to the container
COPY config.yaml .


ENTRYPOINT ["dotnet", "EvilBunny.dll"]
