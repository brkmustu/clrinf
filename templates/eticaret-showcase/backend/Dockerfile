FROM mcr.microsoft.com/dotnet/sdk:10.0 AS build
WORKDIR /src

COPY core/csharp/ ./core/csharp/
COPY templates/eticaret-showcase/backend/ ./templates/eticaret-showcase/backend/

WORKDIR /src/templates/eticaret-showcase/backend/src/WebAPI
RUN dotnet restore WebAPI.csproj
RUN dotnet publish WebAPI.csproj -c Release -o /app/publish /p:UseAppHost=false

FROM mcr.microsoft.com/dotnet/aspnet:10.0 AS final
WORKDIR /app
COPY --from=build /app/publish .
EXPOSE 5000
ENV ASPNETCORE_URLS=http://+:5000
ENTRYPOINT ["dotnet", "WebAPI.dll"]
