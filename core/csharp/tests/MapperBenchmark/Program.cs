using System;
using System.Collections.Generic;
using System.Linq;
using AutoMapper;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;

namespace MapperBenchmark;

public class Program
{
    public static void Main(string[] args)
    {
        Console.WriteLine("Running Mapper Benchmark (AutoMapper vs Static Extension Mapper)...");
        var summary = BenchmarkRunner.Run<MapperComparisonBenchmark>();
    }
}

// Domain Entity & DTO
public class Stok
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public decimal Price { get; set; }
    public int Quantity { get; set; }
    public DateTime CreatedAt { get; set; }
}

public class StokDto
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public decimal Price { get; set; }
    public int Quantity { get; set; }
    public DateTime CreatedAt { get; set; }
}

// Static Extension Mapper (Our Zero-Cost Alternative)
public static class StokMappingExtensions
{
    public static StokDto ToDto(this Stok entity)
    {
        return new StokDto
        {
            Id = entity.Id,
            Name = entity.Name,
            Price = entity.Price,
            Quantity = entity.Quantity,
            CreatedAt = entity.CreatedAt
        };
    }
}

[MemoryDiagnoser]
[RankColumn]
public class MapperComparisonBenchmark
{
    private IMapper _autoMapper = null!;
    private Stok _sampleStok = null!;
    private List<Stok> _stokList = null!;

    [GlobalSetup]
    public void Setup()
    {
        // 1. Configure AutoMapper
        var config = new MapperConfiguration(cfg =>
        {
            cfg.CreateMap<Stok, StokDto>();
        });
        _autoMapper = config.CreateMapper();

        // 2. Prepare Sample Data
        _sampleStok = new Stok
        {
            Id = 1,
            Name = "Laptop Computer",
            Price = 45000.50m,
            Quantity = 25,
            CreatedAt = DateTime.UtcNow
        };

        _stokList = Enumerable.Range(1, 1000).Select(i => new Stok
        {
            Id = i,
            Name = $"Item {i}",
            Price = i * 10.5m,
            Quantity = i,
            CreatedAt = DateTime.UtcNow
        }).ToList();
    }

    [Benchmark(Baseline = true)]
    public StokDto SingleObject_AutoMapper()
    {
        return _autoMapper.Map<StokDto>(_sampleStok);
    }

    [Benchmark]
    public StokDto SingleObject_ExtensionMapper()
    {
        return _sampleStok.ToDto();
    }

    [Benchmark]
    public List<StokDto> List1000_AutoMapper()
    {
        return _autoMapper.Map<List<StokDto>>(_stokList);
    }

    [Benchmark]
    public List<StokDto> List1000_ExtensionMapper()
    {
        return _stokList.Select(x => x.ToDto()).ToList();
    }
}
