using Domain.Configuration.Models;
using Xunit;

namespace ClrinfCS.Tests;

public class ArchProfileTests
{
    [Fact]
    public void BackendConfig_ArchStyle_Flat_Properties()
    {
        var backend = new BackendConfig
        {
            ArchStyle = "flat",
            Pattern = "basic",
            HostType = "mvc"
        };

        Assert.True(backend.IsFlat);
        Assert.False(backend.IsLayered);
        Assert.False(backend.IsCleanCqrs);
        Assert.True(backend.IsBasicPattern);
        Assert.False(backend.IsSeparatedPattern);
        Assert.True(backend.IsMvcHost);
        Assert.False(backend.IsApiHost);
        Assert.False(backend.IsConsoleHost);
    }

    [Fact]
    public void BackendConfig_ArchStyle_Layered_Properties()
    {
        var backend = new BackendConfig
        {
            ArchStyle = "layered",
            Pattern = "separated",
            HostType = "api"
        };

        Assert.False(backend.IsFlat);
        Assert.True(backend.IsLayered);
        Assert.False(backend.IsCleanCqrs);
        Assert.False(backend.IsBasicPattern);
        Assert.True(backend.IsSeparatedPattern);
        Assert.True(backend.IsApiHost);
    }

    [Fact]
    public void BackendConfig_ArchStyle_CleanCqrs_Defaults()
    {
        var backend = new BackendConfig();

        Assert.True(backend.IsCleanCqrs);
        Assert.True(backend.IsSeparatedPattern);
        Assert.True(backend.IsApiHost);
    }

    [Fact]
    public void BackendLayersConfig_BackwardCompatibility_HostAndApi()
    {
        var layers = new BackendLayersConfig
        {
            Api = "MyProject.API"
        };

        Assert.Equal("MyProject.API", layers.Api);
        Assert.Equal("MyProject.API", layers.WebApi);

        layers.Host = "MyProject.Web";
        Assert.Equal("MyProject.Web", layers.Host);
    }
}
