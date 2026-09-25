using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Features.Odemeler;

public static class OdemeBusinessMessages
{
    public const string SectionName = "Odeme";
    public const string OdemeNotExists = "Odeme bulunamadı.";
    public const string OdemeAlreadyExists = "Bu Odeme zaten mevcut.";
}

public class OdemeBusinessRules : BaseBusinessRules
{
    private readonly IOdemeRepository _odemeRepository;

    public OdemeBusinessRules(IOdemeRepository odemeRepository)
    {
        _odemeRepository = odemeRepository;
    }

    public async Task OdemeShouldExistWhenSelected(Odeme? odeme)
    {
        if (odeme == null)
            throw new BusinessException(OdemeBusinessMessages.OdemeNotExists);
    }

    public async Task OdemeIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Odeme? odeme = await _odemeRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await OdemeShouldExistWhenSelected(odeme);
    }
}
