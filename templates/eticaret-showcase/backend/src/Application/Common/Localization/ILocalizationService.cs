using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Localization;

public interface ILocalizationService
{
    Task<string> GetLocalizedAsync(string key, string section);
}
