FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build-env
WORKDIR /app
# Copy csproj and restore
COPY *.csproj ./
RUN dotnet restore
# Copy everything else and build
COPY . ./
RUN dotnet publish -o out

# Build runtime image
FROM mcr.microsoft.com/dotnet/runtime:8.0
WORKDIR /app
COPY --from=build-env /app/out .

ENTRYPOINT ["dotnet", "EvilBunny.dll"]
