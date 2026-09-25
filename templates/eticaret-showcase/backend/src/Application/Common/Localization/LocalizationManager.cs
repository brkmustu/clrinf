using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Localization;

public class LocalizationManager : ILocalizationService
{
    public Task<string> GetLocalizedAsync(string key, string section)
    {
        return Task.FromResult(key);
    }
}
