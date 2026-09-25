using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Features.Kargolar;

public static class KargoBusinessMessages
{
    public const string SectionName = "Kargo";
    public const string KargoNotExists = "Kargo bulunamadı.";
    public const string KargoAlreadyExists = "Bu Kargo zaten mevcut.";
}

public class KargoBusinessRules : BaseBusinessRules
{
    private readonly IKargoRepository _kargoRepository;

    public KargoBusinessRules(IKargoRepository kargoRepository)
    {
        _kargoRepository = kargoRepository;
    }

    public async Task KargoShouldExistWhenSelected(Kargo? kargo)
    {
        if (kargo == null)
            throw new BusinessException(KargoBusinessMessages.KargoNotExists);
    }

    public async Task KargoIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Kargo? kargo = await _kargoRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await KargoShouldExistWhenSelected(kargo);
    }
}
