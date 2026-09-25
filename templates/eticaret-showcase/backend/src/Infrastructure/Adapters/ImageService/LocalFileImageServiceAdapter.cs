using System;
using System.IO;
using System.Threading.Tasks;
using EticaretApp.Application.Services.ImageService;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;

namespace EticaretApp.Infrastructure.Adapters.ImageService;

public class LocalFileImageServiceAdapter : ImageServiceBase
{
    private readonly string _uploadFolderPath;

    public LocalFileImageServiceAdapter(IConfiguration configuration)
    {
        string configuredFolder = configuration["Storage:LocalUploadDirectory"] ?? "uploads";
        _uploadFolderPath = Path.Combine(Directory.GetCurrentDirectory(), "wwwroot", configuredFolder);
        if (!Directory.Exists(_uploadFolderPath))
        {
            Directory.CreateDirectory(_uploadFolderPath);
        }
    }

    public override async Task<string> UploadAsync(IFormFile formFile)
    {
        await FileMustBeInImageFormat(formFile);

        string extension = Path.GetExtension(formFile.FileName);
        string uniqueFileName = $"{Guid.NewGuid():N}{extension}";
        string fullPath = Path.Combine(_uploadFolderPath, uniqueFileName);

        using var stream = new FileStream(fullPath, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
        await formFile.CopyToAsync(stream);

        return $"/uploads/{uniqueFileName}";
    }

    public override Task DeleteAsync(string imageUrl)
    {
        if (string.IsNullOrWhiteSpace(imageUrl))
            return Task.CompletedTask;

        string fileName = Path.GetFileName(imageUrl);
        string fullPath = Path.Combine(_uploadFolderPath, fileName);

        if (File.Exists(fullPath))
        {
            File.Delete(fullPath);
        }

        return Task.CompletedTask;
    }
}
