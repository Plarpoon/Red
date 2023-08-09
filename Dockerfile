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
ENV MYSQL_USER=EvilBunny
#ENV MYSQL_PASSWORD=UNCOMMENT AND WRITE HERE YOUR PASSWORD
ENV MYSQL_DATABASE=evil_db

ENTRYPOINT ["dotnet", "EvilBunny.dll"]
