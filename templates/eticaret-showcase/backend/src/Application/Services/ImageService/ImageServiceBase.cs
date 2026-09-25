using Microsoft.AspNetCore.Http;
using EticaretApp.Application.Common.Exceptions;

namespace EticaretApp.Application.Services.ImageService;

public abstract class ImageServiceBase
{
    public abstract Task<string> UploadAsync(IFormFile formFile);

    public async Task<string> UpdateAsync(IFormFile formFile, string imageUrl)
    {
        await FileMustBeInImageFormat(formFile);

        await DeleteAsync(imageUrl);
        return await UploadAsync(formFile);
    }

    public abstract Task DeleteAsync(string imageUrl);

    protected static async Task FileMustBeInImageFormat(IFormFile formFile)
    {
        List<string> extensions = [".jpg", ".png", ".jpeg", ".webp"];

        string extension = Path.GetExtension(formFile.FileName).ToLower();
        if (!extensions.Contains(extension))
            throw new BusinessException("Unsupported format");
        await Task.CompletedTask;
    }
}
