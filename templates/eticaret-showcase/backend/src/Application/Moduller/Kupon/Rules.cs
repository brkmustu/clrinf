using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Features.Kuponlar;

public static class KuponBusinessMessages
{
    public const string SectionName = "Kupon";
    public const string KuponNotExists = "Kupon bulunamadı.";
    public const string KuponAlreadyExists = "Bu Kupon zaten mevcut.";
}

public class KuponBusinessRules : BaseBusinessRules
{
    private readonly IKuponRepository _kuponRepository;

    public KuponBusinessRules(IKuponRepository kuponRepository)
    {
        _kuponRepository = kuponRepository;
    }

    public async Task KuponShouldExistWhenSelected(Kupon? kupon)
    {
        if (kupon == null)
            throw new BusinessException(KuponBusinessMessages.KuponNotExists);
    }

    public async Task KuponIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Kupon? kupon = await _kuponRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await KuponShouldExistWhenSelected(kupon);
    }
}
